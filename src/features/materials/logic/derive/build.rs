//! From a surface family to a PBR material: tint, roughness, glass, and the procedural texture maps with the size of
//! one texture tile in meters (meshes without UV coordinates are mapped in meters by the renderer).

use super::classify::classify;
use super::text::{mix, saturation};
use super::{SurfaceInfo, SurfaceKind};
use crate::features::materials::models::{
    Material, Procedural, ProceduralKind, TextureMaps, TextureRef, UvTransform,
};

/// Warm light render of the walls, whatever default color the authoring program gave them.
const STUCCO_TINT: [f32; 3] = [0.96, 0.89, 0.76];
/// Dark bronze aluminium of window frames.
const FRAME_TINT: [f32; 3] = [0.17, 0.15, 0.14];
/// Fired clay.
const TERRACOTTA: [f32; 3] = [0.80, 0.42, 0.27];

/// A texture pattern with the meters one tile covers, its relief strength and a seed.
struct Pattern {
    kind: ProceduralKind,
    tile_m: f32,
    relief: f32,
}

fn pattern(kind: SurfaceKind, class: &str) -> Option<Pattern> {
    let pat = |kind, tile_m, relief| {
        Some(Pattern {
            kind,
            tile_m,
            relief,
        })
    };
    match kind {
        SurfaceKind::Stucco => pat(ProceduralKind::Stucco, 1.5, 0.9),
        SurfaceKind::RoofTile => pat(ProceduralKind::RoofTile, 1.2, 1.4),
        SurfaceKind::Wood => pat(ProceduralKind::Wood, 1.2, 0.35),
        SurfaceKind::Stone => pat(ProceduralKind::StoneBlock, 1.8, 1.2),
        SurfaceKind::Brick => pat(ProceduralKind::Brick, 1.0, 1.0),
        SurfaceKind::Concrete => pat(ProceduralKind::Concrete, 2.0, 0.5),
        SurfaceKind::Grass => pat(ProceduralKind::Lawn, 1.2, 0.5),
        SurfaceKind::Ceramic if matches!(class, "IFCSLAB" | "IFCCOVERING") => {
            pat(ProceduralKind::StoneBlock, 3.0, 0.5)
        }
        SurfaceKind::Ceramic if class.starts_with("IFCWALL") => {
            pat(ProceduralKind::StoneBlock, 2.4, 0.4)
        }
        _ => None,
    }
}

/// The base color factor that tints the pattern of `kind`; `rgb` is the style color of the file.
fn tint(kind: SurfaceKind, rgb: [f32; 3], class: &str) -> [f32; 3] {
    match kind {
        SurfaceKind::Stucco => STUCCO_TINT,
        SurfaceKind::Plaster => mix(rgb, [0.95, 0.94, 0.91], 0.8),
        SurfaceKind::RoofTile => TERRACOTTA,
        SurfaceKind::Wood => mix([1.0, 0.95, 0.9], rgb.map(|c| (c / 0.7).min(1.0)), 0.6),
        SurfaceKind::Stone => mix(rgb, [0.90, 0.82, 0.68], 0.65),
        SurfaceKind::Brick => [1.0, 1.0, 1.0],
        SurfaceKind::Concrete => mix(rgb, [0.80, 0.79, 0.77], 0.5),
        SurfaceKind::Metal if saturation(rgb) > 0.25 => [0.30, 0.31, 0.33],
        SurfaceKind::Grass => [0.86, 0.88, 0.78],
        SurfaceKind::Paint | SurfaceKind::Plastic | SurfaceKind::Metal if class == "IFCWINDOW" => {
            FRAME_TINT
        }
        _ => rgb,
    }
}

fn roughness_and_metal(kind: SurfaceKind, class: &str) -> (f32, f32) {
    match kind {
        SurfaceKind::Stucco => (0.95, 0.0),
        SurfaceKind::Plaster => (0.85, 0.0),
        SurfaceKind::Paint if class == "IFCWINDOW" => (0.45, 0.6),
        SurfaceKind::Paint => (0.55, 0.0),
        SurfaceKind::RoofTile => (0.72, 0.0),
        SurfaceKind::Wood => (0.6, 0.0),
        SurfaceKind::Stone => (0.85, 0.0),
        SurfaceKind::Brick | SurfaceKind::Concrete => (0.9, 0.0),
        SurfaceKind::Metal => (0.38, 0.9),
        SurfaceKind::Glass => (0.03, 0.0),
        SurfaceKind::Ceramic => (0.22, 0.0),
        SurfaceKind::Grass | SurfaceKind::Fabric => (1.0, 0.0),
        SurfaceKind::Plastic if class == "IFCWINDOW" => (0.45, 0.6),
        SurfaceKind::Plastic => (0.42, 0.0),
        SurfaceKind::Soil | SurfaceKind::Hidden => (1.0, 0.0),
    }
}

/// The glass tint and how much light passes: the default blue of Revit becomes a pale blue-green, black glass
/// stays dark.
fn glass(rgb: [f32; 3]) -> ([f32; 3], f32) {
    let dark = rgb[0] + rgb[1] + rgb[2] < 0.2;
    if dark {
        ([0.10, 0.12, 0.13], 0.7)
    } else if rgb[2] > 0.7 && rgb[0] < 0.2 {
        ([0.72, 0.87, 0.90], 0.9)
    } else {
        (mix(rgb, [0.78, 0.90, 0.92], 0.6), 0.9)
    }
}

/// The PBR material of the surface `info` describes.
pub fn derive(info: &SurfaceInfo) -> Material {
    let kind = classify(info);
    let class = info.class.to_ascii_uppercase();
    let rgb = [info.color[0], info.color[1], info.color[2]];
    let (roughness, metallic) = roughness_and_metal(kind, &class);
    let mut material = Material::new(info.name);
    material.roughness = roughness;
    material.metallic = metallic;
    material.base_color = {
        let c = tint(kind, rgb, &class);
        [c[0], c[1], c[2], 1.0]
    };
    match kind {
        SurfaceKind::Glass => {
            let (color, transparency) = glass(rgb);
            material.base_color = [color[0], color[1], color[2], 1.0];
            material.transparency = transparency;
        }
        SurfaceKind::Soil | SurfaceKind::Hidden => material.transparency = 1.0,
        _ => {}
    }
    if let Some(pattern) = pattern(kind, &class) {
        let texture = TextureRef::procedural(Procedural::new(pattern.kind));
        material.maps = TextureMaps {
            albedo: Some(texture.clone()),
            normal: Some(texture),
            ..TextureMaps::default()
        };
        material.normal_strength = pattern.relief;
        material.uv = UvTransform {
            scale: [1.0 / pattern.tile_m; 2],
            ..UvTransform::default()
        };
    }
    material
}
