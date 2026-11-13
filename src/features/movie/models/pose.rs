//! The camera pose the animation produces and the simple camera the studio records keyframes from.

use super::quat::{self, Quat};
use serde::{Deserialize, Serialize};

/// Width of the reference (full-frame) sensor in millimeters, used to convert focal length and field of view.
pub const SENSOR_WIDTH_MM: f64 = 36.0;
/// Smallest and largest field of view the animation may produce, in degrees.
pub const FOV_RANGE: (f64, f64) = (5.0, 170.0);

/// Position, orientation and horizontal field of view of a camera at one instant.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct CameraPose {
    /// Eye position in meters (X right, Y forward, Z up).
    pub position: [f64; 3],
    /// Orientation as a unit quaternion `[x, y, z, w]` (see [`quat`] for the axis convention).
    pub orientation: Quat,
    /// Horizontal field of view in degrees.
    pub fov_deg: f64,
}

impl Default for CameraPose {
    fn default() -> Self {
        Self {
            position: [0.0, -10.0, 1.6],
            orientation: quat::IDENTITY,
            fov_deg: 60.0,
        }
    }
}

impl CameraPose {
    /// Builds a pose from a position, yaw/pitch/roll in degrees and a field of view in degrees.
    pub fn from_euler(position: [f64; 3], yaw: f64, pitch: f64, roll: f64, fov_deg: f64) -> Self {
        Self {
            position,
            orientation: quat::from_euler_deg(yaw, pitch, roll),
            fov_deg,
        }
    }

    /// Yaw, pitch and roll in degrees.
    pub fn euler_deg(&self) -> [f64; 3] {
        quat::to_euler_deg(self.orientation)
    }

    /// Sets the orientation from yaw, pitch and roll in degrees.
    pub fn set_euler_deg(&mut self, yaw: f64, pitch: f64, roll: f64) {
        self.orientation = quat::from_euler_deg(yaw, pitch, roll);
    }

    /// Unit vector the camera looks along.
    pub fn forward(&self) -> [f64; 3] {
        quat::rotate(self.orientation, [0.0, 1.0, 0.0])
    }

    /// Equivalent focal length in millimeters on a full-frame sensor.
    pub fn focal_mm(&self) -> f64 {
        SENSOR_WIDTH_MM / 2.0 / (self.fov_deg.to_radians() / 2.0).tan()
    }

    /// Sets the field of view from a focal length in millimeters.
    pub fn set_focal_mm(&mut self, focal_mm: f64) {
        let fov = 2.0 * (SENSOR_WIDTH_MM / 2.0 / focal_mm.max(1.0)).atan();
        self.fov_deg = fov.to_degrees().clamp(FOV_RANGE.0, FOV_RANGE.1);
    }

    /// The pose with every value finite, a unit orientation and a field of view inside [`FOV_RANGE`].
    pub fn sanitized(mut self) -> Self {
        let fallback = Self::default();
        for (value, default) in self.position.iter_mut().zip(fallback.position) {
            if !value.is_finite() {
                *value = default;
            }
        }
        if self.orientation.iter().any(|v| !v.is_finite()) {
            self.orientation = quat::IDENTITY;
        }
        self.orientation = quat::normalize(self.orientation);
        self.fov_deg = if self.fov_deg.is_finite() {
            self.fov_deg.clamp(FOV_RANGE.0, FOV_RANGE.1)
        } else {
            fallback.fov_deg
        };
        self
    }
}
