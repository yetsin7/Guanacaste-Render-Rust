//! What the project dialogs' answers do: new, open, import, save / save as, export, unsaved changes,
//! closing the window and recovery copies.

use super::DialogHost;
use crate::commands::workspace::Pending;
use crate::features::projects::components::open_dialog::OPEN_ERROR;
use crate::features::projects::components::{
    recovery_dialog::RecoveryChoice, unsaved_dialog::UnsavedChoice,
};
use crate::features::projects::logic::save_plan::{save_target, SaveTarget};
use crate::features::rendering::models::{ImageFormat, RESOLUTIONS};
use crate::models::dialog::{Dialog, DialogOutcome};
use crate::services::localization::tr;
use eframe::egui::{Context, ViewportCommand};
use std::path::PathBuf;

/// Folder proposed when nothing better is known: the user's Documents folder.
fn default_folder() -> String {
    std::env::var_os("USERPROFILE")
        .or_else(|| std::env::var_os("HOME"))
        .map(|home| PathBuf::from(home).join("Documents").display().to_string())
        .unwrap_or_default()
}

impl DialogHost {
    /// Prepares a dialog the first frame it becomes active.
    pub(super) fn on_change(&mut self, ctx: &Context) {
        if self.active == self.shown {
            return;
        }
        match self.active {
            Some(Dialog::SaveProject) => self.prepare_save(ctx, false),
            Some(Dialog::SaveAs) => self.prepare_save(ctx, true),
            _ => {}
        }
        self.shown = self.active;
    }

    /// "Save" writes straight to the project's own `.gr`; otherwise (first save, "save as", a Guanacaste 3D
    /// original, a read-only project) it asks for a name, explaining why when it was not the user's choice.
    fn prepare_save(&mut self, ctx: &Context, as_new: bool) {
        let Some(project) = &self.workspace.project else {
            self.workspace
                .notify(tr("workspace.nothing_to_save").to_owned(), true);
            self.active = None;
            return;
        };
        let (suggested, reason) = match save_target(project) {
            SaveTarget::Overwrite(path) if !as_new => {
                self.active = None;
                if self.workspace.save_to(&path, self.save.thumbnail) {
                    if let Some(pending) = self.pending.take() {
                        self.execute(ctx, pending);
                    }
                }
                return;
            }
            SaveTarget::Overwrite(path) => (path, None),
            SaveTarget::AskAs {
                suggested,
                reason_key,
            } => (suggested, reason_key),
        };
        self.save.propose(&suggested, reason);
        if self.save.folder.trim().is_empty() {
            self.save.folder = if self.new_scene.folder.trim().is_empty() {
                default_folder()
            } else {
                self.new_scene.folder.clone()
            };
        }
        self.active = Some(if as_new {
            Dialog::SaveAs
        } else {
            Dialog::SaveProject
        });
    }

    /// Runs `pending`, or first asks about unsaved changes.
    fn perform(&mut self, ctx: &Context, pending: Pending) {
        if self.workspace.dirty() {
            self.pending = Some(pending);
            self.active = Some(Dialog::UnsavedChanges);
            self.shown = self.active;
        } else {
            self.execute(ctx, pending);
        }
    }

    fn execute(&mut self, ctx: &Context, pending: Pending) {
        self.active = None;
        match pending {
            Pending::Close => {
                self.workspace.close_confirmed = true;
                ctx.send_viewport_cmd(ViewportCommand::Close);
            }
            Pending::New(name) => self.workspace.start_new(&name),
            Pending::Open(path) => {
                if !self.workspace.open_file(&path) {
                    self.active = Some(Dialog::Error(OPEN_ERROR));
                }
            }
            Pending::Import => {
                if let (Some(Ok(outcome)), Some(path)) =
                    (self.import.result.as_deref(), self.import.path.clone())
                {
                    self.workspace.adopt_import(outcome, &path);
                }
            }
        }
        self.shown = self.active;
    }

    /// Opens `path` at start-up (the recovery offer, if any, stays queued behind it).
    pub fn open_at_start(&mut self, path: &std::path::Path) {
        if !self.workspace.open_file(path) {
            self.active = Some(Dialog::Error(OPEN_ERROR));
        }
    }

    /// Asks before the window closes with unsaved changes. Returns true when the close must be cancelled.
    pub fn intercept_close(&mut self, ctx: &Context) -> bool {
        if !self.workspace.dirty() || self.workspace.close_confirmed {
            return false;
        }
        self.perform(ctx, Pending::Close);
        true
    }

    /// Carries out what the dialogs recorded this frame; dialogs it opens are already prepared.
    pub(super) fn after(&mut self, ctx: &Context) {
        if std::mem::take(&mut self.new_scene.create) {
            let name = self.new_scene.name.trim();
            let name = if name.is_empty() {
                tr("new.name_hint").to_owned()
            } else {
                name.to_owned()
            };
            self.perform(ctx, Pending::New(name));
        }
        if let Some(path) = self.open.request.take() {
            self.perform(ctx, Pending::Open(path));
        }
        if std::mem::take(&mut self.import.adopt) {
            self.perform(ctx, Pending::Import);
        }
        if let Some(path) = self.save.request.take() {
            if self.workspace.save_to(&path, self.save.thumbnail) {
                self.active = None;
                if let Some(pending) = self.pending.take() {
                    self.execute(ctx, pending);
                }
            }
        }
        if std::mem::take(&mut self.export.request) {
            self.export();
        }
        self.shown = self.active;
    }

    fn export(&mut self) {
        if ImageFormat::ALL[self.export.format] != ImageFormat::Png {
            self.workspace
                .notify(tr("workspace.export_only_png").to_owned(), true);
            return;
        }
        let (width, height) = RESOLUTIONS[self.export.resolution.min(RESOLUTIONS.len() - 1)];
        self.workspace
            .export_png(&PathBuf::from(self.export.path.trim()), width, height);
        if self.workspace.status.as_ref().is_some_and(|s| !s.error) {
            self.active = None;
        }
    }

    /// Answer of the unsaved-changes dialog.
    pub(super) fn settle(&mut self, ctx: &Context, choice: UnsavedChoice) -> DialogOutcome {
        match choice {
            UnsavedChoice::Save => {
                self.prepare_save(ctx, false);
                self.shown = self.active;
            }
            UnsavedChoice::Discard => {
                self.workspace.forget_changes();
                if let Some(pending) = self.pending.take() {
                    self.execute(ctx, pending);
                }
            }
            UnsavedChoice::Cancel => {
                self.pending = None;
                self.active = None;
            }
        }
        DialogOutcome::Keep
    }

    /// Answer of the recovery dialog.
    pub(super) fn recover(&mut self, choice: RecoveryChoice) -> DialogOutcome {
        match choice {
            RecoveryChoice::Restore(index) => {
                self.workspace.restore(index);
                if !self.workspace.recovery.is_empty() {
                    return DialogOutcome::Keep;
                }
            }
            RecoveryChoice::DiscardAll => self.workspace.discard_recovery(),
            RecoveryChoice::Later => self.workspace.recovery.clear(),
        }
        DialogOutcome::Close
    }
}
