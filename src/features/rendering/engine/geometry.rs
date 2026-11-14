//! The renderer's view of a world: triangles in a BVH, linear materials, texture maps and corner UVs.

use super::bvh::{Bvh, Triangle, NO_UV};
use super::portals::Portals;
use super::split::{split, Piece};
use super::types::{RenderMaterial, NO_MAPS};
use super::uvmap;
use super::vec3::Vec3;
use super::world::World;
use crate::features::lighting::logic::emissive_radiance;
use crate::features::materials::logic::color::srgb_to_linear;
use crate::features::materials::logic::convert;
use crate::features::materials::models::Material;
use crate::features::materials::MaterialMaps;
use crate::services::texture_cache::{TextureCache, DEFAULT_BUDGET};

/// A scene prepared for ray tracing.
#[derive(Debug, Default)]
pub struct RenderScene {
    /// Triangles and acceleration structure.
    pub bvh: Bvh,
    /// Materials, indexed like the scene's, plus a final default entry for meshes without one.
    pub materials: Vec<RenderMaterial>,
    /// The PBR materials behind `materials` (same indices; the default entry has none).
    pub pbr: Vec<Material>,
    /// Decoded texture maps of the materials that have any; `RenderMaterial::maps` indexes this.
    pub maps: Vec<MaterialMaps>,
    /// Corner UVs, three per textured triangle (`Triangle::uv` is the first).
    pub uvs: Vec<[f32; 2]>,
    /// Windows and glazed doors that let the sky into rooms.
    pub portals: Portals,
}

/// The tracer's flat material for a PBR one.
fn flat(material: &Material) -> RenderMaterial {
    let c = material.base_color;
    let alpha = (1.0 - material.transparency) * c[3];
    let strength = material.emission_strength;
    RenderMaterial {
        color: [
            srgb_to_linear(c[0]),
            srgb_to_linear(c[1]),
            srgb_to_linear(c[2]),
        ],
        roughness: material.roughness,
        metallic: material.metallic,
        transparency: 1.0 - alpha,
        ior: material.ior,
        emission: material.emission.map(|e| srgb_to_linear(e) * strength),
        emission_nits: [0.0; 3],
        maps: NO_MAPS,
    }
}

fn has_maps(material: &Material) -> bool {
    [
        &material.maps.albedo,
        &material.maps.normal,
        &material.maps.roughness,
        &material.maps.metallic,
        &material.maps.ao,
        &material.maps.emission,
    ]
    .iter()
    .any(|slot| slot.is_some())
}

impl RenderScene {
    /// Converts the meshes of the world (already in world meters), decodes the textures and builds the BVH.
    /// Triangles with invalid indices, non-finite corners or no area are skipped.
    pub fn new(world: &World) -> Self {
        let scene = &world.scene;
        let mut cache = TextureCache::new(DEFAULT_BUDGET);
        let mut materials = Vec::with_capacity(scene.materials.len() + 1);
        let mut pbr = Vec::with_capacity(scene.materials.len());
        let mut maps = Vec::new();
        for (index, source) in scene.materials.iter().enumerate() {
            let material = world
                .pbr
                .get(index)
                .cloned()
                .unwrap_or_else(|| convert::from_scene(source));
            let mut render = flat(&material);
            if let Some(glow) = world
                .emissive
                .iter()
                .find(|e| e.enabled && e.material == material.name)
            {
                let radiance = emissive_radiance(glow);
                render.emission_nits = [radiance.r, radiance.g, radiance.b];
            }
            if has_maps(&material) {
                let (resolved, _failures) = MaterialMaps::resolve(&mut cache, &material);
                render.maps = maps.len() as u32;
                maps.push(resolved);
            }
            materials.push(render);
            pbr.push(material);
        }
        let default_material = materials.len() as u32;
        materials.push(RenderMaterial::default());
        let mut triangles = Vec::with_capacity(scene.triangle_count());
        let mut uvs = Vec::new();
        let mut pieces = Vec::new();
        for (mesh_index, mesh) in scene.meshes.iter().enumerate() {
            if !world.is_visible(mesh_index) {
                continue;
            }
            let material = mesh
                .material
                .filter(|m| *m < default_material as usize)
                .map_or(default_material, |m| m as u32);
            let has_maps = materials[material as usize].maps != NO_MAPS;
            let textured = has_maps && mesh.uvs.len() == mesh.positions.len();
            // Meshes without UVs get meters-based planar coordinates per triangle.
            let projected = has_maps && mesh.uvs.is_empty();
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
                if (b - a).cross(c - a).length() <= 1e-12 {
                    continue;
                }
                let corner_uvs = textured.then(|| tri.map(|i| mesh.uvs[i as usize]));
                pieces.clear();
                split(
                    Piece {
                        p: [a, b, c],
                        uv: corner_uvs,
                    },
                    &mut pieces,
                );
                for piece in &pieces {
                    let [a, b, c] = piece.p;
                    let mut triangle = Triangle {
                        v0: a,
                        e1: b - a,
                        e2: c - a,
                        material,
                        uv: NO_UV,
                    };
                    if let Some(corners) = piece.uv {
                        triangle.uv = uvs.len() as u32;
                        uvs.extend(corners);
                    } else if projected {
                        triangle.uv = uvs.len() as u32;
                        uvs.extend(uvmap::planar(a, b, c));
                    }
                    triangles.push(triangle);
                }
            }
        }
        let bvh = Bvh::build(triangles);
        let portals = Portals::build(&bvh, &materials);
        Self {
            bvh,
            materials,
            pbr,
            maps,
            uvs,
            portals,
        }
    }

    /// Number of triangles that will be traced.
    pub fn triangle_count(&self) -> usize {
        self.bvh.triangles.len()
    }
}
