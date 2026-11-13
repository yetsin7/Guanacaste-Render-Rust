//! Candela lookup in an IES profile for a direction in the light's local frame.
//!
//! Local frame: `(x, y, z)` = components along the light's `right`, `up` and `forward` axes (see
//! [`super::vector::Basis::to_local`]). For type C photometry `forward` is the nadir (vertical angle 0) and the
//! horizontal angle turns from `right` (0°) toward `up` (90°). For types A and B `forward` is the beam axis, the
//! horizontal angle turns around `up` (positive toward `right`) and the vertical angle is the elevation toward `up`.
//! Values are bilinearly interpolated; directions outside the measured vertical range get zero.

use super::vector::normalize;
use crate::features::lighting::models::{IesProfile, PhotometricType, Vec3};

/// Index of the segment containing `value` in ascending `angles` and the blend inside it, or `None` outside.
fn locate(angles: &[f32], value: f32) -> Option<(usize, f32)> {
    let (first, last) = (*angles.first()?, *angles.last()?);
    if angles.len() == 1 {
        return (value - first).abs().le(&1e-3).then_some((0, 0.0));
    }
    if value < first - 1e-4 || value > last + 1e-4 {
        return None;
    }
    let value = value.clamp(first, last);
    let upper = angles
        .partition_point(|&a| a < value)
        .clamp(1, angles.len() - 1);
    let (a, b) = (angles[upper - 1], angles[upper]);
    Some((upper - 1, (value - a) / (b - a)))
}

/// Folds a horizontal angle in `[0, 360)` into the range the file measured, using its symmetry.
fn fold_type_c(horizontal: &[f32], angle: f32) -> f32 {
    let first = horizontal.first().copied().unwrap_or(0.0);
    let last = horizontal.last().copied().unwrap_or(0.0);
    if horizontal.len() == 1 {
        return first;
    }
    if first.abs() < 1e-3 && (last - 90.0).abs() < 1e-3 {
        let angle = if angle > 180.0 { 360.0 - angle } else { angle };
        return if angle > 90.0 { 180.0 - angle } else { angle };
    }
    if first.abs() < 1e-3 && (last - 180.0).abs() < 1e-3 {
        return if angle > 180.0 { 360.0 - angle } else { angle };
    }
    if (first - 90.0).abs() < 1e-3 && (last - 270.0).abs() < 1e-3 {
        return match angle {
            a if a < 90.0 => 180.0 - a,
            a if a > 270.0 => 540.0 - a,
            a => a,
        };
    }
    angle.min(last)
}

/// Mirrors a signed angle onto the measured half when the file starts at 0 (symmetric types A and B).
fn fold_signed(angles: &[f32], angle: f32) -> f32 {
    if angles.first().is_some_and(|first| first.abs() < 1e-3) {
        angle.abs()
    } else {
        angle
    }
}

/// Bilinear candela value at `(horizontal, vertical)` degrees, already folded.
fn sample(profile: &IesProfile, horizontal: f32, vertical: f32) -> f32 {
    let Some((v, tv)) = locate(&profile.vertical, vertical) else {
        return 0.0;
    };
    let (h, th) = locate(&profile.horizontal, horizontal).unwrap_or((0, 0.0));
    let at = |hi: usize, vi: usize| -> f32 {
        let row = &profile.candela[hi.min(profile.candela.len() - 1)];
        row[vi.min(row.len() - 1)]
    };
    let lerp = |a: f32, b: f32, t: f32| a + (b - a) * t;
    let low = lerp(at(h, v), at(h, v + 1), tv);
    let high = lerp(at(h + 1, v), at(h + 1, v + 1), tv);
    lerp(low, high, th)
}

/// Luminous intensity in candela emitted by `profile` toward `local_dir` (any length; see the module docs for the
/// frame).
pub fn ies_intensity(profile: &IesProfile, local_dir: Vec3) -> f32 {
    if profile.candela.is_empty() || profile.vertical.is_empty() || profile.horizontal.is_empty() {
        return 0.0;
    }
    let [x, y, z] = normalize(local_dir);
    if x == 0.0 && y == 0.0 && z == 0.0 {
        return 0.0;
    }
    match profile.photometric_type {
        PhotometricType::C => {
            let vertical = z.clamp(-1.0, 1.0).acos().to_degrees();
            let horizontal = y.atan2(x).to_degrees().rem_euclid(360.0);
            sample(
                profile,
                fold_type_c(&profile.horizontal, horizontal),
                vertical,
            )
        }
        PhotometricType::A | PhotometricType::B => {
            let vertical = y.clamp(-1.0, 1.0).asin().to_degrees();
            let horizontal = x.atan2(z).to_degrees();
            sample(
                profile,
                fold_signed(&profile.horizontal, horizontal),
                fold_signed(&profile.vertical, vertical),
            )
        }
    }
}

/// Intensity toward `local_dir` relative to the brightest direction of the profile, 0..=1.
pub fn ies_shape(profile: &IesProfile, local_dir: Vec3) -> f32 {
    let max = profile.max_candela();
    if max > 0.0 {
        (ies_intensity(profile, local_dir) / max).clamp(0.0, 1.0)
    } else {
        0.0
    }
}
