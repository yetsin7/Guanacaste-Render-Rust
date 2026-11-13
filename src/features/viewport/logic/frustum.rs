//! Visibility test of an axis-aligned box against the camera frustum, used to skip chunks that are off screen.

use super::math::{self, Mat4};
use crate::features::import_export::models::Bounds;

/// Whether any part of `bounds` may be inside the frustum of `view_proj` (depth range `0..1`). Conservative: it can
/// say true for a box that is just outside, never false for a visible one.
pub fn may_be_visible(view_proj: &Mat4, bounds: &Bounds) -> bool {
    let mut outside = [true; 6];
    for corner in 0..8 {
        let p = [
            if corner & 1 == 0 {
                bounds.min[0]
            } else {
                bounds.max[0]
            },
            if corner & 2 == 0 {
                bounds.min[1]
            } else {
                bounds.max[1]
            },
            if corner & 4 == 0 {
                bounds.min[2]
            } else {
                bounds.max[2]
            },
        ];
        let [x, y, z, w] = math::transform_point(view_proj, p);
        outside[0] &= x < -w;
        outside[1] &= x > w;
        outside[2] &= y < -w;
        outside[3] &= y > w;
        outside[4] &= z < 0.0;
        outside[5] &= z > w;
    }
    !outside.iter().any(|o| *o)
}
