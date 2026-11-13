//! Navigation preferences and view vocabulary.

/// Tunable behavior of the navigation controller.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct NavSettings {
    /// Fly speed in meters per second.
    pub move_speed: f32,
    /// Slowest allowed fly speed.
    pub min_speed: f32,
    /// Fastest allowed fly speed.
    pub max_speed: f32,
    /// Speed multiplier while Shift is held.
    pub fast_factor: f32,
    /// Speed multiplier while Ctrl is held.
    pub slow_factor: f32,
    /// Speed change per wheel notch while the right button is held (multiplier).
    pub wheel_speed_step: f32,
    /// Degrees of rotation per pixel of mouse movement.
    pub look_sensitivity: f32,
    /// Zoom per wheel notch as a fraction of the distance to the target.
    pub zoom_step: f32,
    /// Smooth start/stop of the movement.
    pub inertia: bool,
    /// Time constant of the smoothing in seconds.
    pub smoothing_secs: f32,
    /// Invert the vertical look direction.
    pub invert_y: bool,
    /// Duration in seconds of animated moves (frame, presets, saved views).
    pub transition_secs: f32,
}

impl Default for NavSettings {
    fn default() -> Self {
        Self {
            move_speed: 5.0,
            min_speed: 0.05,
            max_speed: 2000.0,
            fast_factor: 4.0,
            slow_factor: 0.25,
            wheel_speed_step: 1.2,
            look_sensitivity: 0.15,
            zoom_step: 0.12,
            inertia: true,
            smoothing_secs: 0.09,
            invert_y: false,
            transition_secs: 0.45,
        }
    }
}

/// Predefined viewing directions.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ViewPreset {
    /// From above, north up.
    Top,
    /// From the south (-Y) looking north.
    Front,
    /// From the north (+Y) looking south.
    Back,
    /// From the west (-X) looking east.
    Left,
    /// From the east (+X) looking west.
    Right,
    /// From the south-west and above.
    Isometric,
}

impl ViewPreset {
    /// Every preset in display order.
    pub const ALL: [ViewPreset; 6] = [
        ViewPreset::Isometric,
        ViewPreset::Top,
        ViewPreset::Front,
        ViewPreset::Back,
        ViewPreset::Left,
        ViewPreset::Right,
    ];

    /// View direction `(yaw, pitch)` in degrees (see `Camera::yaw_pitch`).
    pub fn direction_deg(self) -> (f32, f32) {
        match self {
            ViewPreset::Top => (90.0, -89.0),
            ViewPreset::Front => (90.0, 0.0),
            ViewPreset::Back => (-90.0, 0.0),
            ViewPreset::Left => (0.0, 0.0),
            ViewPreset::Right => (180.0, 0.0),
            ViewPreset::Isometric => (45.0, -30.0),
        }
    }
}

/// Which helpers the 3D view draws.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ViewOptions {
    /// Ground grid.
    pub show_grid: bool,
    /// Coordinate axes.
    pub show_axes: bool,
}

impl Default for ViewOptions {
    fn default() -> Self {
        Self {
            show_grid: true,
            show_axes: true,
        }
    }
}
