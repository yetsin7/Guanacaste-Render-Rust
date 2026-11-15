//! Material catalog exported from Guanacaste 3D, translated to Render's PBR representation.

use crate::features::materials::models::{Material, Procedural, ProceduralKind, TextureRef};
use serde::Deserialize;
use std::sync::OnceLock;

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct Entry {
    id: String,
    name: String,
    color: [f32; 3],
    roughness: f32,
    metallic: f32,
    ior: f32,
    transmission: f32,
    transmission_tint: [f32; 3],
    pattern: String,
    pattern_scale: f32,
    texture: Option<String>,
    size: f32,
    relief: f32,
}

/// Loads the immutable catalog once and keeps the original project identifiers stable.
fn entries() -> &'static [Entry] {
    static CATALOG: OnceLock<Vec<Entry>> = OnceLock::new();
    CATALOG.get_or_init(|| {
        serde_json::from_str(include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/assets/materials/guanacaste-3d-catalog.json"
        )))
        .expect("Bundled material catalog must be valid")
    })
}

/// Converts the original linear reflectance, texture and supported pattern into an editable material.
fn convert(entry: &Entry) -> Material {
    let mut material = Material::new(&entry.name);
    let color = if entry.transmission > 0.0 {
        entry.transmission_tint
    } else {
        entry.color
    }
    .map(super::color::linear_to_srgb);
    material.base_color = [color[0], color[1], color[2], 1.0];
    material.roughness = entry.roughness;
    material.metallic = entry.metallic;
    material.ior = entry.ior;
    material.transparency = entry.transmission;
    if let Some(key) = entry
        .texture
        .as_deref()
        .filter(|key| super::bundled_textures::image(key).is_some())
    {
        let path = format!("builtin:{key}");
        material.maps.albedo = Some(TextureRef::file(&path));
        material.base_color = [1.0; 4];
        material.uv.scale = [1.0 / entry.size.max(0.01); 2];
        if entry.relief > 0.0 {
            let mut bump = TextureRef::file(path);
            bump.is_height = true;
            material.maps.normal = Some(bump);
            material.normal_strength = entry.relief;
        }
    } else {
        let pattern = match entry.pattern.as_str() {
            "Grass" | "Leaves" | "Foliage" => Some(ProceduralKind::Grass),
            "Blocks" => Some(ProceduralKind::Brick),
            "Planks" | "Bark" => Some(ProceduralKind::Wood),
            "RoofTiles" => Some(ProceduralKind::RoofTile),
            "StoneMasonry" => Some(ProceduralKind::StoneBlock),
            "Tiles" => Some(ProceduralKind::Marble),
            "Concrete" | "PouredConcrete" | "Asphalt" => Some(ProceduralKind::Concrete),
            "None" => None,
            _ => Some(ProceduralKind::Noise),
        };
        if let Some(kind) = pattern {
            let procedural = Procedural::new(kind);
            material.maps.albedo = Some(TextureRef::procedural(procedural));
            material.uv.scale = [1.0 / entry.pattern_scale.max(0.01); 2];
        }
    }
    material.clamped()
}

/// Material for a persisted `material:` identifier, or `None` for an unknown identifier.
pub fn find(identifier: &str) -> Option<Material> {
    entries()
        .iter()
        .find(|entry| entry.id == identifier)
        .map(convert)
}

/// All original materials, ready to add to the material picker.
pub fn all() -> Vec<Material> {
    entries().iter().map(convert).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The migrated catalog keeps stable identifiers and both image-backed and solid materials.
    #[test]
    fn catalog_contains_textured_and_untextured_materials() {
        assert!(all().len() > 100);
        assert!(find("brick-red").unwrap().maps.albedo.is_some());
        assert!(find("paint-white").is_some());
        assert!(find("missing-id").is_none());
    }
}
