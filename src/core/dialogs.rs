//! Hosts every dialog, the application menu and the keyboard shortcuts that open them, plus the workspace
//! (the open project) that the project dialogs act on.

mod flow;

use crate::commands::{
    self,
    workspace::{Pending, Workspace},
};
use crate::components::{icons, menu, notice::Level, toast};
use crate::features::help::components::notifications_panel::{self, Notice};
use crate::features::help::components::{about_dialog, error_dialog, shortcuts_dialog};
use crate::features::import_export::ui::{import_dialog, state::ImportDialogState};
use crate::features::projects::components::{
    new_scene_dialog, open_dialog, recovery_dialog, save_dialog, unsaved_dialog,
};
use crate::features::projects::models::{NewSceneState, OpenState, SaveState};
use crate::features::rendering::components::export_dialog;
use crate::features::rendering::models::ExportState;
use crate::features::settings::components::preferences_dialog;
use crate::models::dialog::{Dialog, DialogOutcome};
use crate::models::preferences::Preferences;
use crate::services::localization::tr;
use eframe::egui::{Context, Key, KeyboardShortcut, Modifiers, Pos2};
use std::path::PathBuf;
use std::time::Duration;

/// Keyboard shortcuts of the shell and the dialogs they open (Ctrl+Shift+S is checked first, for "save as").
const SHORTCUTS: [(Key, Dialog); 7] = [
    (Key::I, Dialog::Import),
    (Key::N, Dialog::NewScene),
    (Key::O, Dialog::OpenProject),
    (Key::S, Dialog::SaveProject),
    (Key::E, Dialog::ExportImage),
    (Key::Comma, Dialog::Preferences),
    (Key::Slash, Dialog::Shortcuts),
];

/// Menu entries: icon, label key, shortcut hint and the dialog each opens.
const MENU: [(char, &str, &str, Dialog); 9] = [
    (icons::UPLOAD, "menu.import", "Ctrl+I", Dialog::Import),
    (icons::ADD_BOX, "menu.new", "Ctrl+N", Dialog::NewScene),
    (
        icons::FOLDER_OPEN,
        "menu.open",
        "Ctrl+O",
        Dialog::OpenProject,
    ),
    (icons::SAVE, "menu.save", "Ctrl+S", Dialog::SaveProject),
    (icons::SAVE, "menu.save_as", "Ctrl+Shift+S", Dialog::SaveAs),
    (icons::IMAGE, "menu.export", "Ctrl+E", Dialog::ExportImage),
    (
        icons::SETTINGS,
        "menu.preferences",
        "Ctrl+,",
        Dialog::Preferences,
    ),
    (
        icons::KEYBOARD,
        "menu.shortcuts",
        "Ctrl+/",
        Dialog::Shortcuts,
    ),
    (icons::HELP, "menu.about", "", Dialog::About),
];

/// How long a status message stays on screen.
const STATUS_SECONDS: u64 = 8;

/// State of all dialogs plus the one currently shown.
pub struct DialogHost {
    active: Option<Dialog>,
    shown: Option<Dialog>,
    menu_anchor: Option<Pos2>,
    open: OpenState,
    save: SaveState,
    new_scene: NewSceneState,
    export: ExportState,
    preferences_tab: usize,
    shortcuts_search: String,
    notices: Vec<Notice>,
    import: ImportDialogState,
    pending: Option<Pending>,
    /// The open project and its files.
    pub workspace: Workspace,
}

impl Default for DialogHost {
    fn default() -> Self {
        Self::with_data_dir(crate::features::projects::services::recovery::data_dir())
    }
}

impl DialogHost {
    /// A host whose recent list and recovery copies live in `data_dir`; offers recovery when copies exist.
    pub fn with_data_dir(data_dir: PathBuf) -> Self {
        let workspace = Workspace::new(data_dir);
        let active = (!workspace.recovery.is_empty()).then_some(Dialog::Recovery);
        Self {
            active,
            shown: None,
            menu_anchor: None,
            open: OpenState::default(),
            save: SaveState::default(),
            new_scene: NewSceneState::default(),
            export: ExportState::default(),
            preferences_tab: 0,
            shortcuts_search: String::new(),
            notices: notifications_panel::initial_notices(),
            import: ImportDialogState::default(),
            pending: None,
            workspace,
        }
    }

    /// Opens the application menu with its top-right corner at `anchor`.
    pub fn open_menu(&mut self, anchor: Pos2) {
        self.menu_anchor = Some(anchor);
    }

