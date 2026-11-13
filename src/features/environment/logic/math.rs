//! Small vector and shaping helpers shared by the environment functions.

/// A direction or position `[east, up, north]`.
pub type Vec3 = [f32; 3];

/// Dot product.
pub fn dot(a: Vec3, b: Vec3) -> f32 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

/// Unit vector along `v`; the zenith for a zero vector.
pub fn normalize(v: Vec3) -> Vec3 {
    let length = dot(v, v).sqrt();
    if length > 1e-12 {
        [v[0] / length, v[1] / length, v[2] / length]
    } else {
        [0.0, 1.0, 0.0]
    }
}

/// Unit vector for an azimuth (degrees clockwise from north) and an elevation (degrees above the horizon).
pub fn direction_from_angles(azimuth_deg: f32, elevation_deg: f32) -> Vec3 {
    let (az, el) = (azimuth_deg.to_radians(), elevation_deg.to_radians());
    [az.sin() * el.cos(), el.sin(), az.cos() * el.cos()]
}

/// Azimuth (0..360, clockwise from north) and elevation in degrees of a direction.
pub fn angles_from_direction(direction: Vec3) -> (f32, f32) {
    let d = normalize(direction);
    let azimuth = d[0].atan2(d[2]).to_degrees().rem_euclid(360.0);
    (azimuth, d[1].clamp(-1.0, 1.0).asin().to_degrees())
}

/// Hermite smooth step from 0 at `edge0` to 1 at `edge1` (works for reversed edges too).
pub fn smoothstep(edge0: f32, edge1: f32, x: f32) -> f32 {
    let t = ((x - edge0) / (edge1 - edge0)).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

/// Linear interpolation.
pub fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t
}
