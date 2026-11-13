//! The camera shared by the 3D view, the lens/filters, the render and the movie features.

use crate::features::viewport::logic::math::{self, Mat4, Vec3};

/// Largest pitch magnitude in degrees; keeps the view direction away from the vertical axis so `up` never degenerates.
pub const MAX_PITCH_DEG: f32 = 89.0;

/// A perspective camera in scene space: meters, right-handed, **Z up**. The camera has no roll: its up vector is
/// always +Z.
///
/// The camera looks from `position` towards `target`. When flying, `target` travels with `position`; when orbiting,
/// `target` is the pivot.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Camera {
    /// Eye position in meters.
    pub position: Vec3,
    /// Point the camera looks at, in meters. Must differ from `position`.
    pub target: Vec3,
    /// Vertical field of view in degrees (1..=170).
    pub fov_y_deg: f32,
    /// Near clip distance in meters (> 0).
    pub near: f32,
    /// Far clip distance in meters (> `near`).
    pub far: f32,
}

impl Default for Camera {
    /// An isometric-style view of a 20 m neighbourhood of the origin.
    fn default() -> Self {
        Self {
            position: [-12.0, -12.0, 9.0],
            target: [0.0, 0.0, 1.5],
            fov_y_deg: 50.0,
            near: 0.05,
            far: 2000.0,
        }
    }
}

impl Camera {
    /// A camera with the default lens looking from `position` at `target`.
    pub fn looking_at(position: Vec3, target: Vec3) -> Self {
        Self {
            position,
            target,
            ..Self::default()
        }
    }

    /// Unit vector from the eye towards the target (+Y if both coincide).
    pub fn forward(&self) -> Vec3 {
        math::normalize_or(math::sub(self.target, self.position), [0.0, 1.0, 0.0])
    }

    /// Unit vector to the right of the view, always horizontal.
    pub fn right(&self) -> Vec3 {
        math::normalize_or(math::cross(self.forward(), math::Z), [1.0, 0.0, 0.0])
    }

    /// Unit vector pointing up on screen.
    pub fn up(&self) -> Vec3 {
        math::cross(self.right(), self.forward())
    }

    /// Distance between the eye and the target.
    pub fn distance(&self) -> f32 {
        math::length(math::sub(self.target, self.position))
    }

    /// View direction as `(yaw, pitch)` in radians. Yaw is measured from +X towards +Y; pitch is positive looking up.
    pub fn yaw_pitch(&self) -> (f32, f32) {
        let f = self.forward();
        (f[1].atan2(f[0]), f[2].clamp(-1.0, 1.0).asin())
    }

    /// Unit view direction for a yaw and pitch in radians (the inverse of [`Camera::yaw_pitch`]).
    pub fn direction(yaw: f32, pitch: f32) -> Vec3 {
        [
            pitch.cos() * yaw.cos(),
            pitch.cos() * yaw.sin(),
            pitch.sin(),
        ]
    }

    /// Whether every number is finite and the lens is valid.
    pub fn is_valid(&self) -> bool {
        let finite = self
            .position
            .iter()
            .chain(self.target.iter())
            .all(|v| v.is_finite());
        finite
            && self.fov_y_deg.is_finite()
            && (1.0..=170.0).contains(&self.fov_y_deg)
            && self.near > 0.0
            && self.far > self.near
            && self.distance() > 1e-6
    }

    /// World-to-view matrix (column-major).
    pub fn view_matrix(&self) -> Mat4 {
        math::look_at(self.position, self.target, math::Z)
    }

    /// Perspective projection with depth in `0..1` (column-major) for the given width/height ratio.
    pub fn projection(&self, aspect: f32) -> Mat4 {
        math::perspective(self.fov_y_deg.to_radians(), aspect, self.near, self.far)
    }

    /// Projection times view.
    pub fn view_projection(&self, aspect: f32) -> Mat4 {
        math::mul(&self.projection(aspect), &self.view_matrix())
    }
}