    /// The dialog currently shown, if any.
    pub fn active(&self) -> Option<Dialog> {
        self.active
    }

    /// Serves requests and shortcuts, then draws the menu, the active dialog and the status message.
    pub fn show(&mut self, ctx: &Context, preferences: &mut Preferences) {
        self.intake(ctx);
        if self.active.is_none() && !ctx.egui_wants_keyboard_input() {
            let save_as = KeyboardShortcut::new(Modifiers::COMMAND | Modifiers::SHIFT, Key::S);
            if ctx.input_mut(|input| input.consume_shortcut(&save_as)) {
                self.active = Some(Dialog::SaveAs);
            }
            for (key, dialog) in SHORTCUTS {
                if ctx.input_mut(|input| {
                    input.consume_shortcut(&KeyboardShortcut::new(Modifiers::COMMAND, key))
                }) {
                    self.active = Some(dialog);
                }
            }
        }
        self.show_menu(ctx);
        self.on_change(ctx);
        if let Some(dialog) = self.active {
            self.draw(ctx, dialog, preferences);
        }
        self.after(ctx);
        self.workspace.autosave();
        if self.workspace.dirty() {
            ctx.request_repaint_after(Duration::from_secs(5));
        }
        self.show_status(ctx);
    }

    fn draw(&mut self, ctx: &Context, dialog: Dialog, preferences: &mut Preferences) {
        let outcome = match dialog {
            Dialog::NewScene => new_scene_dialog::show(ctx, &mut self.new_scene),
            Dialog::OpenProject => open_dialog::show(ctx, &mut self.open, &self.workspace.recent),
            Dialog::Import => import_dialog::show(ctx, &mut self.import),
            Dialog::SaveProject | Dialog::SaveAs => save_dialog::show(ctx, &mut self.save),
            Dialog::ExportImage => export_dialog::show(ctx, &mut self.export),
            Dialog::Preferences => {
                preferences_dialog::show(ctx, &mut self.preferences_tab, preferences)
            }
            Dialog::Shortcuts => shortcuts_dialog::show(ctx, &mut self.shortcuts_search),
            Dialog::About => about_dialog::show(ctx),
            Dialog::Notifications => {
                let anchor = ctx.content_rect().right_top() + eframe::egui::vec2(-12.0, 36.0);
                notifications_panel::show(ctx, anchor, &mut self.notices)
            }
            Dialog::Error(error) => error_dialog::show(ctx, &error),
            Dialog::UnsavedChanges => {
                let name = self.workspace.name().unwrap_or_default().to_owned();
                match unsaved_dialog::show(ctx, &name) {
                    Some(choice) => self.settle(ctx, choice),
                    None => DialogOutcome::Keep,
                }
            }
            Dialog::Recovery => match recovery_dialog::show(ctx, &self.workspace.recovery) {
                Some(choice) => self.recover(choice),
                None => DialogOutcome::Keep,
            },
        };
        match outcome {
            DialogOutcome::Keep => {}
            DialogOutcome::Close => self.active = None,
            DialogOutcome::Open(next) => self.active = Some(next),
        }
    }

    fn show_status(&mut self, ctx: &Context) {
        let Some(status) = &self.workspace.status else {
            return;
        };
        let limit = Duration::from_secs(if status.error {
            STATUS_SECONDS * 2
        } else {
            STATUS_SECONDS
        });
        if status.at.elapsed() > limit {
            self.workspace.status = None;
            return;
        }
        let level = if status.error {
            Level::Error
        } else {
            Level::Info
        };
        toast::show(ctx, level, &status.text, 16.0);
        ctx.request_repaint_after(Duration::from_millis(500));
    }

    /// Serves a pending dialog request from any screen.
    fn intake(&mut self, ctx: &Context) {
        if let Some(dialog) = commands::take_dialog_request(ctx) {
            self.active = Some(dialog);
            self.menu_anchor = None;
        }
    }

    fn show_menu(&mut self, ctx: &Context) {
        let Some(anchor) = self.menu_anchor else {
            return;
        };
        let items: Vec<menu::MenuItem> = MENU
            .iter()
            .map(|(glyph, key, shortcut, _)| menu::MenuItem {
                glyph: *glyph,
                label: tr(key),
                shortcut,
            })
            .collect();
        match menu::show(ctx, "app-menu", anchor, &items) {
            Some(Some(index)) => {
                self.active = Some(MENU[index].3);
                self.menu_anchor = None;
            }
            Some(None) => self.menu_anchor = None,
            None => {}
        }
    }
}

#[cfg(test)]
mod tests;
