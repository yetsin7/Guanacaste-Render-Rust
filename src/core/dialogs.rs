//! Hosts every dialog, the application menu and the keyboard shortcuts that open them.

use crate::commands;
use crate::components::{icons, menu};
use crate::features::help::components::notifications_panel::{self, Notice};
use crate::features::help::components::{about_dialog, error_dialog, shortcuts_dialog};
use crate::features::import_export::ui::{import_dialog, state::ImportDialogState};
use crate::features::projects::components::{new_scene_dialog, open_dialog, save_dialog};
use crate::features::projects::models::{NewSceneState, OpenState, RecentFile, SaveState};
use crate::features::rendering::components::export_dialog;
use crate::features::rendering::models::ExportState;
use crate::features::settings::components::preferences_dialog;
use crate::models::dialog::{Dialog, DialogOutcome};
use crate::models::preferences::Preferences;
use crate::services::localization::tr;
use crate::services::sample_data::files;
use eframe::egui::{Context, Key, KeyboardShortcut, Modifiers, Pos2};

/// Keyboard shortcuts of the shell and the dialogs they open.
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
const MENU: [(char, &str, &str, Dialog); 8] = [
    (icons::UPLOAD, "menu.import", "Ctrl+I", Dialog::Import),
    (icons::ADD_BOX, "menu.new", "Ctrl+N", Dialog::NewScene),
    (
        icons::FOLDER_OPEN,
        "menu.open",
        "Ctrl+O",
        Dialog::OpenProject,
    ),
    (icons::SAVE, "menu.save", "Ctrl+S", Dialog::SaveProject),
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

/// State of all dialogs plus the one currently shown.
pub struct DialogHost {
    active: Option<Dialog>,
    menu_anchor: Option<Pos2>,
    open: OpenState,
    save: SaveState,
    new_scene: NewSceneState,
    export: ExportState,
    preferences_tab: usize,
    shortcuts_search: String,
    notices: Vec<Notice>,
    recent: Vec<RecentFile>,
    import: ImportDialogState,
}

impl Default for DialogHost {
    fn default() -> Self {
        Self {
            active: None,
            menu_anchor: None,
            open: OpenState::default(),
            save: SaveState::default(),
            new_scene: NewSceneState::default(),
            export: ExportState::default(),
            preferences_tab: 0,
            shortcuts_search: String::new(),
            notices: notifications_panel::initial_notices(),
            recent: files::recent_files(),
            import: ImportDialogState::default(),
        }
    }
}

impl DialogHost {
    /// Opens the application menu with its top-right corner at `anchor`.
    pub fn open_menu(&mut self, anchor: Pos2) {
        self.menu_anchor = Some(anchor);
    }

    /// The dialog currently shown, if any.
    pub fn active(&self) -> Option<Dialog> {
        self.active
    }

    /// Serves requests and shortcuts, then draws the menu and the active dialog.
    pub fn show(&mut self, ctx: &Context, preferences: &mut Preferences) {
        self.intake(ctx);
        if self.active.is_none() && !ctx.egui_wants_keyboard_input() {
            for (key, dialog) in SHORTCUTS {
                if ctx.input_mut(|input| {
                    input.consume_shortcut(&KeyboardShortcut::new(Modifiers::COMMAND, key))
                }) {
                    self.active = Some(dialog);
                }
            }
        }
        self.show_menu(ctx);
        let Some(dialog) = self.active else { return };
        let outcome = match dialog {
            Dialog::NewScene => new_scene_dialog::show(ctx, &mut self.new_scene),
            Dialog::OpenProject => open_dialog::show(ctx, &mut self.open, &self.recent),
            Dialog::Import => import_dialog::show(ctx, &mut self.import),
            Dialog::SaveProject => save_dialog::show(ctx, &mut self.save),
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
        };
        match outcome {
            DialogOutcome::Keep => {}
            DialogOutcome::Close => self.active = None,
            DialogOutcome::Open(next) => self.active = Some(next),
        }
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
mod tests {
    use super::*;

    #[test]
    fn every_menu_entry_has_a_dialog() {
        let dialogs: Vec<Dialog> = MENU.iter().map(|entry| entry.3).collect();
        assert!(dialogs.contains(&Dialog::OpenProject));
        assert!(dialogs.contains(&Dialog::About));
    }

    #[test]
    fn shortcuts_open_distinct_dialogs() {
        let mut dialogs: Vec<String> = SHORTCUTS
            .iter()
            .map(|entry| format!("{:?}", entry.1))
            .collect();
        dialogs.sort();
        dialogs.dedup();
        assert_eq!(dialogs.len(), SHORTCUTS.len());
    }

    #[test]
    fn a_request_opens_its_dialog() {
        let ctx = Context::default();
        let mut host = DialogHost::default();
        commands::request_dialog(&ctx, Dialog::About);
        host.intake(&ctx);
        assert_eq!(host.active(), Some(Dialog::About));
    }
}
