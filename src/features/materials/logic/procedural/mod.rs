//! CPU procedural textures: brick, wood, concrete, marble, grass and noise. Every pattern tiles seamlessly.

mod architecture;
mod noise;
mod patterns;

use crate::features::materials::logic::color::{linear_to_srgb, luminance, srgb_to_linear};
use crate::features::materials::models::{Procedural, ProceduralKind};
use crate::services::texture_cache::Bitmap;

pub use noise::{fbm, value_noise};

/// Side in pixels of a baked procedural texture.
pub const BAKE_SIZE: usize = 512;

/// The color (sRGB, 0..1) of the pattern at `(u, v)`; the pattern repeats every 1.0 in both directions.
pub fn color_at(procedural: &Procedural, u: f32, v: f32) -> [f32; 3] {
    let n = procedural.scale_cells();
    let seed = procedural.seed;
    let rgb = match procedural.kind {
        ProceduralKind::Brick => patterns::brick(u, v, n, seed),
        ProceduralKind::Wood => patterns::wood(u, v, n, seed),
        ProceduralKind::Concrete => patterns::concrete(u, v, n, seed),
        ProceduralKind::Marble => patterns::marble(u, v, n, seed),
        ProceduralKind::Grass => patterns::grass(u, v, n, seed),
        ProceduralKind::Noise => patterns::noise(u, v, n, seed),
        ProceduralKind::Stucco => architecture::stucco(u, v, n, seed),
        ProceduralKind::RoofTile => architecture::roof_tile(u, v, n, seed),
        ProceduralKind::StoneBlock => architecture::stone_block(u, v, n, seed),
        ProceduralKind::Lawn => architecture::lawn(u, v, n, seed),
    };
    rgb.map(|c| c.clamp(0.0, 1.0))
}

impl Procedural {
    /// The scale as a whole number of pattern repeats (1..=8), which keeps the tile seamless.
    pub fn scale_cells(&self) -> i32 {
        if self.scale.is_finite() {
            self.scale.round().clamp(1.0, 8.0) as i32
        } else {
            1
        }
    }
}

/// Renders the pattern into a `size` x `size` bitmap. With `grayscale` the result is the pattern's luminance
/// (computed in linear light and re-encoded), used when the pattern drives roughness, metalness, AO or relief.
pub fn bake(procedural: &Procedural, size: usize, grayscale: bool) -> Bitmap {
    let size = size.max(1);
    let mut pixels = Vec::with_capacity(size * size);
    for y in 0..size {
        for x in 0..size {
            let rgb = color_at(
                procedural,
                (x as f32 + 0.5) / size as f32,
                (y as f32 + 0.5) / size as f32,
            );
            let rgb = if grayscale {
                let l = luminance(rgb.map(srgb_to_linear));
                [linear_to_srgb(l); 3]
            } else {
                rgb
            };
            let byte = |c: f32| (c * 255.0 + 0.5) as u8;
            pixels.push([byte(rgb[0]), byte(rgb[1]), byte(rgb[2]), 255]);
        }
    }
    Bitmap {
        width: size,
        height: size,
        pixels,
    }
}

#[cfg(test)]
mod tests;
