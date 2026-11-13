//! Workspace keyboard shortcuts: `1`-`4` switch mode, `F` frames the view, `Space` plays the movie. The
//! dialog shortcuts (`Ctrl+N/O/S/E/I`) live in the dialog host. Nothing fires while a dialog is open or a text
//! field has the keyboard.

use crate::models::navigation::{Mode, Screen};
use crate::models::AppState;
use eframe::egui::{Context, Key, Modifiers};

const MODE_KEYS: [Key; 4] = [Key::Num1, Key::Num2, Key::Num3, Key::Num4];

/// Reads this frame's keys and records what they ask for in `app`.
pub fn handle(ctx: &Context, app: &mut AppState, screen: Screen, dialog_open: bool) {
    if dialog_open || ctx.egui_wants_keyboard_input() || screen == Screen::Home {
        return;
    }
    let pressed = |key: Key| ctx.input_mut(|input| input.consume_key(Modifiers::NONE, key));
    for (index, key) in MODE_KEYS.into_iter().enumerate() {
        if pressed(key) {
            app.navigate(Screen::Workspace(Mode::ALL[index]));
        }
    }
    if let Screen::Workspace(mode) = screen {
        if pressed(Key::F) {
            app.requests.frame_view = true;
        }
        if mode == Mode::Movie && pressed(Key::Space) {
            app.movie.playing = !app.movie.playing;
        }
    }
}
