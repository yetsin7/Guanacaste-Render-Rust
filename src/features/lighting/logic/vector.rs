//! Small 3D vector helpers and the orthonormal frame of a light.

use crate::features::lighting::models::{Orientation, Vec3};

/// `a + b`.
pub fn add(a: Vec3, b: Vec3) -> Vec3 {
    [a[0] + b[0], a[1] + b[1], a[2] + b[2]]
}

/// `a - b`.
pub fn sub(a: Vec3, b: Vec3) -> Vec3 {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

/// `a * s`.
pub fn scale(a: Vec3, s: f32) -> Vec3 {
    [a[0] * s, a[1] * s, a[2] * s]
}

/// Dot product.
pub fn dot(a: Vec3, b: Vec3) -> f32 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

/// Cross product.
pub fn cross(a: Vec3, b: Vec3) -> Vec3 {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}

/// Euclidean length.
pub fn length(a: Vec3) -> f32 {
    dot(a, a).sqrt()
}

/// `a` scaled to unit length; the zero vector stays zero.
pub fn normalize(a: Vec3) -> Vec3 {
    let len = length(a);
    if len > 0.0 {
        scale(a, 1.0 / len)
    } else {
        a
    }
}

/// Orthonormal frame of a light: it emits along `forward`; area lights span `right` × `up`, linear lights lie
/// along `right`; IES horizontal angles start at `right` and turn toward `up`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Basis {
    /// Emission axis.
    pub forward: Vec3,
    /// First tangent.
    pub right: Vec3,
    /// Second tangent.
    pub up: Vec3,
}

impl Basis {
    /// `world` expressed in this frame as `(right, up, forward)` components.
    pub fn to_local(&self, world: Vec3) -> Vec3 {
        [
            dot(world, self.right),
            dot(world, self.up),
            dot(world, self.forward),
        ]
    }
}

/// The frame of an orientation (Y up; yaw 0 and pitch 0 look toward -Z with `right` = +X).
pub fn basis(orientation: Orientation) -> Basis {
    let (sy, cy) = orientation.yaw_deg.to_radians().sin_cos();
    let (sp, cp) = orientation.pitch_deg.to_radians().sin_cos();
    let (sr, cr) = orientation.roll_deg.to_radians().sin_cos();
    let forward = [-sy * cp, sp, -cy * cp];
    let right0 = [cy, 0.0, -sy];
    let up0 = cross(right0, forward);
    Basis {
        forward,
        right: add(scale(right0, cr), scale(up0, sr)),
        up: sub(scale(up0, cr), scale(right0, sr)),
    }
}
