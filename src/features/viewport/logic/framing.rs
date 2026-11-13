//! Framing a model: where to put the camera so the whole scene fits, and clip planes that suit its size.

use super::math::{self, Vec3};
use crate::features::import_export::models::Bounds;
use crate::features::viewport::models::{Camera, ViewPreset, MAX_PITCH_DEG};

/// Radius used for an empty scene, in meters.
pub const EMPTY_SCENE_RADIUS: f32 = 10.0;
/// Smallest radius ever used for framing, in meters.
pub const MIN_RADIUS: f32 = 0.5;

/// Center of `bounds`.
pub fn center(bounds: &Bounds) -> Vec3 {
    math::scale(math::add(bounds.min, bounds.max), 0.5)
}

/// Radius of the sphere around the center of `bounds` that contains the box (at least [`MIN_RADIUS`]).
pub fn radius(bounds: &Bounds) -> f32 {
    (math::length(math::sub(bounds.max, bounds.min)) * 0.5).max(MIN_RADIUS)
}

/// Distance from the center at which a sphere of `radius` fits entirely in a view of `fov_y_deg` and `aspect`.
pub fn fit_distance(radius: f32, fov_y_deg: f32, aspect: f32) -> f32 {
    let half_v = (fov_y_deg.to_radians() * 0.5).clamp(0.01, 1.5);
    let aspect = if aspect.is_finite() && aspect > 0.05 {
        aspect
    } else {
        1.0
    };
    let half_h = (half_v.tan() * aspect).atan();
    radius / half_v.min(half_h).sin() * 1.08
}

/// Near and far clip distances suited to a scene of the given `radius`.
pub fn clip_planes(radius: f32) -> (f32, f32) {
    let near = (radius * 0.001).clamp(0.05, 1.0);
    let far = (radius * 60.0).max(1000.0);
    (near, far)
}

/// A camera that sees all of `bounds` (or a default neighbourhood of the origin when `None`) from the direction
/// `yaw`/`pitch` (radians), keeping the lens of `lens`.
pub fn frame(bounds: Option<&Bounds>, yaw: f32, pitch: f32, aspect: f32, lens: &Camera) -> Camera {
    let (target, radius) = match bounds {
        Some(b) => (center(b), radius(b)),
        None => ([0.0, 0.0, 1.5], EMPTY_SCENE_RADIUS),
    };
    let limit = MAX_PITCH_DEG.to_radians();
    let direction = Camera::direction(yaw, pitch.clamp(-limit, limit));
    let distance = fit_distance(radius, lens.fov_y_deg, aspect);
    let (near, far) = clip_planes(radius + distance);
    Camera {
        position: math::sub(target, math::scale(direction, distance)),
        target,
        fov_y_deg: lens.fov_y_deg,
        near,
        far,
    }
}

/// Like [`frame`] but looking along one of the predefined [`ViewPreset`] directions.
pub fn frame_preset(
    bounds: Option<&Bounds>,
    preset: ViewPreset,
    aspect: f32,
    lens: &Camera,
) -> Camera {
    let (yaw, pitch) = preset.direction_deg();
    frame(bounds, yaw.to_radians(), pitch.to_radians(), aspect, lens)
}
