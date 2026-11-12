//! Movie studio entities and UI state.

/// Length of the sequence in seconds.
pub const DURATION: f32 = 14.0;
/// Frames per second of the sequence.
pub const FPS: u32 = 30;

/// One row of the timeline.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TrackKind {
    /// Camera orbit keyframes.
    Camera,
    /// Height curve.
    Height,
    /// Focal length clip.
    Focal,
    /// Sun and dusk pass.
    Sun,
}

/// UI state of the movie studio.
#[derive(Debug, Clone, PartialEq)]
pub struct MovieState {
    /// Playhead position in seconds.
    pub time: f32,
    /// Playback running.
    pub playing: bool,
    /// Loop toggle.
    pub looping: bool,
    /// Focal length, normalized over 18..85 mm.
    pub focal: f32,
    /// Selected aperture index.
    pub aperture: usize,
    /// Motion blur, normalized.
    pub motion_blur: f32,
    /// Depth-of-field switch.
    pub depth_of_field: bool,
    /// Atmosphere passes `[bloom, anamorphic]`.
    pub atmosphere: [bool; 2],
    /// Per-track toggle (lock for camera and sun, visibility for the others).
    pub track_flags: [bool; 4],
    /// Number of keyframes added by the user.
    pub added_keyframes: u32,
    /// Auto-spline switch.
    pub auto_spline: bool,
    /// Search text.
    pub search: String,
}

impl Default for MovieState {
    fn default() -> Self {
        Self {
            time: 4.4,
            playing: false,
            looping: false,
            focal: (28.0 - 18.0) / (85.0 - 18.0),
            aperture: 1,
            motion_blur: 0.65,
            depth_of_field: true,
            atmosphere: [false, false],
            track_flags: [true, true, true, true],
            added_keyframes: 0,
            auto_spline: false,
            search: String::new(),
        }
    }
}
