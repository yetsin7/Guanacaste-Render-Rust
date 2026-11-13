//! egui side of the 3D view: input reading, the interactive panel and the shared view state.

pub mod input;
pub mod panel;
pub mod state;

pub use state::{SceneChange, SceneSource, ViewportState};
