//! Easing curves: they remap the progress `0..=1` of a segment before the value is interpolated.

use crate::features::movie::models::Ease;

/// Remaps `progress` (clamped to `0..=1`) with the curve; the result starts at 0 and ends at 1, except for
/// [`Ease::Hold`], which stays at 0 until the next key.
pub fn apply(ease: Ease, progress: f64) -> f64 {
    let u = progress.clamp(0.0, 1.0);
    match ease {
        Ease::Linear => u,
        Ease::EaseIn => u * u * u,
        Ease::EaseOut => 1.0 - (1.0 - u).powi(3),
        Ease::EaseInOut => {
            if u < 0.5 {
                4.0 * u * u * u
            } else {
                1.0 - (-2.0 * u + 2.0).powi(3) / 2.0
            }
        }
        Ease::Hold => 0.0,
    }
}

/// Linear interpolation.
pub fn lerp(a: f64, b: f64, t: f64) -> f64 {
    a + (b - a) * t
}

/// Cubic Hermite interpolation of a scalar between `p0` and `p1` over a span of `span` seconds with the given
/// slopes (units per second); `t` is the progress `0..=1`.
pub fn hermite(p0: f64, m0: f64, p1: f64, m1: f64, span: f64, t: f64) -> f64 {
    let t2 = t * t;
    let t3 = t2 * t;
    (2.0 * t3 - 3.0 * t2 + 1.0) * p0
        + (t3 - 2.0 * t2 + t) * span * m0
        + (-2.0 * t3 + 3.0 * t2) * p1
        + (t3 - t2) * span * m1
}
