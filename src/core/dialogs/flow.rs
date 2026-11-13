//! What the project dialogs' answers do: new, open, import, save / save as, export, unsaved changes,
//! closing the window and recovery copies.

use super::DialogHost;
use crate::commands::workspace::{Pending, Workspace};
use crate::features::projects::components::open_dialog::OPEN_ERROR;
use crate::features::projects::components::{
    recovery_dialog::RecoveryChoice, unsaved_dialog::UnsavedChoice,
};
use crate::features::projects::logic::save_plan::{save_target, SaveTarget};
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
    pub(super) fn on_change(&mut self, ctx: &Context, workspace: &mut Workspace) {
        if self.active == self.shown {
            return;
        }
        match self.active {
            Some(Dialog::SaveProject) => self.prepare_save(ctx, workspace, false),
            Some(Dialog::SaveAs) => self.prepare_save(ctx, workspace, true),
            _ => {}
        }
        self.shown = self.active;
    }

    /// "Save" writes straight to the project's own `.gr`; otherwise (first save, "save as", a Guanacaste 3D
    /// original, a read-only project) it asks for a name, explaining why when it was not the user's choice.
    fn prepare_save(&mut self, ctx: &Context, workspace: &mut Workspace, as_new: bool) {
        let Some(project) = &workspace.project else {
            workspace.notify(tr("workspace.nothing_to_save").to_owned(), true);
            self.active = None;
            return;
        };
        let (suggested, reason) = match save_target(project) {
            SaveTarget::Overwrite(path) if !as_new => {
                self.active = None;
                if workspace.save_to(&path, self.save.thumbnail) {
                    if let Some(pending) = self.pending.take() {
                        self.execute(ctx, workspace, pending);
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
    pub(super) fn perform(&mut self, ctx: &Context, workspace: &mut Workspace, pending: Pending) {
        if workspace.dirty() {
            self.pending = Some(pending);
            self.active = Some(Dialog::UnsavedChanges);
            self.shown = self.active;
        } else {
            self.execute(ctx, workspace, pending);
        }
    }

    fn execute(&mut self, ctx: &Context, workspace: &mut Workspace, pending: Pending) {
        self.active = None;
        match pending {
            Pending::Close => {
                workspace.close_confirmed = true;
                ctx.send_viewport_cmd(ViewportCommand::Close);
            }
            Pending::New(name) => workspace.start_new(&name),
            Pending::Open(path) => {
                if !workspace.open_file(&path) {
                    self.active = Some(Dialog::Error(OPEN_ERROR));
                }
            }
            Pending::Import => {
                if let (Some(Ok(outcome)), Some(path)) =
                    (self.import.result.as_deref(), self.import.path.clone())
                {
                    workspace.adopt_import(outcome, &path);
                }
            }
        }
        self.shown = self.active;
    }

    /// Handles files dropped on the window: a Guanacaste Render project opens as a project, anything else goes
    /// to the import dialog (which explains closed formats). Only the first file is used.
    pub(super) fn take_drops(&mut self, ctx: &Context, workspace: &mut Workspace) {
        use crate::features::import_export::logic::formats::{detect, sniff, Detected, Signature};
        use crate::features::import_export::ui::drop;
        use crate::services::localization::trf;
        drop::overlay(ctx);
        let paths = drop::dropped_paths(ctx);
        let Some(first) = paths.first().cloned() else {
            return;
        };
        if !matches!(self.active, None | Some(Dialog::Import)) {
            return;
        }
        if paths.len() > 1 {
            let name = first
                .file_name()
                .map_or_else(String::new, |n| n.to_string_lossy().into_owned());
            workspace.notify(
                trf("import.drop_many", &[&paths.len().to_string(), &name]),
                false,
            );
        }
        let is_project = matches!(detect(&first), Detected::Supported(_))
            && sniff(&first).is_ok_and(|s| s == Signature::Render);
        if is_project {
            self.perform(ctx, workspace, Pending::Open(first));
        } else {
            self.import.choose(first);
            self.active = Some(Dialog::Import);
            self.menu_anchor = None;
        }
    }

    /// Opens `path` at start-up (the recovery offer, if any, stays queued behind it).
    pub fn open_at_start(&mut self, workspace: &mut Workspace, path: &std::path::Path) {
        if !workspace.open_file(path) {
            self.active = Some(Dialog::Error(OPEN_ERROR));
        }
    }

    /// Asks before the window closes with unsaved changes. Returns true when the close must be cancelled.
    pub fn intercept_close(&mut self, ctx: &Context, workspace: &mut Workspace) -> bool {
        if !workspace.dirty() || workspace.close_confirmed {
            return false;
        }
        self.perform(ctx, workspace, Pending::Close);
        true
    }

    /// Carries out what the dialogs recorded this frame; dialogs it opens are already prepared.
    pub(super) fn after(&mut self, ctx: &Context, workspace: &mut Workspace) {
        if std::mem::take(&mut self.new_scene.create) {
            let name = self.new_scene.name.trim();
            let name = if name.is_empty() {
                tr("new.name_hint").to_owned()
            } else {
                name.to_owned()
            };
            self.perform(ctx, workspace, Pending::New(name));
        }
        if let Some(path) = self.open.request.take() {
            self.perform(ctx, workspace, Pending::Open(path));
        }
        if std::mem::take(&mut self.import.adopt) {
            self.perform(ctx, workspace, Pending::Import);
        }
        if let Some(path) = self.save.request.take() {
            if workspace.save_to(&path, self.save.thumbnail) {
                self.active = None;
                if let Some(pending) = self.pending.take() {
                    self.execute(ctx, workspace, pending);
                }
            }
        }
        if std::mem::take(&mut self.export.request) {
            self.export();
        }
        self.shown = self.active;
    }

    /// Hands the export settings to the shell, which queues the render and reports the result.
    fn export(&mut self) {
        self.render_request = Some(self.export.clone());
        self.active = None;
    }

    /// Answer of the unsaved-changes dialog.
    pub(super) fn settle(
        &mut self,
        ctx: &Context,
        workspace: &mut Workspace,
        choice: UnsavedChoice,
    ) -> DialogOutcome {
        match choice {
            UnsavedChoice::Save => {
                self.prepare_save(ctx, workspace, false);
                self.shown = self.active;
            }
            UnsavedChoice::Discard => {
                workspace.forget_changes();
                if let Some(pending) = self.pending.take() {
                    self.execute(ctx, workspace, pending);
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
    pub(super) fn recover(
        &mut self,
        workspace: &mut Workspace,
        choice: RecoveryChoice,
    ) -> DialogOutcome {
        match choice {
            RecoveryChoice::Restore(index) => {
                workspace.restore(index);
                if !workspace.recovery.is_empty() {
                    return DialogOutcome::Keep;
                }
            }
            RecoveryChoice::DiscardAll => workspace.discard_recovery(),
            RecoveryChoice::Later => workspace.recovery.clear(),
        }
        DialogOutcome::Close
    }
}
