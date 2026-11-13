//! Simple physically inspired shading: one key light, a sky/ground environment and GGX specular.

use super::vec3::{add, dot, lerp, mul, normalize, scale, V3};
use crate::features::materials::logic::sample::Surface;
use std::f32::consts::PI;

/// Direction towards the key light in view space (upper left, in front).
pub const LIGHT: V3 = [-0.45, 0.6, 0.66];
const LIGHT_INTENSITY: f32 = 2.6;
const VIEW: V3 = [0.0, 0.0, 1.0];

fn smoothstep(a: f32, b: f32, x: f32) -> f32 {
    let t = ((x - a) / (b - a)).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

/// Radiance of the environment looking along `direction`: a gradient from warm ground to blue sky with a bright
/// softbox where the key light is.
pub fn environment(direction: V3) -> V3 {
    let up = direction[1];
    let ground = [0.12, 0.11, 0.10];
    let horizon = [0.75, 0.78, 0.82];
    let zenith = [0.28, 0.45, 0.85];
    let sky = if up > 0.0 {
        lerp(horizon, zenith, smoothstep(0.0, 0.8, up))
    } else {
        lerp(horizon, ground, smoothstep(0.0, 0.25, -up))
    };
    let softbox = dot(normalize(direction), normalize(LIGHT))
        .max(0.0)
        .powi(48)
        * 9.0;
    add(sky, [softbox; 3])
}

/// The lit appearance of a surface point.
#[derive(Debug, Clone, Copy)]
pub struct Lit {
    /// Light scattered diffusely (already multiplied by the albedo).
    pub diffuse: V3,
    /// Specular reflection of the light and the environment.
    pub specular: V3,
    /// Emitted light.
    pub emission: V3,
}

/// Shades `surface` at a point with unit normal `n` (view space), seen from +Z.
pub fn shade(surface: &Surface, n: V3, ior: f32) -> Lit {
    let l = normalize(LIGHT);
    let nl = dot(n, l).max(0.0);
    let nv = dot(n, VIEW).max(0.001);
    let roughness = surface.roughness.max(0.03);
    let a = roughness * roughness;
    let f0_dielectric = ((ior - 1.0) / (ior + 1.0)).powi(2);
    let f0 = lerp([f0_dielectric; 3], surface.albedo, surface.metallic);

    let hemisphere = lerp(
        [0.14, 0.13, 0.12],
        [0.55, 0.62, 0.78],
        (n[1] * 0.5 + 0.5).clamp(0.0, 1.0),
    );
    let irradiance = add(hemisphere, [nl * LIGHT_INTENSITY; 3]);
    let diffuse_color = scale(surface.albedo, 1.0 - surface.metallic);
    let diffuse = scale(mul(diffuse_color, irradiance), surface.ao);

    let h = normalize(add(l, VIEW));
    let nh = dot(n, h).max(0.0);
    let vh = dot(VIEW, h).max(0.0);
    let d = a * a / (PI * (nh * nh * (a * a - 1.0) + 1.0).powi(2));
    let k = a / 2.0;
    let g = (nl / (nl * (1.0 - k) + k)) * (nv / (nv * (1.0 - k) + k));
    let fresnel_direct = add(
        f0,
        scale(add([1.0; 3], scale(f0, -1.0)), (1.0 - vh).powi(5)),
    );
    let direct = scale(
        fresnel_direct,
        d * g / (4.0 * nl.max(0.01) * nv) * nl * LIGHT_INTENSITY,
    );

    let reflected = normalize(add(scale(n, 2.0 * dot(n, VIEW)), scale(VIEW, -1.0)));
    let sharp = environment(reflected);
    let rough_blend = smoothstep(0.1, 0.85, roughness);
    let blurred = lerp(sharp, mul(hemisphere, [1.4; 3]), rough_blend);
    let grazing = (1.0 - roughness).max(f0_dielectric);
    let edge = (1.0 - nv).powi(5);
    let fresnel_env = add(f0, scale(add([grazing; 3], scale(f0, -1.0)), edge));
    let ambient = scale(mul(fresnel_env, blurred), surface.ao);
    Lit {
        diffuse,
        specular: add(direct, ambient),
        emission: surface.emission,
    }
}
