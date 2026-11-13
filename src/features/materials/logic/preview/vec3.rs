//! Minimal 3-vector helpers for the preview shader.

/// A 3D vector.
pub type V3 = [f32; 3];

/// Dot product.
pub fn dot(a: V3, b: V3) -> f32 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

/// Cross product.
pub fn cross(a: V3, b: V3) -> V3 {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}

/// Component-wise sum.
pub fn add(a: V3, b: V3) -> V3 {
    [a[0] + b[0], a[1] + b[1], a[2] + b[2]]
}

/// Vector times scalar.
pub fn scale(a: V3, s: f32) -> V3 {
    [a[0] * s, a[1] * s, a[2] * s]
}

/// Component-wise product.
pub fn mul(a: V3, b: V3) -> V3 {
    [a[0] * b[0], a[1] * b[1], a[2] * b[2]]
}

/// Linear interpolation.
pub fn lerp(a: V3, b: V3, t: f32) -> V3 {
    add(scale(a, 1.0 - t), scale(b, t))
}

/// The unit vector along `a` (the input when it has no length).
pub fn normalize(a: V3) -> V3 {
    let length = dot(a, a).sqrt();
    if length > 1e-8 {
        scale(a, 1.0 / length)
    } else {
        a
    }
}

/// A rotation matrix stored as three rows.
#[derive(Debug, Clone, Copy)]
pub struct Rotation([V3; 3]);

impl Rotation {
    /// Rotation by `yaw` about Y after `pitch` about X (radians).
    pub fn yaw_pitch(yaw: f32, pitch: f32) -> Self {
        let (sy, cy) = yaw.sin_cos();
        let (sp, cp) = pitch.sin_cos();
        // Ry * Rx
        Self([
            [cy, sy * sp, sy * cp],
            [0.0, cp, -sp],
            [-sy, cy * sp, cy * cp],
        ])
    }

    /// Rotates `v` (object to view).
    pub fn apply(&self, v: V3) -> V3 {
        [dot(self.0[0], v), dot(self.0[1], v), dot(self.0[2], v)]
    }

    /// Rotates `v` by the inverse (view to object); the matrix is orthonormal so this is the transpose.
    pub fn inverse_apply(&self, v: V3) -> V3 {
        let m = &self.0;
        [
            m[0][0] * v[0] + m[1][0] * v[1] + m[2][0] * v[2],
            m[0][1] * v[0] + m[1][1] * v[1] + m[2][1] * v[2],
            m[0][2] * v[0] + m[1][2] * v[1] + m[2][2] * v[2],
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rotation_round_trips_and_preserves_length() {
        let r = Rotation::yaw_pitch(0.7, -0.4);
        let v = [0.3, -0.5, 0.8];
        let back = r.inverse_apply(r.apply(v));
        for i in 0..3 {
            assert!((back[i] - v[i]).abs() < 1e-5);
        }
        assert!((dot(r.apply(v), r.apply(v)) - dot(v, v)).abs() < 1e-5);
    }

    #[test]
    fn identity_rotation_leaves_vectors_alone() {
        let r = Rotation::yaw_pitch(0.0, 0.0);
        assert_eq!(r.apply([1.0, 2.0, 3.0]), [1.0, 2.0, 3.0]);
    }

    #[test]
    fn cross_and_normalize_behave() {
        assert_eq!(cross([1.0, 0.0, 0.0], [0.0, 1.0, 0.0]), [0.0, 0.0, 1.0]);
        assert!((dot(normalize([3.0, 4.0, 0.0]), [1.0, 0.0, 0.0]) - 0.6).abs() < 1e-6);
        assert_eq!(normalize([0.0; 3]), [0.0; 3]);
    }
}
