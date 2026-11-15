//! The uniform block shared by the scene shaders (must match `Globals` in `shader.wgsl`).

use crate::features::viewport::logic::math::{self, Mat4, Vec3};
use crate::features::viewport::models::Camera;

/// Size of the uniform block in bytes.
pub const GLOBALS_SIZE: usize = 224;

/// Direction towards the sun used for the default lighting (unnormalized).
const SUN: Vec3 = [-0.45, -0.35, 0.82];
/// Sky color at the top of the view (gamma space).
const SKY_TOP: [f32; 4] = [0.16, 0.22, 0.33, 1.0];
/// Sky color at the horizon, also the color distant geometry fades into.
const HORIZON: [f32; 4] = [0.66, 0.73, 0.80, 1.0];
/// Color of the far ground below the horizon (grass seen through haze).
const GROUND_FAR: [f32; 4] = [0.44, 0.60, 0.33, 1.0];
/// Geometry starts to fade into the haze at this fraction of the fade-out distance.
const FOG_START_FRACTION: f32 = 0.45;
/// The haze never hides the scene closer than this (meters), so a small model keeps its crisp colors.
const MIN_FOG_END: f32 = 160.0;

/// Per-frame values the shaders need.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Globals {
    /// Projection times view.
    pub view_proj: Mat4,
    /// Eye position.
    pub eye: Vec3,
    /// Unit vector towards the sun.
    pub light_dir: Vec3,
    /// Distance at which grid lines have faded out completely.
    pub grid_fade: f32,
    /// Camera basis: right, up and forward unit vectors (the sky shader rebuilds each pixel's ray from them).
    pub basis: [Vec3; 3],
    /// Tangent of half the vertical field of view, and the width/height ratio.
    pub lens: [f32; 2],
    /// Distances where the haze starts and where geometry is fully lost in it.
    pub fog: [f32; 2],
}

impl Globals {
    /// Values for `camera` seen through a view with the given width/height ratio. `scene_radius` sets how far the
    /// grid stays visible and where the haze starts.
    pub fn new(camera: &Camera, aspect: f32, scene_radius: f32) -> Self {
        let fog_end = (scene_radius * 3.0).max(MIN_FOG_END);
        Self {
            view_proj: camera.view_projection(aspect),
            eye: camera.position,
            light_dir: math::normalize_or(SUN, math::Z),
            grid_fade: (scene_radius * 4.0).max(40.0),
            basis: [camera.right(), camera.up(), camera.forward()],
            lens: [(camera.fov_y_deg.to_radians() * 0.5).tan(), aspect],
            fog: [fog_end * FOG_START_FRACTION, fog_end],
        }
    }

    /// The uniform block as bytes in WGSL layout.
    pub fn to_bytes(&self) -> [u8; GLOBALS_SIZE] {
        let mut floats = Vec::with_capacity(GLOBALS_SIZE / 4);
        for column in &self.view_proj {
            floats.extend_from_slice(column);
        }
        floats.extend_from_slice(&[self.eye[0], self.eye[1], self.eye[2], 1.0]);
        floats.extend_from_slice(&[self.light_dir[0], self.light_dir[1], self.light_dir[2], 0.0]);
        floats.extend_from_slice(&SKY_TOP);
        floats.extend_from_slice(&HORIZON);
        floats.extend_from_slice(&[self.grid_fade, 0.0, 0.0, 0.0]);
        for axis in self.basis {
            floats.extend_from_slice(&[axis[0], axis[1], axis[2], 0.0]);
        }
        floats.extend_from_slice(&[self.lens[0], self.lens[1], self.fog[0], self.fog[1]]);
        floats.extend_from_slice(&GROUND_FAR);
        let mut bytes = [0u8; GLOBALS_SIZE];
        for (chunk, value) in bytes.chunks_exact_mut(4).zip(floats) {
            chunk.copy_from_slice(&value.to_ne_bytes());
        }
        bytes
    }

    /// Clear color of the offscreen target (hidden by the sky pass).
    pub fn clear_color() -> [f64; 4] {
        HORIZON.map(f64::from)
    }
}
