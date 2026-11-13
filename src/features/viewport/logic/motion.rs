//! Elementary camera motions: look, orbit, pan, dolly and translate. Each one edits a [`Camera`] in place and keeps
//! it valid (no roll, pitch limited).

use super::math::{self, Vec3};
use crate::features::viewport::models::{Camera, MAX_PITCH_DEG};

/// Closest the eye may get to the target while zooming, in meters.
pub const MIN_DISTANCE: f32 = 0.1;

/// The view direction after turning by a pointer movement of `(dx, dy)` pixels at `degrees_per_pixel`.
/// Dragging right turns right (yaw decreases); dragging down looks down (pitch decreases) unless `invert_y`.
fn turned(camera: &Camera, dx: f32, dy: f32, degrees_per_pixel: f32, invert_y: bool) -> Vec3 {
    let (yaw, pitch) = camera.yaw_pitch();
    let limit = MAX_PITCH_DEG.to_radians();
    let vertical = if invert_y { dy } else { -dy };
    let yaw = yaw - (dx * degrees_per_pixel).to_radians();
    let pitch = (pitch + (vertical * degrees_per_pixel).to_radians()).clamp(-limit, limit);
    Camera::direction(yaw, pitch)
}

/// Mouse look: rotates the view around the eye; the target keeps its distance.
pub fn look(camera: &mut Camera, dx: f32, dy: f32, degrees_per_pixel: f32, invert_y: bool) {
    let distance = camera.distance().max(MIN_DISTANCE);
    let direction = turned(camera, dx, dy, degrees_per_pixel, invert_y);
    camera.target = math::add(camera.position, math::scale(direction, distance));
}

/// Orbit: rotates the eye around the target.
pub fn orbit(camera: &mut Camera, dx: f32, dy: f32, degrees_per_pixel: f32, invert_y: bool) {
    let distance = camera.distance().max(MIN_DISTANCE);
    let direction = turned(camera, dx, dy, degrees_per_pixel, invert_y);
    camera.position = math::sub(camera.target, math::scale(direction, distance));
}

/// Moves eye and target together by `delta` meters.
pub fn translate(camera: &mut Camera, delta: Vec3) {
    camera.position = math::add(camera.position, delta);
    camera.target = math::add(camera.target, delta);
}

/// Pan: slides the view so the scene follows the pointer pixel for pixel at the target's depth.
pub fn pan(camera: &mut Camera, dx: f32, dy: f32, view_height: f32) {
    let height = view_height.max(1.0);
    let per_pixel = 2.0 * camera.distance() * (camera.fov_y_deg.to_radians() * 0.5).tan() / height;
    let right = math::scale(camera.right(), -dx * per_pixel);
    let up = math::scale(camera.up(), dy * per_pixel);
    translate(camera, math::add(right, up));
}

/// Zoom by `notches` (positive = closer): the eye approaches the target by `step` of the distance per notch. When
/// the minimum distance is reached the whole camera keeps moving forward by `push` meters per notch, so the user
/// can zoom through a scene instead of stopping at the pivot.
pub fn dolly(camera: &mut Camera, notches: f32, step: f32, push: f32) {
    let distance = camera.distance();
    let forward = camera.forward();
    let wanted = if notches >= 0.0 {
        distance * (1.0 - step).powf(notches)
    } else {
        distance / (1.0 - step).powf(-notches)
    };
    let new_distance = wanted.max(MIN_DISTANCE.min(distance));
    camera.position = math::sub(camera.target, math::scale(forward, new_distance));
    if notches > 0.0 && wanted < MIN_DISTANCE {
        translate(camera, math::scale(forward, push * notches));
    }
}
