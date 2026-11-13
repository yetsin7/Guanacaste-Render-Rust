//! Reads egui's input for the 3D view and turns it into the toolkit-free [`NavInput`] plus discrete actions.

use crate::features::viewport::models::{KeyModifiers, MouseButtons, MoveKeys, NavInput};
use eframe::egui::{Context, Id, Key, PointerButton, Rect, Response};

/// Points the wheel moves per notch (egui's default line scroll speed).
const POINTS_PER_NOTCH: f32 = 40.0;

/// A one-shot request raised by a key press.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ViewAction {
    /// Frame the whole model (`F` or `Home`).
    Frame,
    /// Save the camera in slot `n` (`Ctrl+1..9`, 0-based).
    SaveSlot(usize),
    /// Go to the camera saved in slot `n` (`1..9`, 0-based).
    RecallSlot(usize),
}

/// What the view read this frame.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct FrameInput {
    /// Continuous navigation input.
    pub nav: NavInput,
    /// One-shot requests.
    pub actions: Vec<ViewAction>,
    /// A mouse button is currently steering the camera.
    pub grabbed: bool,
}

const DIGITS: [Key; 9] = [
    Key::Num1,
    Key::Num2,
    Key::Num3,
    Key::Num4,
    Key::Num5,
    Key::Num6,
    Key::Num7,
    Key::Num8,
    Key::Num9,
];

fn grab_id() -> Id {
    Id::new("viewport-3d-grab")
}

/// Buttons that were pressed over the view and are still held (a drag that leaves the view keeps steering).
fn grabbed_buttons(ctx: &Context, response: &Response) -> MouseButtons {
    let mut grabbed: [bool; 3] = ctx
        .data(|data| data.get_temp(grab_id()))
        .unwrap_or_default();
    ctx.input(|input| {
        let buttons = [
            PointerButton::Primary,
            PointerButton::Middle,
            PointerButton::Secondary,
        ];
        for (slot, button) in grabbed.iter_mut().zip(buttons) {
            if input.pointer.button_pressed(button) {
                *slot = response.hovered();
            }
            if !input.pointer.button_down(button) {
                *slot = false;
            }
        }
    });
    ctx.data_mut(|data| data.insert_temp(grab_id(), grabbed));
    MouseButtons {
        left: grabbed[0],
        middle: grabbed[1],
        right: grabbed[2],
    }
}

/// Reads this frame's input for the view covering `rect`. Keys act while the pointer is over the view or a drag
/// that started in it is held, and never while a text field has the keyboard.
pub fn read(ctx: &Context, response: &Response, rect: Rect) -> FrameInput {
    let buttons = grabbed_buttons(ctx, response);
    let grabbed = buttons.left || buttons.middle || buttons.right;
    let active = (response.hovered() || grabbed) && !ctx.egui_wants_keyboard_input();
    let hovered = response.hovered() || grabbed;
    ctx.input(|input| {
        let down = |key: Key| active && input.key_down(key);
        let keys = MoveKeys {
            forward: down(Key::W),
            back: down(Key::S),
            left: down(Key::A),
            right: down(Key::D),
            up: down(Key::E),
            down: down(Key::Q),
        };
        let modifiers = KeyModifiers {
            shift: input.modifiers.shift,
            ctrl: input.modifiers.ctrl,
            alt: input.modifiers.alt,
        };
        let mut actions = Vec::new();
        if active && !modifiers.ctrl && !modifiers.alt {
            if input.key_pressed(Key::F) || input.key_pressed(Key::Home) {
                actions.push(ViewAction::Frame);
            }
            for (slot, key) in DIGITS.iter().enumerate() {
                if input.key_pressed(*key) {
                    actions.push(ViewAction::RecallSlot(slot));
                }
            }
        }
        if active && modifiers.ctrl && !modifiers.alt {
            for (slot, key) in DIGITS.iter().enumerate() {
                if input.key_pressed(*key) {
                    actions.push(ViewAction::SaveSlot(slot));
                }
            }
        }
        let scroll = if hovered {
            input.smooth_scroll_delta.y / POINTS_PER_NOTCH
        } else {
            0.0
        };
        let delta = input.pointer.delta();
        FrameInput {
            nav: NavInput {
                dt: input.unstable_dt,
                view_size: [rect.width(), rect.height()],
                pointer_delta: if grabbed {
                    [delta.x, delta.y]
                } else {
                    [0.0; 2]
                },
                scroll,
                buttons,
                modifiers,
                keys,
            },
            actions,
            grabbed,
        }
    })
}
