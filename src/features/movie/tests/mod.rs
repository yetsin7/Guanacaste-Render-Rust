//! Unit tests of the movie domain: interpolation, continuity, editing, playback and persistence.

mod continuity_tests;
mod editing_tests;
mod interpolation_tests;
mod playback_tests;
mod serde_tests;

use super::models::{Animation, CameraKey, CameraPose, Ease, PathMode, Timing};

/// A pose at `(x, y, z)` looking `yaw` degrees around the vertical axis.
pub(super) fn pose(x: f64, y: f64, z: f64, yaw: f64) -> CameraPose {
    CameraPose::from_euler([x, y, z], yaw, 0.0, 0.0, 60.0)
}

/// An animation whose camera keys are `(time, pose)` pairs, with the given path mode.
pub(super) fn animation(keys: &[(f64, CameraPose)], path: PathMode) -> Animation {
    let mut animation = Animation::default();
    animation.camera.path = path;
    animation.camera.timing = Timing::Keyed;
    animation.camera.keys = keys
        .iter()
        .map(|(time, pose)| CameraKey {
            time: *time,
            pose: *pose,
            ease: Ease::Linear,
            pause: 0.0,
        })
        .collect();
    animation
}

/// Whether two numbers differ by less than `tolerance`.
pub(super) fn near(a: f64, b: f64, tolerance: f64) -> bool {
    (a - b).abs() < tolerance
}
