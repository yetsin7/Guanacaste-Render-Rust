//! Camera vocabulary shared by the 3D view, photo mode and movie mode: a camera pose and the saved views of
//! the photo strip. The 3D view owns how a pose is driven; these are plain values.

/// Where the camera is and where it looks.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CameraPose {
    /// Eye position in metres (X east, Y up, Z south).
    pub position: [f32; 3],
    /// Heading in degrees, 0 looking north, clockwise.
    pub yaw: f32,
    /// Elevation in degrees, positive looking up.
    pub pitch: f32,
    /// Vertical field of view in degrees.
    pub fov: f32,
}

impl Default for CameraPose {
    fn default() -> Self {
        Self {
            position: [0.0, 1.7, 12.0],
            yaw: 0.0,
            pitch: -8.0,
            fov: 60.0,
        }
    }
}

/// A view kept in the photo strip.
#[derive(Debug, Clone, PartialEq)]
pub struct SavedCamera {
    /// Name shown under the thumbnail.
    pub name: String,
    /// The stored pose.
    pub pose: CameraPose,
}

/// Most views the photo strip holds.
pub const MAX_SAVED_CAMERAS: usize = 12;

/// The saved views and the one last recalled.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct CameraStrip {
    /// Saved views, in strip order.
    pub saved: Vec<SavedCamera>,
    /// Index of the view last saved or recalled.
    pub selected: Option<usize>,
}

impl CameraStrip {
    /// Whether another view fits.
    pub fn has_room(&self) -> bool {
        self.saved.len() < MAX_SAVED_CAMERAS
    }

    /// Stores `pose` as a new view named `name` and selects it; returns its index, or `None` when full.
    pub fn save(&mut self, name: String, pose: CameraPose) -> Option<usize> {
        if !self.has_room() {
            return None;
        }
        self.saved.push(SavedCamera { name, pose });
        let index = self.saved.len() - 1;
        self.selected = Some(index);
        Some(index)
    }

    /// The pose of view `index`, selecting it.
    pub fn recall(&mut self, index: usize) -> Option<CameraPose> {
        let pose = self.saved.get(index)?.pose;
        self.selected = Some(index);
        Some(pose)
    }

    /// Removes view `index`, keeping the selection on the same view when it survives.
    pub fn remove(&mut self, index: usize) {
        if index >= self.saved.len() {
            return;
        }
        self.saved.remove(index);
        self.selected = match self.selected {
            Some(selected) if selected == index => None,
            Some(selected) if selected > index => Some(selected - 1),
            other => other,
        };
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn save_recall_and_remove_keep_the_selection_consistent() {
        let mut strip = CameraStrip::default();
        let far = CameraPose {
            position: [5.0, 2.0, 30.0],
            ..CameraPose::default()
        };
        assert_eq!(strip.save("A".into(), CameraPose::default()), Some(0));
        assert_eq!(strip.save("B".into(), far), Some(1));
        assert_eq!(strip.recall(1), Some(far));
        strip.remove(0);
        assert_eq!(strip.selected, Some(0));
        assert_eq!(strip.saved[0].name, "B");
        strip.remove(0);
        assert_eq!(strip.selected, None);
        assert_eq!(strip.recall(3), None);
    }

    #[test]
    fn the_strip_has_a_limit() {
        let mut strip = CameraStrip::default();
        for index in 0..MAX_SAVED_CAMERAS {
            assert!(strip
                .save(index.to_string(), CameraPose::default())
                .is_some());
        }
        assert!(!strip.has_room());
        assert_eq!(strip.save("extra".into(), CameraPose::default()), None);
    }
}
