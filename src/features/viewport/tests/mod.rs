//! Unit tests of the 3D view: math, camera, navigation, framing, scene buffers and view state.

mod buffers_tests;
mod camera_tests;
mod controller_tests;
mod framing_tests;
mod state_tests;

use crate::features::import_export::models::{Bounds, Mesh};
use crate::features::viewport::logic::math::Vec3;

/// A unit-size axis-aligned cube centered at `center` with `size` edge length, as a 12-triangle mesh.
pub(crate) fn cube(center: Vec3, size: f32, material: Option<usize>) -> Mesh {
    let h = size / 2.0;
    let mut positions = Vec::new();
    for corner in 0..8u32 {
        positions.push([
            center[0] + if corner & 1 == 0 { -h } else { h },
            center[1] + if corner & 2 == 0 { -h } else { h },
            center[2] + if corner & 4 == 0 { -h } else { h },
        ]);
    }
    let indices = vec![
        0, 2, 1, 1, 2, 3, 4, 5, 6, 5, 7, 6, 0, 1, 4, 1, 5, 4, 2, 6, 3, 3, 6, 7, 0, 4, 2, 2, 4, 6,
        1, 3, 5, 3, 7, 5,
    ];
    Mesh {
        positions,
        indices,
        material,
        uvs: Vec::new(),
    }
}

/// Bounds from two corners.
pub(crate) fn bounds(min: Vec3, max: Vec3) -> Bounds {
    Bounds { min, max }
}
