//! What the renderer draws: the merged scene geometry plus the full material set that goes with it.

use crate::features::import_export::models::{Bounds, ImportedScene};
use crate::features::lighting::models::EmissiveMaterial;
use crate::features::materials::logic::convert;
use crate::features::materials::models::{Material, TextureRef};

/// A scene ready to render: meshes, PBR materials (one per scene material) and emissive overrides.
#[derive(Debug, Clone, PartialEq)]
pub struct World {
    /// Geometry in world meters, Z up. Mesh materials index `pbr`.
    pub scene: ImportedScene,
    /// One PBR material per `scene.materials` entry.
    pub pbr: Vec<Material>,
    /// Materials of the lighting domain that glow, matched to `pbr` by name.
    pub emissive: Vec<EmissiveMaterial>,
    /// Bounds of the model itself (without the ground), used to frame the preset views.
    pub focus: Option<Bounds>,
}

impl World {
    /// A world whose materials are derived from the colors (and diffuse textures) of `scene`: opaque surfaces
    /// are fairly matte, translucent ones glossy like glass.
    pub fn new(scene: ImportedScene) -> Self {
        let pbr = scene
            .materials
            .iter()
            .map(|m| {
                let mut material = convert::from_scene(m);
                if m.pbr.is_none() {
                    material.roughness = if m.color[3] < 0.99 { 0.05 } else { 0.7 };
                }
                if let Some(path) = &m.texture {
                    material.maps.albedo = Some(TextureRef::file(path.clone()));
                }
                material
            })
            .collect();
        Self {
            focus: scene.bounds(),
            scene,
            pbr,
            emissive: Vec::new(),
        }
    }
}

impl From<ImportedScene> for World {
    fn from(scene: ImportedScene) -> Self {
        Self::new(scene)
    }
}
