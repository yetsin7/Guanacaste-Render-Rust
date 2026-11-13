//! Procedural cloud layer: a drifting fractal noise field projected on a flat cloud deck.

use super::math::{smoothstep, Vec3};
use crate::features::environment::models::CloudSettings;

/// Integer hash to 0..1 (deterministic, platform independent).
pub fn hash(x: i32, y: i32, z: i32) -> f32 {
    let mut h = (x as u32).wrapping_mul(0x8da6_b343)
        ^ (y as u32).wrapping_mul(0xd816_3841)
        ^ (z as u32).wrapping_mul(0xcb1a_b31f);
    h ^= h >> 13;
    h = h.wrapping_mul(0x85eb_ca6b);
    h ^= h >> 16;
    (h & 0x00ff_ffff) as f32 / 16_777_216.0
}

/// Smooth value noise over the plane, 0..1.
fn value_noise(x: f32, y: f32) -> f32 {
    let (xi, yi) = (x.floor(), y.floor());
    let (fx, fy) = (x - xi, y - yi);
    let (sx, sy) = (fx * fx * (3.0 - 2.0 * fx), fy * fy * (3.0 - 2.0 * fy));
    let (xi, yi) = (xi as i32, yi as i32);
    let a = hash(xi, yi, 7);
    let b = hash(xi + 1, yi, 7);
    let c = hash(xi, yi + 1, 7);
    let d = hash(xi + 1, yi + 1, 7);
    (a + (b - a) * sx) + ((c + (d - c) * sx) - (a + (b - a) * sx)) * sy
}

/// Fractal noise, roughly 0.1..0.9.
fn fbm(mut x: f32, mut y: f32) -> f32 {
    let (mut sum, mut amplitude, mut total) = (0.0, 0.5, 0.0);
    for _ in 0..5 {
        sum += value_noise(x, y) * amplitude;
        total += amplitude;
        amplitude *= 0.5;
        x = x * 2.03 + 17.1;
        y = y * 2.03 - 9.7;
    }
    sum / total
}

/// Cloud opacity (0..1) seen along `direction` (unit vector). `drift` is the accumulated wind displacement of
/// the deck in noise units. Directions at or below the horizon have no clouds.
pub fn cloud_alpha(direction: Vec3, clouds: &CloudSettings, drift: [f32; 2]) -> f32 {
    let up = direction[1];
    if up <= 0.0 || clouds.coverage <= 0.0 || clouds.density <= 0.0 {
        return 0.0;
    }
    let scale = 1.6 / (up + 0.14);
    let noise = fbm(
        direction[0] * scale + drift[0],
        direction[2] * scale + drift[1],
    );
    let threshold = 1.0 - clouds.coverage * 1.05;
    let shape = smoothstep(threshold - 0.04, threshold + 0.26, noise);
    shape * clouds.density * smoothstep(0.0, 0.14, up)
}
