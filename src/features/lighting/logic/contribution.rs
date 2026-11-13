//! Direct lighting for the CPU renderer.
//!
//! Every function returns **illuminance in lux per color channel** at a surface point: the light's color, scaled to
//! unit luminance, times lux (so the luminance of the result is the illuminance in lux). A Lambertian surface with albedo `ρ` reflects radiance `ρ · E / π`. Shadows are the renderer's job: it
//! traces a ray from the point to each [`LightSample::position`] (when `light.cast_shadows`) and keeps the samples
//! that are visible.

use super::photometry::{dimmer, directional_intensity, inverse_square, light_color};
use super::vector::{add, basis, dot, length, scale, sub};
use crate::features::lighting::models::{EmissiveMaterial, Light, LightKind, Rgb, Vec3};
use std::f32::consts::PI;

/// Samples per axis used by [`light_contribution`] for area lights (squared) and linear lights (doubled).
pub const DEFAULT_SAMPLES_PER_AXIS: u32 = 4;

/// One point on a light and what it adds to the shaded point when nothing blocks it.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LightSample {
    /// Point on the light, meters (target of the shadow ray).
    pub position: Vec3,
    /// Illuminance it adds when visible, lux per channel.
    pub irradiance: Rgb,
}

/// A point on the surface of a light.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct EmitterPoint {
    /// Position, meters.
    pub position: Vec3,
    /// Emission axis of the surface (the light's forward axis).
    pub normal: Vec3,
    /// Probability density per square meter (area lights) or per meter (linear lights) of uniform sampling; `1`
    /// for point and spot lights.
    pub pdf: f32,
}

/// Maps `(u, v)` in `[0, 1)²` uniformly onto the light: the rectangle of an area light, the segment of a linear
/// light (only `u` is used), or the position itself for point and spot lights. For stochastic path tracing.
pub fn sample_emitter(light: &Light, u: f32, v: f32) -> EmitterPoint {
    let frame = basis(light.orientation);
    match light.kind {
        LightKind::Area { width, height, .. } => {
            let (w, h) = (width.max(1e-4), height.max(1e-4));
            let offset = add(
                scale(frame.right, (u - 0.5) * w),
                scale(frame.up, (v - 0.5) * h),
            );
            EmitterPoint {
                position: add(light.position, offset),
                normal: frame.forward,
                pdf: 1.0 / (w * h),
            }
        }
        LightKind::Linear { length } => {
            let l = length.max(1e-4);
            EmitterPoint {
                position: add(light.position, scale(frame.right, (u - 0.5) * l)),
                normal: frame.forward,
                pdf: 1.0 / l,
            }
        }
        LightKind::Point | LightKind::Spot { .. } => EmitterPoint {
            position: light.position,
            normal: frame.forward,
            pdf: 1.0,
        },
    }
}

/// Cosine between the surface normal and the direction to the light, or 0 when the light is behind the surface.
fn receiver_cosine(normal: Vec3, to_light: Vec3, distance: f32) -> f32 {
    if distance <= 0.0 {
        return 0.0;
    }
    (dot(normal, to_light) / distance).max(0.0)
}

/// Illuminance from an isotropic-or-directional point emitter at `origin` with `intensity_at(local_dir)` candela.
fn point_sample(
    light: &Light,
    origin: Vec3,
    point: Vec3,
    normal: Vec3,
    intensity_at: impl Fn(Vec3) -> f32,
) -> Rgb {
    let to_light = sub(origin, point);
    let distance = length(to_light);
    let cos_s = receiver_cosine(normal, to_light, distance);
    if cos_s <= 0.0 {
        return Rgb::BLACK;
    }
    let local = basis(light.orientation).to_local(scale(to_light, -1.0));
    let lux = intensity_at(local) * cos_s * inverse_square(distance, light.cutoff_radius);
    light_color(light) * (lux * dimmer(light))
}

/// Stratified samples of `light` as seen from `point` with unit `normal`; `per_axis` samples per axis for area lights
/// (`per_axis²` in total) and `2 · per_axis` along linear lights; a single sample for point and spot lights.
/// Disabled lights give no samples.
pub fn light_samples(light: &Light, point: Vec3, normal: Vec3, per_axis: u32) -> Vec<LightSample> {
    if !light.enabled || dimmer(light) <= 0.0 {
        return Vec::new();
    }
    let n = per_axis.max(1);
    match light.kind {
        LightKind::Point | LightKind::Spot { .. } => {
            let irradiance = point_sample(light, light.position, point, normal, |dir| {
                directional_intensity(light, dir)
            });
            vec![LightSample {
                position: light.position,
                irradiance,
            }]
        }
        LightKind::Linear { .. } => {
            let count = 2 * n;
            let intensity = light.power.lumens() / (4.0 * PI * count as f32);
            (0..count)
                .map(|i| {
                    let at = sample_emitter(light, (i as f32 + 0.5) / count as f32, 0.5).position;
                    LightSample {
                        position: at,
                        irradiance: point_sample(light, at, point, normal, |_| intensity),
                    }
                })
                .collect()
        }
        LightKind::Area {
            width,
            height,
            two_sided,
        } => {
            let area = width.max(1e-4) * height.max(1e-4);
            let faces = if two_sided { 2.0 } else { 1.0 };
            let radiance = light.power.lumens() / (PI * area * faces);
            let patch = area / (n * n) as f32;
            let color = light_color(light) * dimmer(light);
            let mut out = Vec::with_capacity((n * n) as usize);
            for j in 0..n {
                for i in 0..n {
                    let u = (i as f32 + 0.5) / n as f32;
                    let v = (j as f32 + 0.5) / n as f32;
                    let emitter = sample_emitter(light, u, v);
                    let to_light = sub(emitter.position, point);
                    let distance = length(to_light);
                    let cos_s = receiver_cosine(normal, to_light, distance);
                    let mut cos_l = -dot(emitter.normal, to_light) / distance.max(1e-6);
                    if two_sided {
                        cos_l = cos_l.abs();
                    }
                    let lux = if cos_s > 0.0 && cos_l > 0.0 {
                        radiance
                            * patch
                            * cos_l
                            * cos_s
                            * inverse_square(distance, light.cutoff_radius)
                    } else {
                        0.0
                    };
                    out.push(LightSample {
                        position: emitter.position,
                        irradiance: color * lux,
                    });
                }
            }
            out
        }
    }
}

/// Unshadowed illuminance (lux per channel) that `light` casts on `point` with unit `normal`.
pub fn light_contribution(light: &Light, point: Vec3, normal: Vec3) -> Rgb {
    light_samples(light, point, normal, DEFAULT_SAMPLES_PER_AXIS)
        .into_iter()
        .fold(Rgb::BLACK, |sum, sample| sum + sample.irradiance)
}

/// Radiance (cd/m² per channel) emitted by surfaces with an emissive material, black when it is off.
pub fn emissive_radiance(material: &EmissiveMaterial) -> Rgb {
    if !material.enabled {
        return Rgb::BLACK;
    }
    super::photometry::photometric_color(material.kelvin) * material.luminance.max(0.0)
}
