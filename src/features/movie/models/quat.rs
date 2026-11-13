//! Unit quaternions stored as `[x, y, z, w]`: the small set of operations the camera animation needs.
//!
//! Orientation convention of the application: Z points up, the camera looks along +Y when the quaternion is the
//! identity, yaw turns around +Z, pitch around +X (positive looks up) and roll around +Y. The composition order
//! is `yaw * pitch * roll`.

/// A quaternion `[x, y, z, w]`.
pub type Quat = [f64; 4];

/// The rotation that changes nothing.
pub const IDENTITY: Quat = [0.0, 0.0, 0.0, 1.0];

/// Hamilton product `a * b` (applies `b` first, then `a`).
pub fn mul(a: Quat, b: Quat) -> Quat {
    [
        a[3] * b[0] + a[0] * b[3] + a[1] * b[2] - a[2] * b[1],
        a[3] * b[1] - a[0] * b[2] + a[1] * b[3] + a[2] * b[0],
        a[3] * b[2] + a[0] * b[1] - a[1] * b[0] + a[2] * b[3],
        a[3] * b[3] - a[0] * b[0] - a[1] * b[1] - a[2] * b[2],
    ]
}

/// Conjugate (the inverse of a unit quaternion).
pub fn conjugate(q: Quat) -> Quat {
    [-q[0], -q[1], -q[2], q[3]]
}

/// Four-dimensional dot product.
pub fn dot(a: Quat, b: Quat) -> f64 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2] + a[3] * b[3]
}

/// Scales `q` to unit length; a degenerate input becomes the identity.
pub fn normalize(q: Quat) -> Quat {
    let length = dot(q, q).sqrt();
    if length < 1e-12 || !length.is_finite() {
        return IDENTITY;
    }
    [q[0] / length, q[1] / length, q[2] / length, q[3] / length]
}

/// `q` or `-q`, whichever lies in the same hemisphere as `reference` (so interpolation takes the short way).
pub fn same_hemisphere(q: Quat, reference: Quat) -> Quat {
    if dot(q, reference) < 0.0 {
        [-q[0], -q[1], -q[2], -q[3]]
    } else {
        q
    }
}

/// Spherical linear interpolation along the shortest arc.
pub fn slerp(a: Quat, b: Quat, t: f64) -> Quat {
    let b = same_hemisphere(b, a);
    let cosine = dot(a, b).clamp(-1.0, 1.0);
    if cosine > 0.9995 {
        return normalize([
            a[0] + (b[0] - a[0]) * t,
            a[1] + (b[1] - a[1]) * t,
            a[2] + (b[2] - a[2]) * t,
            a[3] + (b[3] - a[3]) * t,
        ]);
    }
    let angle = cosine.acos();
    let sine = angle.sin();
    let wa = ((1.0 - t) * angle).sin() / sine;
    let wb = (t * angle).sin() / sine;
    [
        a[0] * wa + b[0] * wb,
        a[1] * wa + b[1] * wb,
        a[2] * wa + b[2] * wb,
        a[3] * wa + b[3] * wb,
    ]
}

/// Logarithm of a unit quaternion: the rotation vector (half-angle times axis).
pub fn log(q: Quat) -> [f64; 3] {
    let vector = (q[0] * q[0] + q[1] * q[1] + q[2] * q[2]).sqrt();
    if vector < 1e-12 {
        return [0.0; 3];
    }
    let scale = vector.atan2(q[3]) / vector;
    [q[0] * scale, q[1] * scale, q[2] * scale]
}

/// Exponential of a rotation vector; the inverse of [`log`].
pub fn exp(v: [f64; 3]) -> Quat {
    let angle = (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt();
    if angle < 1e-12 {
        return IDENTITY;
    }
    let scale = angle.sin() / angle;
    [v[0] * scale, v[1] * scale, v[2] * scale, angle.cos()]
}

/// Inner control point of the smooth quaternion spline (SQUAD) at `current`, given its neighbours.
pub fn inner_control(previous: Quat, current: Quat, next: Quat) -> Quat {
    let inverse = conjugate(current);
    let to_next = log(mul(inverse, same_hemisphere(next, current)));
    let to_previous = log(mul(inverse, same_hemisphere(previous, current)));
    let blend = [
        -(to_next[0] + to_previous[0]) / 4.0,
        -(to_next[1] + to_previous[1]) / 4.0,
        -(to_next[2] + to_previous[2]) / 4.0,
    ];
    normalize(mul(current, exp(blend)))
}

/// Spherical cubic interpolation between `q1` and `q2` with inner control points `s1` and `s2`.
pub fn squad(q1: Quat, q2: Quat, s1: Quat, s2: Quat, t: f64) -> Quat {
    let outer = slerp(q1, q2, t);
    let inner = slerp(s1, s2, t);
    slerp(outer, inner, 2.0 * t * (1.0 - t))
}

/// Rotation of `angle` radians around the unit axis `axis`.
fn axis_angle(axis: [f64; 3], angle: f64) -> Quat {
    let (sine, cosine) = (angle / 2.0).sin_cos();
    [axis[0] * sine, axis[1] * sine, axis[2] * sine, cosine]
}

/// Builds a quaternion from yaw, pitch and roll in degrees.
pub fn from_euler_deg(yaw: f64, pitch: f64, roll: f64) -> Quat {
    let z = axis_angle([0.0, 0.0, 1.0], yaw.to_radians());
    let x = axis_angle([1.0, 0.0, 0.0], pitch.to_radians());
    let y = axis_angle([0.0, 1.0, 0.0], roll.to_radians());
    normalize(mul(mul(z, x), y))
}

/// Yaw, pitch and roll in degrees of a unit quaternion (pitch within -90..=90).
pub fn to_euler_deg(q: Quat) -> [f64; 3] {
    let [x, y, z, w] = normalize(q);
    let r01 = 2.0 * (x * y - z * w);
    let r11 = 1.0 - 2.0 * (x * x + z * z);
    let r20 = 2.0 * (x * z - y * w);
    let r21 = 2.0 * (y * z + x * w);
    let r22 = 1.0 - 2.0 * (x * x + y * y);
    let pitch = r21.clamp(-1.0, 1.0).asin();
    let yaw = (-r01).atan2(r11);
    let roll = (-r20).atan2(r22);
    [yaw.to_degrees(), pitch.to_degrees(), roll.to_degrees()]
}

/// Rotates the vector `v` by `q`.
pub fn rotate(q: Quat, v: [f64; 3]) -> [f64; 3] {
    let p = mul(mul(q, [v[0], v[1], v[2], 0.0]), conjugate(q));
    [p[0], p[1], p[2]]
}

/// Angle in radians between two orientations (0 when equal), taking the short way round.
pub fn angle_between(a: Quat, b: Quat) -> f64 {
    2.0 * dot(a, b).abs().clamp(0.0, 1.0).acos()
}
