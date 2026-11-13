//! Evaluating a material at a surface point. These are pure functions of the material, its resolved textures and the
//! UV coordinates, so a CPU renderer can call them from any thread.

use super::color::srgb_to_linear;
use super::maps::MaterialMaps;
use super::sampling::{fetch, lod_for, transform_uv, Decode};
use crate::features::materials::models::Material;
use crate::services::texture_cache::MipChain;

/// Linear (not sRGB) RGBA, each channel 0..1 for color; alpha 1 is opaque.
pub type Rgba = [f32; 4];

/// Every channel of a material at one point, in linear units.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Surface {
    /// Base color in linear light.
    pub albedo: [f32; 3],
    /// Opacity: `(1 - transparency)` times the base alpha times the albedo map alpha.
    pub alpha: f32,
    /// Unit tangent-space normal, +Z out of the surface, +Y along V's opposite (OpenGL convention).
    pub normal: [f32; 3],
    /// Perceptual roughness 0..1.
    pub roughness: f32,
    /// Metalness 0..1.
    pub metallic: f32,
    /// Ambient occlusion 0..1 (1 = unoccluded).
    pub ao: f32,
    /// Emitted radiance color in linear light (color * strength * map).
    pub emission: [f32; 3],
}

/// The linear base color and opacity of `material` at `uv` (full-resolution texture lookup). This is the albedo
/// the renderer multiplies with lighting; use [`surface`] for the remaining channels.
pub fn sample(material: &Material, maps: &MaterialMaps, uv: [f32; 2]) -> Rgba {
    let s = surface(material, maps, uv, 0.0);
    [s.albedo[0], s.albedo[1], s.albedo[2], s.alpha]
}

/// Every channel of `material` at `uv`. `footprint` is the UV size of one output pixel (0 for the sharpest
/// lookup); it selects the mip level so tiled textures do not shimmer.
pub fn surface(material: &Material, maps: &MaterialMaps, uv: [f32; 2], footprint: f32) -> Surface {
    let wrap = material.uv.wrap;
    let point = transform_uv(&material.uv, uv);
    let read = |chain: &MipChain, decode: Decode| {
        let lod = lod_for(chain, footprint, material.uv.scale);
        fetch(chain, point, lod, wrap, decode)
    };
    let factor = material.base_color.map(srgb_to_linear);
    let mut albedo = [factor[0], factor[1], factor[2]];
    let mut alpha = (1.0 - material.transparency) * material.base_color[3];
    if let Some(chain) = &maps.albedo {
        let t = read(chain, Decode::Srgb);
        for (a, t) in albedo.iter_mut().zip(t) {
            *a *= t;
        }
        alpha *= t[3];
    }
    let scalar = |chain: &Option<std::sync::Arc<MipChain>>, factor: f32| match chain {
        Some(chain) => factor * read(chain, Decode::Linear)[0],
        None => factor,
    };
    let roughness = scalar(&maps.roughness, material.roughness).clamp(0.0, 1.0);
    let metallic = scalar(&maps.metallic, material.metallic).clamp(0.0, 1.0);
    let ao = scalar(&maps.ao, 1.0).clamp(0.0, 1.0);
    let normal = match &maps.normal {
        Some(chain) => {
            let t = read(chain, Decode::Linear);
            let n = [
                (t[0] * 2.0 - 1.0) * material.normal_strength,
                (t[1] * 2.0 - 1.0) * material.normal_strength,
                (t[2] * 2.0 - 1.0).max(0.05),
            ];
            let length = (n[0] * n[0] + n[1] * n[1] + n[2] * n[2]).sqrt();
            [n[0] / length, n[1] / length, n[2] / length]
        }
        None => [0.0, 0.0, 1.0],
    };
    let mut emission = material.emission.map(srgb_to_linear);
    if let Some(chain) = &maps.emission {
        let t = read(chain, Decode::Srgb);
        for (e, t) in emission.iter_mut().zip(t) {
            *e *= t;
        }
    }
    let emission = emission.map(|e| e * material.emission_strength);
    Surface {
        albedo,
        alpha: alpha.clamp(0.0, 1.0),
        normal,
        roughness,
        metallic,
        ao,
        emission,
    }
}
