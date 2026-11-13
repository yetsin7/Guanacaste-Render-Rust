//! Saved camera positions: nine numbered slots (`Ctrl+1..9` saves, `1..9` recalls).

use crate::features::viewport::models::Camera;

/// Number of slots.
pub const SLOT_COUNT: usize = 9;

/// Nine optional saved cameras.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct CameraSlots {
    slots: [Option<Camera>; SLOT_COUNT],
}

impl CameraSlots {
    /// Stores `camera` in `slot` (0-based). Returns false when the slot does not exist or the camera is invalid.
    pub fn save(&mut self, slot: usize, camera: Camera) -> bool {
        match self.slots.get_mut(slot) {
            Some(entry) if camera.is_valid() => {
                *entry = Some(camera);
                true
            }
            _ => false,
        }
    }

    /// The camera saved in `slot` (0-based), if any.
    pub fn recall(&self, slot: usize) -> Option<Camera> {
        self.slots.get(slot).copied().flatten()
    }

    /// Whether `slot` holds a camera.
    pub fn is_filled(&self, slot: usize) -> bool {
        self.recall(slot).is_some()
    }

    /// Empties every slot (a different scene was opened).
    pub fn clear(&mut self) {
        self.slots = Default::default();
    }
}
