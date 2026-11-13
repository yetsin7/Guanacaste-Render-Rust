//! Workspace keyboard shortcuts: `F1`-`F4` switch mode. `1`-`9` recall camera slots, `F` frames the view and
//! `Space` plays the movie; those belong to the 3D view and the movie mode. The dialog shortcuts
//! (`Ctrl+N/O/I/S/E`) live in the dialog host. Nothing fires while a dialog is open or a text field has the
//! keyboard.

use crate::models::navigation::{Mode, Screen};
use crate::models::AppState;
use eframe::egui::{Context, Key, Modifiers};

const MODE_KEYS: [Key; 4] = [Key::F1, Key::F2, Key::F3, Key::F4];

/// Reads this frame's keys and records what they ask for in `app`.
pub fn handle(ctx: &Context, app: &mut AppState, screen: Screen, dialog_open: bool) {
    if dialog_open || ctx.egui_wants_keyboard_input() || screen == Screen::Home {
        return;
    }
    for (index, key) in MODE_KEYS.into_iter().enumerate() {
        if ctx.input_mut(|input| input.consume_key(Modifiers::NONE, key)) {
            app.navigate(Screen::Workspace(Mode::ALL[index]));
        }
    }
}
