//! Cameras that frame a scene automatically: the "saved views" every project offers before the user makes its own.

use super::types::RenderCamera;
use super::vec3::Vec3;
use crate::features::import_export::models::Bounds;

/// A ready-made viewpoint around the scene bounds.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ViewPreset {
    /// From a front corner and above, like an architectural perspective.
    #[default]
    Corner,
    /// Straight at the south (-Y) face.
    Front,
    /// Straight at the east (+X) face.
    Side,
    /// Looking down from above.
    Plan,
}

impl ViewPreset {
    /// Presets in display order.
    pub const ALL: [ViewPreset; 4] = [
        ViewPreset::Corner,
        ViewPreset::Front,
        ViewPreset::Side,
        ViewPreset::Plan,
    ];

    /// Localization key of the preset name.
    pub fn label_key(self) -> &'static str {
        match self {
            ViewPreset::Corner => "render.view_corner",
            ViewPreset::Front => "render.view_front",
            ViewPreset::Side => "render.view_side",
            ViewPreset::Plan => "render.view_plan",
        }
    }

    /// Unit vector from the scene center toward the camera, and the up hint.
    fn placement(self) -> (Vec3, Vec3) {
        match self {
            ViewPreset::Corner => (
                Vec3::new(0.7, -1.0, 0.65).normalized(),
                Vec3::new(0.0, 0.0, 1.0),
            ),
            ViewPreset::Front => (
                Vec3::new(0.0, -1.0, 0.12).normalized(),
                Vec3::new(0.0, 0.0, 1.0),
            ),
            ViewPreset::Side => (
                Vec3::new(1.0, 0.0, 0.12).normalized(),
                Vec3::new(0.0, 0.0, 1.0),
            ),
            ViewPreset::Plan => (Vec3::new(0.0, 0.0, 1.0), Vec3::new(0.0, 1.0, 0.0)),
        }
    }
}

/// A camera of `width` x `height` pixels that sees all of `bounds` from the given preset.
pub fn frame_bounds(bounds: &Bounds, preset: ViewPreset, width: u32, height: u32) -> RenderCamera {
    let min = Vec3::from_array(bounds.min);
    let max = Vec3::from_array(bounds.max);
    let center = (min + max) * 0.5;
    let radius = ((max - min).length() * 0.5).max(0.5);
    let fov_y: f32 = 40.0;
    let aspect = width.max(1) as f32 / height.max(1) as f32;
    let half_y = (fov_y.to_radians() * 0.5).tan();
    let half = half_y.min(half_y * aspect);
    // The bounding sphere fits the narrower of the two field angles.
    let distance = radius / half.atan().sin() * 1.05;
    let (toward, up) = preset.placement();
    let position = center + toward * distance;
    RenderCamera {
        position: [position.x, position.y, position.z],
        direction: [-toward.x, -toward.y, -toward.z],
        up: [up.x, up.y, up.z],
        fov_y_degrees: fov_y,
        width,
        height,
        near: (distance * 0.01).max(0.01),
        far: distance + radius * 4.0,
    }
}
