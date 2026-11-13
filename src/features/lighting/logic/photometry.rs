//! Photometric conversions: lumens to candela, distance attenuation, spot cone falloff and light color.

use super::ies_eval::{ies_intensity, ies_shape};
use super::kelvin::kelvin_to_rgb;
use crate::features::lighting::models::{Light, LightKind, Rgb, Vec3};
use std::f32::consts::PI;

/// Distances below this (meters) are clamped to keep the inverse square finite.
pub const MIN_DISTANCE: f32 = 0.01;

/// Smooth window that brings a light to exactly zero at `radius`: `(1 - (d/r)^4)^2` clamped to `0..=1`.
/// A radius of zero or less disables the cut-off.
pub fn distance_window(distance: f32, radius: f32) -> f32 {
    if radius <= 0.0 {
        return 1.0;
    }
    let ratio = distance / radius;
    let fade = (1.0 - ratio * ratio * ratio * ratio).clamp(0.0, 1.0);
    fade * fade
}

/// Physical inverse-square attenuation `1 / d²` with the cut-off window applied.
pub fn inverse_square(distance: f32, radius: f32) -> f32 {
    let d = distance.max(MIN_DISTANCE);
    distance_window(distance, radius) / (d * d)
}

/// Spot cone factor for the cosine between the beam axis and the light-to-point direction: 1 inside the inner cone,
/// 0 outside the outer cone and a smoothstep between them. Angles are full apertures in degrees.
pub fn spot_falloff(cos_angle: f32, inner_deg: f32, outer_deg: f32) -> f32 {
    let outer = outer_deg.clamp(0.1, 179.9);
    let inner = inner_deg.clamp(0.0, outer);
    let cos_outer = (outer.to_radians() * 0.5).cos();
    let cos_inner = (inner.to_radians() * 0.5).cos();
    if cos_inner - cos_outer <= 1e-6 {
        return if cos_angle >= cos_outer { 1.0 } else { 0.0 };
    }
    let t = ((cos_angle - cos_outer) / (cos_inner - cos_outer)).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

/// Peak luminous intensity in candela of a point or spot light without IES profile: the flux spread over the full
/// sphere (point) or over the outer cone (spot), as in common real-time engines.
pub fn peak_intensity(light: &Light) -> f32 {
    let lumens = light.power.lumens();
    match light.kind {
        LightKind::Spot { outer_deg, .. } => {
            let half = outer_deg.clamp(0.1, 179.9).to_radians() * 0.5;
            lumens / (2.0 * PI * (1.0 - half.cos()))
        }
        _ => lumens / (4.0 * PI),
    }
}

/// Luminous intensity in candela of a point or spot light toward `local_dir` (light frame, see
/// [`super::vector::Basis::to_local`]), including the cone and the IES profile, before dimming.
///
/// With an IES profile the file's candela values are scaled by `light lumens / rated lumens` of the file; files with
/// absolute photometry (rated lumens ≤ 0) use their shape over the isotropic intensity.
pub fn directional_intensity(light: &Light, local_dir: Vec3) -> f32 {
    let cone = match light.kind {
        LightKind::Spot {
            inner_deg,
            outer_deg,
        } => {
            let len = super::vector::length(local_dir);
            let cos = if len > 0.0 { local_dir[2] / len } else { 0.0 };
            spot_falloff(cos, inner_deg, outer_deg)
        }
        _ => 1.0,
    };
    if cone <= 0.0 {
        return 0.0;
    }
    let base = match &light.ies {
        Some(profile) => {
            let rated = profile.lumens_per_lamp * profile.lamps as f32;
            if rated > 0.0 {
                ies_intensity(profile, local_dir) * light.power.lumens() / rated
            } else {
                ies_shape(profile, local_dir) * light.power.lumens() / (4.0 * PI)
            }
        }
        None => peak_intensity(light),
    };
    base * cone
}

/// Black-body color of `kelvin` scaled to unit luminance, so `color * lux` keeps the photometric magnitude.
pub fn photometric_color(kelvin: f32) -> Rgb {
    let color = kelvin_to_rgb(kelvin);
    let luminance = color.luminance();
    if luminance > 0.0 {
        color * (1.0 / luminance)
    } else {
        Rgb::WHITE
    }
}

/// Light color: [`photometric_color`] of its temperature times its filter.
pub fn light_color(light: &Light) -> Rgb {
    photometric_color(light.kelvin).tint(light.filter)
}

/// Dimmer clamped to `0..=1`.
pub fn dimmer(light: &Light) -> f32 {
    light.dimmer.clamp(0.0, 1.0)
}
