//! Movie studio entities: the animation data (serialized in the project) and the UI state.

mod animation;
mod pose;
pub mod quat;

pub use animation::*;
pub use pose::*;

/// The element selected in the timeline or in the path view.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Selection {
    /// Nothing selected.
    #[default]
    None,
    /// A camera key (index into `animation.camera.keys`).
    Camera(usize),
    /// A key of a channel.
    Value {
        /// Index into `animation.channels`.
        channel: usize,
        /// Index into the channel's keys.
        key: usize,
    },
}

/// Fixed scale and center of the plan view, kept while a key is dragged so the view does not move under the
/// pointer.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PlanLock {
    /// World X and Y (meters) at the center of the view.
    pub center: [f64; 2],
    /// Pixels per meter.
    pub scale: f64,
}

/// UI state of the movie studio (the animation itself lives in the project).
#[derive(Debug, Clone, PartialEq)]
pub struct MovieState {
    /// Playhead position in animation seconds.
    pub time: f64,
    /// Playback running.
    pub playing: bool,
    /// The studio camera: what "record camera" stores as a key. The 3D view will keep it up to date.
    pub live_camera: CameraPose,
    /// Selected element.
    pub selection: Selection,
    /// View lock while dragging in the path view.
    pub plan_lock: Option<PlanLock>,
    /// Name typed for a new channel.
    pub new_channel: String,
}

impl Default for MovieState {
    fn default() -> Self {
        Self {
            time: 0.0,
            playing: false,
            live_camera: CameraPose::default(),
            selection: Selection::None,
            plan_lock: None,
            new_channel: String::new(),
        }
    }
}
