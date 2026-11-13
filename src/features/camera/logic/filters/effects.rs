//! Lens and film effects: vignette and grain.

use crate::features::camera::models::image_buffer::{luminance, ImageBuffer};

fn smoothstep(edge0: f32, edge1: f32, x: f32) -> f32 {
    let t = ((x - edge0) / (edge1 - edge0).max(1e-4)).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

/// Darkens toward the corners. The distance to the center is normalized so the corners are 1; light is
/// multiplied by `1 - strength * smoothstep(radius, radius + softness, distance)`, so the center is untouched.
pub fn vignette(image: &mut ImageBuffer, strength: f32, radius: f32, softness: f32) {
    let (width, height) = (image.width() as f32, image.height() as f32);
    let (cx, cy) = ((width - 1.0) / 2.0, (height - 1.0) / 2.0);
    image.map_rgb_at(|x, y, c| {
        let nx = if cx > 0.0 { (x as f32 - cx) / cx } else { 0.0 };
        let ny = if cy > 0.0 { (y as f32 - cy) / cy } else { 0.0 };
        let distance = (nx * nx + ny * ny).sqrt() / std::f32::consts::SQRT_2;
        let factor = 1.0 - strength * smoothstep(radius, radius + softness, distance);
        [c[0] * factor, c[1] * factor, c[2] * factor]
    });
}

/// Deterministic noise in -1..=1 for a pixel and a seed.
fn noise(x: usize, y: usize, seed: u32) -> f32 {
    let mut h = (x as u32).wrapping_mul(0x9E37_79B1)
        ^ (y as u32).wrapping_mul(0x85EB_CA77)
        ^ seed.wrapping_mul(0xC2B2_AE3D);
    h ^= h >> 15;
    h = h.wrapping_mul(0x2C1B_3C6D);
    h ^= h >> 12;
    h = h.wrapping_mul(0x297A_2D39);
    h ^= h >> 15;
    (h >> 8) as f32 / 8_388_607.5 - 1.0
}

/// Film grain: monochrome noise, strongest in the midtones, added to the light. The same `seed` always gives the
/// same grain; change it per frame for video.
pub fn grain(image: &mut ImageBuffer, amount: f32, seed: u32) {
    image.map_rgb_at(|x, y, c| {
        let n = noise(x, y, seed) * amount * 0.2 * (luminance(c).clamp(0.0, 1.0) + 0.05).sqrt();
        [c[0] + n, c[1] + n, c[2] + n]
    });
}
