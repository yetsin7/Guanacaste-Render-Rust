//! Pinhole/thin-lens geometry: field of view, focal length and the film gate. Angles are in radians and lengths
//! in millimetres unless a name says otherwise. All math runs in `f64` so the focal length <-> angle conversion
//! round-trips to well below `f32` precision.

/// Diagonal of a 36 x 24 mm full-frame sensor, the reference of "35 mm equivalent" focal lengths.
pub const FULL_FRAME_DIAGONAL_MM: f32 = 43.266_615;

/// Field of view for a film `gate_mm` wide (or tall) seen through a lens of `focal_mm`.
pub fn fov_from_focal(focal_mm: f32, gate_mm: f32) -> f32 {
    (2.0 * (f64::from(gate_mm) / (2.0 * f64::from(focal_mm))).atan()) as f32
}

/// Focal length that gives a field of view of `fov` across a film `gate_mm` wide (or tall). Exact inverse of
/// [`fov_from_focal`].
pub fn focal_from_fov(fov: f32, gate_mm: f32) -> f32 {
    (f64::from(gate_mm) / (2.0 * (f64::from(fov) / 2.0).tan())) as f32
}

/// The largest rectangle of the given width/height `aspect` that fits inside a sensor, as `(width, height)`.
/// This is the part of the sensor that is exposed for that framing.
pub fn gate_size(sensor_width: f32, sensor_height: f32, aspect: f32) -> (f32, f32) {
    let aspect = aspect.max(1e-3);
    if sensor_width / aspect <= sensor_height {
        (sensor_width, sensor_width / aspect)
    } else {
        (sensor_height * aspect, sensor_height)
    }
}

/// Focal length of a full-frame lens that frames the same angle as `focal_mm` on a sensor with the given
/// diagonal (the "35 mm equivalent").
pub fn equivalent_focal(focal_mm: f32, sensor_diagonal_mm: f32) -> f32 {
    focal_mm * FULL_FRAME_DIAGONAL_MM / sensor_diagonal_mm.max(1e-3)
}

/// Maps a normalized slider position to a value on a logarithmic scale between `min` and `max`.
pub fn log_value(t: f32, min: f32, max: f32) -> f32 {
    let t = f64::from(t.clamp(0.0, 1.0));
    (f64::from(min) * (f64::from(max) / f64::from(min)).powf(t)) as f32
}

/// Inverse of [`log_value`]: the normalized position of `value` on the logarithmic scale.
pub fn log_position(value: f32, min: f32, max: f32) -> f32 {
    let value = f64::from(value.clamp(min, max));
    ((value / f64::from(min)).ln() / (f64::from(max) / f64::from(min)).ln()) as f32
}
