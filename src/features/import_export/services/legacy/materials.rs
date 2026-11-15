//! The materials of the imported scene: one per surface category in use, the textured materials of embedded
//! images, and the per-element render material assignments (`RenderMaterials`) of the document.

use super::json::{int, list, number, text};
use super::part::Key;
use super::surface::Surface;
use crate::features::import_export::models::Material;
use crate::features::materials::models::{Material as Pbr, Procedural, ProceduralKind, TextureRef};
use serde_json::Value;
use std::collections::HashMap;

/// The materials of the scene being built, shared by every element.
#[derive(Debug, Default)]
pub struct MaterialBook {
    materials: Vec<Material>,
    by_surface: HashMap<Surface, usize>,
    by_name: HashMap<String, usize>,
    overrides: HashMap<(i64, String), usize>,
}

/// The import material that carries `pbr`: its summary color plus the full material.
pub fn scene_material(pbr: Pbr) -> Material {
    let c = pbr.base_color;
    Material {
        name: pbr.name.clone(),
        color: [c[0], c[1], c[2], c[3] * (1.0 - pbr.transparency)],
        texture: None,
        pbr: Some(Box::new(pbr)),
    }
}

/// The procedural pattern a built-in texture name of Guanacaste 3D stands for, with its tint.
fn builtin_pattern(name: &str) -> Option<(ProceduralKind, [f32; 4])> {
    let name = name.to_lowercase();
    let white = [1.0, 1.0, 1.0, 1.0];
    if name.contains("brick") {
        // Weathered clay: street pavers and exposed brick carry dust and wear, never the pure kiln color.
        // Fired clay reflects about a third of the light; the pattern's pale mortar would otherwise read as salmon.
        Some((ProceduralKind::Brick, [0.74, 0.64, 0.60, 1.0]))
    } else if name.contains("wood") || name.contains("timber") || name.contains("plank") {
        Some((ProceduralKind::Wood, [1.0, 0.92, 0.82, 1.0]))
    } else if name.contains("marble") || name.contains("tile") {
        Some((ProceduralKind::Marble, white))
    } else if name.contains("grass") || name.contains("lawn") {
        Some((ProceduralKind::Grass, white))
    } else if name.contains("asphalt") {
        Some((ProceduralKind::Noise, [0.62, 0.62, 0.64, 1.0]))
    } else if name.contains("concrete") || name.contains("stone") || name.contains("plaster") {
        Some((ProceduralKind::Concrete, white))
    } else {
        None
    }
}

impl MaterialBook {
    /// The materials registered so far.
    pub fn into_materials(self) -> Vec<Material> {
        self.materials
    }

    /// Registers `material` and returns its index.
    pub fn register(&mut self, material: Pbr) -> usize {
        self.materials.push(scene_material(material));
        self.materials.len() - 1
    }

    /// The material of a surface category, created on first use.
    pub fn surface(&mut self, surface: Surface) -> usize {
        if let Some(index) = self.by_surface.get(&surface) {
            return *index;
        }
        let index = self.register(surface.material());
        self.by_surface.insert(surface, index);
        index
    }

    /// The material registered under `name`, created with `make` on first use.
    pub fn named(&mut self, name: &str, make: impl FnOnce() -> Pbr) -> usize {
        if let Some(index) = self.by_name.get(name) {
            return *index;
        }
        let index = self.register(make());
        self.by_name.insert(name.to_owned(), index);
        index
    }

    /// The scene material index of a key.
    pub fn resolve(&mut self, key: Key) -> usize {
        match key {
            Key::Surface(surface) => self.surface(surface),
            Key::Custom(index) => index,
        }
    }

    /// Reads the `RenderMaterials` of the document: each assigns a texture to one surface category of one element.
    pub fn read_overrides(&mut self, document: &Value) {
        for entry in list(document, "RenderMaterials") {
            let (category, element) = (int(entry, "Category"), text(entry, "ElementId"));
            let Some(surface) = Surface::from_category(category) else {
                continue;
            };
            if element.is_empty() {
                continue;
            }
            let path = text(entry, "TexturePath");
            let mut material = surface.material();
            material.name = format!(
                "{} ({})",
                surface.name(),
                path.trim_start_matches("builtin:")
            );
            let pattern = path
                .strip_prefix("builtin:")
                .and_then(builtin_pattern)
                .or_else(|| builtin_pattern(path));
            if let Some((kind, tint)) = pattern {
                let mut procedural = Procedural::new(kind);
                procedural.seed = 11;
                material.maps.albedo = Some(TextureRef::procedural(procedural));
                material.base_color = tint;
                let size = number(entry, "TextureSize");
                let tile = if size > 0.0 { size as f32 } else { 1.0 };
                // `TextureSize` scales the pattern (1 = natural size). The procedural brick holds 4 × 8 bricks per
                // tile, so a 0.9 × 0.6 m tile gives real 225 × 75 mm bricks or pavers; other patterns repeat
                // every 2 m.
                material.uv.scale = if kind == ProceduralKind::Brick {
                    [1.0 / (0.9 * tile), 1.0 / (0.6 * tile)]
                } else {
                    [1.0 / (tile * 2.0), 1.0 / (tile * 2.0)]
                };
                let relief = number(entry, "Relief") as f32;
                if relief > 0.0 {
                    let mut bump = TextureRef::procedural(procedural);
                    bump.is_height = true;
                    material.maps.normal = Some(bump);
                    material.normal_strength = (relief * 1.5).clamp(0.0, 2.0);
                }
            }
            for (key, target) in [("RoughnessOverride", 0), ("MetallicOverride", 1)] {
                if let Some(value) = entry.get(key).and_then(Value::as_f64) {
                    if target == 0 {
                        material.roughness = value as f32;
                    } else {
                        material.metallic = value as f32;
                    }
                }
            }
            let index = self.register(material);
            self.overrides.insert((category, element.to_owned()), index);
        }
    }

    /// The overrides of the element with `id`: `(replaced surface, material index)` pairs.
    pub fn overrides_of(&self, id: &str) -> Vec<(Surface, usize)> {
        self.overrides
            .iter()
            .filter(|((_, element), _)| element == id)
            .filter_map(|((category, _), index)| Some((Surface::from_category(*category)?, *index)))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn surface_materials_are_created_once() {
        let mut book = MaterialBook::default();
        let a = book.surface(Surface::Wall);
        let b = book.surface(Surface::Glass);
        assert_eq!(book.surface(Surface::Wall), a);
        assert_ne!(a, b);
        assert_eq!(book.into_materials().len(), 2);
    }

    #[test]
    fn a_builtin_texture_assignment_overrides_one_surface_of_one_element() {
        let document: Value = serde_json::from_str(
            r#"{"RenderMaterials":[{"Category":0,"ElementId":"w1","TexturePath":"builtin:brick-red","TextureSize":1,"Relief":0.6}]}"#,
        )
        .unwrap();
        let mut book = MaterialBook::default();
        book.read_overrides(&document);
        let found = book.overrides_of("w1");
        assert_eq!(found.len(), 1);
        assert_eq!(found[0].0, Surface::Wall);
        assert!(book.overrides_of("other").is_empty());
        let material = book.into_materials().remove(found[0].1);
        let pbr = material.pbr.expect("full material");
        assert!(pbr.maps.albedo.is_some() && pbr.maps.normal.is_some());
    }
}
