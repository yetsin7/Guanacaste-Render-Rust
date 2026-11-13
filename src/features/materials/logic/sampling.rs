//! Low-level texture lookups: UV transform, wrapping and trilinear filtering.

use super::color::srgb_table;
use crate::features::materials::models::{UvTransform, Wrap};
use crate::services::texture_cache::MipChain;

/// Applies the material's UV mapping: scale and rotation about the center, then the offset.
pub fn transform_uv(uv: &UvTransform, point: [f32; 2]) -> [f32; 2] {
    let x = (point[0] - 0.5) * uv.scale[0];
    let y = (point[1] - 0.5) * uv.scale[1];
    let (sin, cos) = uv.rotation.to_radians().sin_cos();
    [
        x * cos - y * sin + 0.5 + uv.offset[0],
        x * sin + y * cos + 0.5 + uv.offset[1],
    ]
}

/// Maps a texel index that may be outside `0..size` into range according to `wrap`.
pub fn wrap_index(index: i64, size: usize, wrap: Wrap) -> usize {
    let n = size as i64;
    match wrap {
        Wrap::Repeat => index.rem_euclid(n) as usize,
        Wrap::Mirror => {
            let i = index.rem_euclid(2 * n);
            (if i >= n { 2 * n - 1 - i } else { i }) as usize
        }
        Wrap::Clamp => index.clamp(0, n - 1) as usize,
    }
}

/// How the stored bytes become floats.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Decode {
    /// sRGB color to linear light (alpha stays linear).
    Srgb,
    /// Plain 0..1 values.
    Linear,
}

fn texel(chain: &MipChain, level: usize, x: i64, y: i64, wrap: Wrap, decode: Decode) -> [f32; 4] {
    let bitmap = &chain.levels[level];
    let p = bitmap.at(
        wrap_index(x, bitmap.width, wrap),
        wrap_index(y, bitmap.height, wrap),
    );
    let alpha = p[3] as f32 / 255.0;
    match decode {
        Decode::Srgb => {
            let table = srgb_table();
            [
                table[p[0] as usize],
                table[p[1] as usize],
                table[p[2] as usize],
                alpha,
            ]
        }
        Decode::Linear => [
            p[0] as f32 / 255.0,
            p[1] as f32 / 255.0,
            p[2] as f32 / 255.0,
            alpha,
        ],
    }
}

fn bilinear(chain: &MipChain, level: usize, uv: [f32; 2], wrap: Wrap, decode: Decode) -> [f32; 4] {
    let bitmap = &chain.levels[level];
    let x = uv[0] * bitmap.width as f32 - 0.5;
    let y = uv[1] * bitmap.height as f32 - 0.5;
    let (x0, y0) = (x.floor(), y.floor());
    let (fx, fy) = (x - x0, y - y0);
    let (ix, iy) = (x0.clamp(-1e9, 1e9) as i64, y0.clamp(-1e9, 1e9) as i64);
    let a = texel(chain, level, ix, iy, wrap, decode);
    let b = texel(chain, level, ix + 1, iy, wrap, decode);
    let c = texel(chain, level, ix, iy + 1, wrap, decode);
    let d = texel(chain, level, ix + 1, iy + 1, wrap, decode);
    std::array::from_fn(|k| {
        let top = a[k] + (b[k] - a[k]) * fx;
        let bottom = c[k] + (d[k] - c[k]) * fx;
        top + (bottom - top) * fy
    })
}

/// Trilinear lookup at `uv` (already transformed) with mip level `lod` (0 = full resolution). V grows downward
/// through the image rows. Non-finite coordinates read the center.
pub fn fetch(chain: &MipChain, uv: [f32; 2], lod: f32, wrap: Wrap, decode: Decode) -> [f32; 4] {
    let uv = if uv[0].is_finite() && uv[1].is_finite() {
        uv
    } else {
        [0.5, 0.5]
    };
    let top = (chain.levels.len() - 1) as f32;
    let lod = if lod.is_finite() {
        lod.clamp(0.0, top)
    } else {
        0.0
    };
    let low = lod.floor() as usize;
    let near = bilinear(chain, low, uv, wrap, decode);
    let fraction = lod - low as f32;
    if fraction < 1e-3 || low + 1 >= chain.levels.len() {
        return near;
    }
    let far = bilinear(chain, low + 1, uv, wrap, decode);
    std::array::from_fn(|k| near[k] + (far[k] - near[k]) * fraction)
}

/// Mip level for a pixel that covers `footprint` UV units of the untransformed surface, given the UV `scale`.
pub fn lod_for(chain: &MipChain, footprint: f32, scale: [f32; 2]) -> f32 {
    let (w, h) = chain.size();
    let texels = footprint * scale[0].abs().max(scale[1].abs()) * w.max(h) as f32;
    if texels > 1.0 {
        texels.log2()
    } else {
        0.0
    }
}
