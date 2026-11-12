//! Small vector and 4x4 matrix helpers in `f64`.

/// A 3D vector.
pub type V3 = [f64; 3];

/// Vector difference.
pub fn sub(a: V3, b: V3) -> V3 {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

/// Vector sum.
pub fn add(a: V3, b: V3) -> V3 {
    [a[0] + b[0], a[1] + b[1], a[2] + b[2]]
}

/// Vector scaled by `s`.
pub fn scale(a: V3, s: f64) -> V3 {
    [a[0] * s, a[1] * s, a[2] * s]
}

/// Dot product.
pub fn dot(a: V3, b: V3) -> f64 {
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

/// Euclidean length.
pub fn length(a: V3) -> f64 {
    dot(a, a).sqrt()
}

/// Unit vector; the input is returned unchanged when it has no length.
pub fn normalize(a: V3) -> V3 {
    let len = length(a);
    if len < 1e-12 {
        a
    } else {
        scale(a, 1.0 / len)
    }
}

/// A rigid or scaled transform, row-major, applied to column vectors.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Mat4(pub [[f64; 4]; 4]);

impl Mat4 {
    /// The identity transform.
    pub const IDENTITY: Mat4 = Mat4([
        [1.0, 0.0, 0.0, 0.0],
        [0.0, 1.0, 0.0, 0.0],
        [0.0, 0.0, 1.0, 0.0],
        [0.0, 0.0, 0.0, 1.0],
    ]);

    /// A transform whose columns are the axes `x`, `y`, `z` and the translation `origin`.
    pub fn from_axes(origin: V3, x: V3, y: V3, z: V3) -> Mat4 {
        Mat4([
            [x[0], y[0], z[0], origin[0]],
            [x[1], y[1], z[1], origin[1]],
            [x[2], y[2], z[2], origin[2]],
            [0.0, 0.0, 0.0, 1.0],
        ])
    }

    /// A pure translation.
    pub fn translation(offset: V3) -> Mat4 {
        Mat4::from_axes(offset, [1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0])
    }

    /// The product `self * other` (`other` is applied first).
    pub fn mul(&self, other: &Mat4) -> Mat4 {
        let mut out = [[0.0; 4]; 4];
        for (row, out_row) in out.iter_mut().enumerate() {
            for (column, cell) in out_row.iter_mut().enumerate() {
                *cell = (0..4).map(|k| self.0[row][k] * other.0[k][column]).sum();
            }
        }
        Mat4(out)
    }

    /// Transforms a point.
    pub fn point(&self, p: V3) -> V3 {
        let m = &self.0;
        [
            m[0][0] * p[0] + m[0][1] * p[1] + m[0][2] * p[2] + m[0][3],
            m[1][0] * p[0] + m[1][1] * p[1] + m[1][2] * p[2] + m[1][3],
            m[2][0] * p[0] + m[2][1] * p[1] + m[2][2] * p[2] + m[2][3],
        ]
    }

    /// Transforms a direction (ignores the translation).
    pub fn direction(&self, d: V3) -> V3 {
        let m = &self.0;
        [
            m[0][0] * d[0] + m[0][1] * d[1] + m[0][2] * d[2],
            m[1][0] * d[0] + m[1][1] * d[1] + m[1][2] * d[2],
            m[2][0] * d[0] + m[2][1] * d[1] + m[2][2] * d[2],
        ]
    }

    /// Inverse of a transform made of orthonormal axes (uniform scale 1) and a translation.
    pub fn inverse_rigid(&self) -> Mat4 {
        let m = &self.0;
        let rotation = [
            [m[0][0], m[1][0], m[2][0]],
            [m[0][1], m[1][1], m[2][1]],
            [m[0][2], m[1][2], m[2][2]],
        ];
        let t = [m[0][3], m[1][3], m[2][3]];
        let mut out = Mat4::IDENTITY;
        for row in 0..3 {
            for column in 0..3 {
                out.0[row][column] = rotation[row][column];
            }
            out.0[row][3] =
                -(rotation[row][0] * t[0] + rotation[row][1] * t[1] + rotation[row][2] * t[2]);
        }
        out
    }
}

/// Builds the transform of an IFC axis placement: `z` is the main axis, `x` the reference direction.
pub fn placement(origin: V3, axis: Option<V3>, reference: Option<V3>) -> Mat4 {
    let z = normalize(axis.unwrap_or([0.0, 0.0, 1.0]));
    let hint = reference.unwrap_or(if z[0].abs() > 0.9 {
        [0.0, 1.0, 0.0]
    } else {
        [1.0, 0.0, 0.0]
    });
    let x = normalize(sub(hint, scale(z, dot(hint, z))));
    let y = cross(z, x);
    Mat4::from_axes(origin, x, y, z)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn placement_with_defaults_is_a_translation() {
        let m = placement([1.0, 2.0, 3.0], None, None);
        assert_eq!(m.point([1.0, 1.0, 1.0]), [2.0, 3.0, 4.0]);
    }

    #[test]
    fn rotated_placement_maps_axes() {
        let m = placement([0.0; 3], Some([0.0, 0.0, 1.0]), Some([0.0, 1.0, 0.0]));
        let p = m.point([1.0, 0.0, 0.0]);
        assert!((p[0]).abs() < 1e-12 && (p[1] - 1.0).abs() < 1e-12);
    }

    #[test]
    fn inverse_undoes_the_transform() {
        let m = placement(
            [5.0, -2.0, 1.0],
            Some([0.0, 1.0, 0.0]),
            Some([1.0, 0.0, 0.0]),
        );
        let back = m.inverse_rigid().point(m.point([1.5, 2.5, -3.0]));
        assert!(length(sub(back, [1.5, 2.5, -3.0])) < 1e-9);
    }
}
