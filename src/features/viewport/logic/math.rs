//! Small vector and matrix helpers (`f32`, right-handed, Z up, column-major matrices, depth range `0..1`).

/// A 3D vector or point.
pub type Vec3 = [f32; 3];
/// A 4x4 matrix stored as four columns: `m[column][row]`.
pub type Mat4 = [[f32; 4]; 4];

/// The world up axis.
pub const Z: Vec3 = [0.0, 0.0, 1.0];

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

/// `a` scaled to unit length, or `fallback` when `a` is (almost) zero or not finite.
pub fn normalize_or(a: Vec3, fallback: Vec3) -> Vec3 {
    let len = length(a);
    if len.is_finite() && len > 1e-12 {
        scale(a, 1.0 / len)
    } else {
        fallback
    }
}

/// Linear interpolation from `a` to `b`.
pub fn lerp(a: Vec3, b: Vec3, t: f32) -> Vec3 {
    add(scale(a, 1.0 - t), scale(b, t))
}

/// Matrix product `a * b`.
pub fn mul(a: &Mat4, b: &Mat4) -> Mat4 {
    let mut out = [[0.0; 4]; 4];
    for (column, out_column) in out.iter_mut().enumerate() {
        for (row, value) in out_column.iter_mut().enumerate() {
            *value = (0..4).map(|k| a[k][row] * b[column][k]).sum();
        }
    }
    out
}

/// Right-handed view matrix looking from `eye` to `target`.
pub fn look_at(eye: Vec3, target: Vec3, up: Vec3) -> Mat4 {
    let f = normalize_or(sub(target, eye), [0.0, 1.0, 0.0]);
    let s = normalize_or(cross(f, up), [1.0, 0.0, 0.0]);
    let u = cross(s, f);
    [
        [s[0], u[0], -f[0], 0.0],
        [s[1], u[1], -f[1], 0.0],
        [s[2], u[2], -f[2], 0.0],
        [-dot(s, eye), -dot(u, eye), dot(f, eye), 1.0],
    ]
}

/// Right-handed perspective projection with clip depth in `0..1`.
pub fn perspective(fov_y: f32, aspect: f32, near: f32, far: f32) -> Mat4 {
    let focal = 1.0 / (fov_y * 0.5).tan();
    let aspect = if aspect.is_finite() && aspect > 1e-6 {
        aspect
    } else {
        1.0
    };
    let range = near - far;
    [
        [focal / aspect, 0.0, 0.0, 0.0],
        [0.0, focal, 0.0, 0.0],
        [0.0, 0.0, far / range, -1.0],
        [0.0, 0.0, near * far / range, 0.0],
    ]
}

/// Transforms the point `p` (w = 1) by `m`, returning homogeneous `[x, y, z, w]`.
pub fn transform_point(m: &Mat4, p: Vec3) -> [f32; 4] {
    let mut out = [0.0; 4];
    for (row, value) in out.iter_mut().enumerate() {
        *value = m[0][row] * p[0] + m[1][row] * p[1] + m[2][row] * p[2] + m[3][row];
    }
    out
}

/// Smooth ease in/out of `t` clamped to `0..=1`.
pub fn smoothstep(t: f32) -> f32 {
    let t = t.clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}
