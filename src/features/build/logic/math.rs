//! Vector math in `f64` for placement: rays, planes, triangles, boxes and instance transforms.

use crate::features::build::models::Transform;

/// A point or direction in meters.
pub type Vec3 = [f64; 3];

/// `a + b`.
pub fn add(a: Vec3, b: Vec3) -> Vec3 {
    [a[0] + b[0], a[1] + b[1], a[2] + b[2]]
}

/// `a - b`.
pub fn sub(a: Vec3, b: Vec3) -> Vec3 {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

/// `a * s`.
pub fn scale(a: Vec3, s: f64) -> Vec3 {
    [a[0] * s, a[1] * s, a[2] * s]
}

/// Dot product.
pub fn dot(a: Vec3, b: Vec3) -> f64 {
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

/// Unit vector along `a` (`a` itself when it is zero).
pub fn normalize(a: Vec3) -> Vec3 {
    let length = dot(a, a).sqrt();
    if length <= f64::EPSILON {
        a
    } else {
        scale(a, 1.0 / length)
    }
}

/// Converts an `f32` point to `f64`.
pub fn widen(p: [f32; 3]) -> Vec3 {
    p.map(f64::from)
}

/// A 3×3 matrix, row-major.
pub type Mat3 = [[f64; 3]; 3];

/// `m · v`.
pub fn mul(m: &Mat3, v: Vec3) -> Vec3 {
    [dot(m[0], v), dot(m[1], v), dot(m[2], v)]
}

/// `mᵀ · v` (the inverse for a rotation matrix).
pub fn mul_transposed(m: &Mat3, v: Vec3) -> Vec3 {
    [
        m[0][0] * v[0] + m[1][0] * v[1] + m[2][0] * v[2],
        m[0][1] * v[0] + m[1][1] * v[1] + m[2][1] * v[2],
        m[0][2] * v[0] + m[1][2] * v[1] + m[2][2] * v[2],
    ]
}

/// Rotation matrix of Euler angles in degrees applied X, then Y, then Z (`Rz · Ry · Rx`).
pub fn rotation(degrees: [f32; 3]) -> Mat3 {
    let [x, y, z] = degrees.map(|d| f64::from(d).to_radians());
    let (sx, cx, sy, cy, sz, cz) = (x.sin(), x.cos(), y.sin(), y.cos(), z.sin(), z.cos());
    [
        [cz * cy, cz * sy * sx - sz * cx, cz * sy * cx + sz * sx],
        [sz * cy, sz * sy * sx + cz * cx, sz * sy * cx - cz * sx],
        [-sy, cy * sx, cy * cx],
    ]
}

/// World position of a local asset point under `transform`.
pub fn to_world(transform: &Transform, local: Vec3) -> Vec3 {
    let scaled = [
        local[0] * f64::from(transform.scale[0]),
        local[1] * f64::from(transform.scale[1]),
        local[2] * f64::from(transform.scale[2]),
    ];
    add(
        mul(&rotation(transform.rotation), scaled),
        transform.position,
    )
}

fn safe_scale(transform: &Transform) -> Vec3 {
    transform
        .scale
        .map(|s| if s.abs() < 1e-4 { 1e-4 } else { f64::from(s) })
}

/// A ray in the asset's local space; parameters along it match the world ray's.
pub fn to_local_ray(transform: &Transform, ray: &Ray) -> Ray {
    let matrix = rotation(transform.rotation);
    let s = safe_scale(transform);
    let unscale = |v: Vec3| [v[0] / s[0], v[1] / s[1], v[2] / s[2]];
    Ray {
        origin: unscale(mul_transposed(&matrix, sub(ray.origin, transform.position))),
        direction: unscale(mul_transposed(&matrix, ray.direction)),
    }
}

/// The 4×4 world matrix of `transform`, column-major (`m[column][row]`), for renderers.
pub fn world_matrix(transform: &Transform) -> [[f32; 4]; 4] {
    let r = rotation(transform.rotation);
    let s = transform.scale.map(f64::from);
    let p = transform.position;
    let mut out = [[0.0f32; 4]; 4];
    for column in 0..3 {
        for row in 0..3 {
            out[column][row] = (r[row][column] * s[column]) as f32;
        }
    }
    out[3] = [p[0] as f32, p[1] as f32, p[2] as f32, 1.0];
    out
}

/// A half-line from `origin` along `direction`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Ray {
    /// Start point.
    pub origin: Vec3,
    /// Direction (not necessarily unit length).
    pub direction: Vec3,
}

impl Ray {
    /// A ray with a unit direction.
    pub fn new(origin: Vec3, direction: Vec3) -> Self {
        Self {
            origin,
            direction: normalize(direction),
        }
    }

    /// The point at parameter `t`.
    pub fn at(&self, t: f64) -> Vec3 {
        add(self.origin, scale(self.direction, t))
    }
}

/// Parameter where `ray` meets the plane through `point` with `normal`, if ahead of the origin.
pub fn ray_plane(ray: &Ray, point: Vec3, normal: Vec3) -> Option<f64> {
    let denominator = dot(ray.direction, normal);
    if denominator.abs() < 1e-12 {
        return None;
    }
    let t = dot(sub(point, ray.origin), normal) / denominator;
    (t >= 0.0).then_some(t)
}

/// Parameter where `ray` meets triangle `abc` from either side (Möller–Trumbore).
pub fn ray_triangle(ray: &Ray, a: Vec3, b: Vec3, c: Vec3) -> Option<f64> {
    let (e1, e2) = (sub(b, a), sub(c, a));
    let p = cross(ray.direction, e2);
    let det = dot(e1, p);
    if det.abs() < 1e-12 {
        return None;
    }
    let inverse = 1.0 / det;
    let s = sub(ray.origin, a);
    let u = dot(s, p) * inverse;
    if !(0.0..=1.0).contains(&u) {
        return None;
    }
    let q = cross(s, e1);
    let v = dot(ray.direction, q) * inverse;
    if v < 0.0 || u + v > 1.0 {
        return None;
    }
    let t = dot(e2, q) * inverse;
    (t >= 0.0).then_some(t)
}

/// Entry parameter of `ray` into the box `min..max` (0 when the origin is inside).
pub fn ray_box(ray: &Ray, min: Vec3, max: Vec3) -> Option<f64> {
    let (mut near, mut far) = (0.0f64, f64::INFINITY);
    for axis in 0..3 {
        if ray.direction[axis].abs() < 1e-12 {
            if ray.origin[axis] < min[axis] || ray.origin[axis] > max[axis] {
                return None;
            }
            continue;
        }
        let inverse = 1.0 / ray.direction[axis];
        let mut t0 = (min[axis] - ray.origin[axis]) * inverse;
        let mut t1 = (max[axis] - ray.origin[axis]) * inverse;
        if t0 > t1 {
            std::mem::swap(&mut t0, &mut t1);
        }
        near = near.max(t0);
        far = far.min(t1);
        if near > far {
            return None;
        }
    }
    Some(near)
}
