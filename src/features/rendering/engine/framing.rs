//! Cameras that frame a scene automatically: the "saved views" every project offers before the user makes its own.
//! The distance is the smallest one that keeps all eight corners of the bounds inside the picture, so a long house
//! fills the frame instead of floating in a sphere of empty sky.

use super::types::RenderCamera;
use super::vec3::Vec3;
use crate::features::import_export::models::Bounds;

/// A ready-made viewpoint around the scene bounds.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ViewPreset {
    /// From a front corner and above, like an architectural perspective.
    #[default]
    Corner,
    /// Straight at the main facade (by default the south, -Y, face).
    Front,
    /// Straight at the side facade.
    Side,
    /// Looking down from above.
    Plan,
    /// A bird's-eye view from the other front corner.
    Aerial,
    /// Standing inside the roomiest room (found by the scene itself; around the bounds it falls back to the corner).
    Interior,
}

impl ViewPreset {
    /// Presets in display order. [`ViewPreset::Interior`] is not listed: finding a room builds the whole
    /// acceleration structure, which is too slow for the interface thread.
    pub const ALL: [ViewPreset; 5] = [
        ViewPreset::Corner,
        ViewPreset::Front,
        ViewPreset::Side,
        ViewPreset::Plan,
        ViewPreset::Aerial,
    ];

    /// Localization key of the preset name.
    pub fn label_key(self) -> &'static str {
        match self {
            ViewPreset::Corner => "render.view_corner",
            ViewPreset::Front => "render.view_front",
            ViewPreset::Side => "render.view_side",
            ViewPreset::Plan => "render.view_plan",
            ViewPreset::Aerial => "render.view_aerial",
            ViewPreset::Interior => "render.view_interior",
        }
    }

    /// Horizontal angle from the main facade (radians, counterclockwise seen from above) and elevation angle.
    fn angles(self) -> (f32, f32) {
        match self {
            ViewPreset::Corner => (0.62, 0.30),
            ViewPreset::Front => (0.0, 0.10),
            ViewPreset::Side => (std::f32::consts::FRAC_PI_2, 0.10),
            ViewPreset::Plan => (0.0, std::f32::consts::FRAC_PI_2),
            ViewPreset::Aerial => (-0.75, 0.62),
            ViewPreset::Interior => ViewPreset::Corner.angles(),
        }
    }
}

/// Facing of the south (-Y) facade as the direction from the building toward the camera.
pub const SOUTH: f32 = -std::f32::consts::FRAC_PI_2;

/// A camera of `width` x `height` pixels that sees all of `bounds` from the given preset, the main facade facing -Y.
pub fn frame_bounds(bounds: &Bounds, preset: ViewPreset, width: u32, height: u32) -> RenderCamera {
    frame_facing(bounds, SOUTH, preset, width, height)
}

/// Like [`frame_bounds`] with the main facade facing the horizontal direction `facade` (radians from +X).
pub fn frame_facing(
    bounds: &Bounds,
    facade: f32,
    preset: ViewPreset,
    width: u32,
    height: u32,
) -> RenderCamera {
    let min = Vec3::from_array(bounds.min);
    let max = Vec3::from_array(bounds.max);
    let center = (min + max) * 0.5;
    let half_size = (max - min) * 0.5;
    let fov_y: f32 = 40.0;
    let aspect = width.max(1) as f32 / height.max(1) as f32;
    let tan_y = (fov_y.to_radians() * 0.5).tan();
    let tan_x = tan_y * aspect;
    let (turn, elevation) = preset.angles();
    let azimuth = facade + turn;
    let toward = Vec3::new(
        elevation.cos() * azimuth.cos(),
        elevation.cos() * azimuth.sin(),
        elevation.sin(),
    );
    let up_hint = if preset == ViewPreset::Plan {
        // North up in the plan; the facade direction would roll the picture.
        Vec3::new(0.0, 1.0, 0.0)
    } else {
        Vec3::new(0.0, 0.0, 1.0)
    };
    let forward = -toward;
    let mut right = forward.cross(up_hint);
    if right.length() < 1e-4 {
        right = Vec3::new(1.0, 0.0, 0.0);
    }
    let right = right.normalized();
    let up = right.cross(forward).normalized();
    // Look a little below the middle: rooftops matter less than the walls and the garden in front.
    let target = center - Vec3::new(0.0, 0.0, half_size.z * 0.1);
    let mut distance = half_size.length() * 0.3;
    for corner in 0..8 {
        let offset = Vec3::new(
            if corner & 1 == 0 {
                -half_size.x
            } else {
                half_size.x
            },
            if corner & 2 == 0 {
                -half_size.y
            } else {
                half_size.y
            },
            if corner & 4 == 0 {
                -half_size.z
            } else {
                half_size.z
            },
        ) + (center - target);
        let nearer = offset.dot(toward);
        let fit_x = offset.dot(right).abs() / (tan_x * 0.92) + nearer;
        let fit_y = offset.dot(up).abs() / (tan_y * 0.92) + nearer;
        distance = distance.max(fit_x).max(fit_y);
    }
    let position = target + toward * distance;
    let radius = half_size.length();
    RenderCamera {
        position: [position.x, position.y, position.z],
        direction: [forward.x, forward.y, forward.z],
        up: [up_hint.x, up_hint.y, up_hint.z],
        fov_y_degrees: fov_y,
        width,
        height,
        near: (distance * 0.01).max(0.05),
        far: (distance + radius * 6.0).max(30_000.0),
        panoramic: false,
    }
}
