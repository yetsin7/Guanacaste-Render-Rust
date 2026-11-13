//! Thin-lens depth of field. Lengths are in millimetres inside the functions that say `_mm`; the camera model
//! converts from metres.

/// Default circle of confusion: the sensor diagonal divided by 1500 (0.029 mm on full frame).
pub fn circle_of_confusion_mm(sensor_diagonal_mm: f32) -> f32 {
    sensor_diagonal_mm / 1500.0
}

/// Hyperfocal distance `H = f^2 / (N c) + f`: focusing there keeps everything from `H / 2` to infinity sharp.
pub fn hyperfocal_mm(focal_mm: f32, aperture: f32, coc_mm: f32) -> f32 {
    let (f, n, c) = (f64::from(focal_mm), f64::from(aperture), f64::from(coc_mm));
    (f * f / (n * c) + f) as f32
}

/// Near and far limits of acceptable sharpness for a subject at `focus_mm`; `far` is `None` (infinity) when the
/// focus distance reaches the hyperfocal distance.
pub fn limits_mm(focal_mm: f32, hyperfocal_mm: f32, focus_mm: f32) -> (f32, Option<f32>) {
    let (f, h, s) = (
        f64::from(focal_mm),
        f64::from(hyperfocal_mm),
        f64::from(focus_mm),
    );
    let near = s * (h - f) / (h + s - 2.0 * f);
    let far = (s < h).then(|| (s * (h - f) / (h - s)) as f32);
    (near as f32, far)
}

/// Diameter, on the sensor, of the blur disc of a point at `object_mm` when the lens is focused at `focus_mm`:
/// `b = f^2 |d - s| / (N d (s - f))`. Zero at the focus plane; it grows without bound in front of it and
/// saturates behind it.
pub fn blur_diameter_mm(focal_mm: f32, aperture: f32, focus_mm: f32, object_mm: f32) -> f32 {
    let (f, n, s, d) = (
        f64::from(focal_mm),
        f64::from(aperture),
        f64::from(focus_mm).max(f64::from(focal_mm) * 1.001),
        f64::from(object_mm).max(1e-3),
    );
    (f * f * (d - s).abs() / (n * d * (s - f))) as f32
}
