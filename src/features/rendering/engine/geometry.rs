//! The renderer's view of an imported scene: triangles in a BVH plus linear-color materials.

use super::bvh::{Bvh, Triangle};
use super::types::RenderMaterial;
use super::vec3::Vec3;
use crate::features::import_export::models::ImportedScene;

/// A scene prepared for ray tracing.
#[derive(Debug, Default)]
pub struct RenderScene {
    /// Triangles and acceleration structure.
    pub bvh: Bvh,
    /// Materials, indexed like the scene's, plus a final default entry for meshes without one.
    pub materials: Vec<RenderMaterial>,
}

fn srgb_to_linear(value: f32) -> f32 {
    let v = value.clamp(0.0, 1.0);
    if v <= 0.04045 {
        v / 12.92
    } else {
        ((v + 0.055) / 1.055).powf(2.4)
    }
}

/// The render material that corresponds to an imported material color (sRGB with alpha).
pub fn material_from_color(color: [f32; 4]) -> RenderMaterial {
    let alpha = color[3].clamp(0.0, 1.0);
    RenderMaterial {
        color: [
            srgb_to_linear(color[0]),
            srgb_to_linear(color[1]),
            srgb_to_linear(color[2]),
        ],
        roughness: if alpha < 0.99 { 0.05 } else { 0.7 },
        metallic: 0.0,
        transparency: 1.0 - alpha,
    }
}

impl RenderScene {
    /// Converts the meshes of `scene` (already in world meters) and builds the BVH. Triangles with invalid
    /// indices, non-finite corners or no area are skipped.
    pub fn new(scene: &ImportedScene) -> Self {
        let mut materials: Vec<RenderMaterial> = scene
            .materials
            .iter()
            .map(|m| material_from_color(m.color))
            .collect();
        let default_material = materials.len() as u32;
        materials.push(RenderMaterial::default());
        let mut triangles = Vec::with_capacity(scene.triangle_count());
        for mesh in &scene.meshes {
            let material = mesh
                .material
                .filter(|m| *m < default_material as usize)
                .map_or(default_material, |m| m as u32);
            let corner = |index: u32| {
                mesh.positions
                    .get(index as usize)
                    .map(|p| Vec3::from_array(*p))
                    .filter(|v| v.is_finite())
            };
            for tri in mesh.indices.as_chunks::<3>().0 {
                let (Some(a), Some(b), Some(c)) = (corner(tri[0]), corner(tri[1]), corner(tri[2]))
                else {
                    continue;
                };
                let triangle = Triangle {
                    v0: a,
                    e1: b - a,
                    e2: c - a,
                    material,
                };
                if triangle.e1.cross(triangle.e2).length() > 1e-12 {
                    triangles.push(triangle);
                }
            }
        }
        Self {
            bvh: Bvh::build(triangles),
            materials,
        }
    }

    /// Number of triangles that will be traced.
    pub fn triangle_count(&self) -> usize {
        self.bvh.triangles.len()
    }
}
