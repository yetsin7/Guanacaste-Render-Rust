//! A scene waiting to be sent to the GPU, shared between the UI state and the paint callback.

use crate::features::viewport::logic::scene_buffers::SceneBuffers;
use std::sync::Mutex;

/// CPU-side buffers of one scene generation. The GPU takes them once; afterwards the memory is released.
pub struct SceneUpload {
    /// Increases with every new scene; the GPU compares it with the generation it already holds.
    pub generation: u64,
    buffers: Mutex<Option<SceneBuffers>>,
}

impl SceneUpload {
    /// Wraps `buffers` as generation `generation`.
    pub fn new(generation: u64, buffers: SceneBuffers) -> Self {
        Self {
            generation,
            buffers: Mutex::new(Some(buffers)),
        }
    }

    /// Hands the buffers over to the GPU (only the first call returns them).
    pub fn take(&self) -> Option<SceneBuffers> {
        self.buffers.lock().ok()?.take()
    }
}
