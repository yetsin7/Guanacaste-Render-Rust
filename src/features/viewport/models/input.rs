//! One frame of navigation input, already stripped of any UI toolkit types so the controllers stay pure.

/// Mouse buttons held during the frame (only those that started inside the 3D view).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct MouseButtons {
    /// Primary button.
    pub left: bool,
    /// Wheel button.
    pub middle: bool,
    /// Secondary button.
    pub right: bool,
}

/// Modifier keys held during the frame.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct KeyModifiers {
    /// Shift: faster flight, pan with the middle button.
    pub shift: bool,
    /// Ctrl: slower flight (only while the right button is held).
    pub ctrl: bool,
    /// Alt: orbit with the left button, dolly with the right button.
    pub alt: bool,
}

/// Movement keys held during the frame.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct MoveKeys {
    /// `W`: forward.
    pub forward: bool,
    /// `S`: backward.
    pub back: bool,
    /// `A`: left.
    pub left: bool,
    /// `D`: right.
    pub right: bool,
    /// `E`: up.
    pub up: bool,
    /// `Q`: down.
    pub down: bool,
}

impl MoveKeys {
    /// Whether any movement key is held.
    pub fn any(&self) -> bool {
        self.forward || self.back || self.left || self.right || self.up || self.down
    }
}

/// Everything the navigation controller needs from one frame.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct NavInput {
    /// Seconds since the previous frame.
    pub dt: f32,
    /// Size of the 3D view in points (used for pixel-accurate panning).
    pub view_size: [f32; 2],
    /// Pointer movement in points since the previous frame.
    pub pointer_delta: [f32; 2],
    /// Wheel movement in notches (positive = away from the user = zoom in).
    pub scroll: f32,
    /// Held mouse buttons.
    pub buttons: MouseButtons,
    /// Held modifier keys.
    pub modifiers: KeyModifiers,
    /// Held movement keys.
    pub keys: MoveKeys,
}
