//! Builds the world a project renders: the imported model, the always-present ground, the landscape (terrain
//! layers, water, vegetation) and the placed objects of the library, with the project's PBR materials.

use crate::features::build::logic::math::to_world;
use crate::features::build::models::Transform;
use crate::features::import_export::models::{ImportedScene, Material as SceneMaterial, Mesh};
use crate::features::landscape::logic::actions::landscape_meshes;
use crate::features::landscape::models::Landscape;
use crate::features::library::logic::procedural;
use crate::features::library::models::{AssetRef, ProceduralKind};
use crate::features::library::services::custom_asset;
use crate::features::materials::logic::project_io::library_from_docs;
use crate::features::projects::logic::convert;
use crate::features::projects::models::Project;
use crate::features::rendering::engine::World;
use std::collections::HashMap;
use std::path::Path;

/// The ground sinks this far below Z = 0 so a floor slab at Z = 0 never fights it for the same pixels.
pub const GROUND_SINK: f32 = 0.01;
/// Triangle budget of the scattered vegetation; denser landscapes are thinned evenly.
pub const VEGETATION_TRIANGLES: usize = 1_500_000;

/// A scene being assembled: meshes and materials appended in order.
struct Builder {
    scene: ImportedScene,
}

impl Builder {
    /// Adds `source`'s materials and returns the index its first material got.
    fn add_materials(&mut self, materials: &[SceneMaterial]) -> usize {
        let base = self.scene.materials.len();
        self.scene.materials.extend_from_slice(materials);
        base
    }

    /// Adds the meshes of `source` moved by `place`, shifting their material indices by `base`.
    fn add_meshes(&mut self, source: &[Mesh], base: usize, place: &dyn Fn([f32; 3]) -> [f32; 3]) {
        for mesh in source {
            self.scene.meshes.push(Mesh {
                positions: mesh.positions.iter().map(|p| place(*p)).collect(),
                indices: mesh.indices.clone(),
                material: mesh.material.map(|m| m + base),
                uvs: mesh.uvs.clone(),
            });
        }
    }
}

/// The procedural kind a vegetation asset name stands for.
pub fn vegetation_kind(name: &str) -> ProceduralKind {
    let name = name.to_lowercase();
    if name.contains("palm") {
        ProceduralKind::Palm
    } else if name.contains("pino") || name.contains("pine") || name.contains("conif") {
        ProceduralKind::Conifer
    } else if name.contains("arbusto") || name.contains("bush") || name.contains("shrub") {
        ProceduralKind::Bush
    } else {
        ProceduralKind::Tree
    }
}

fn transform_point(transform: &Transform, p: [f32; 3]) -> [f32; 3] {
    let w = to_world(
        transform,
        [f64::from(p[0]), f64::from(p[1]), f64::from(p[2])],
    );
    [w[0] as f32, w[1] as f32, w[2] as f32]
}

/// Geometry of one asset, generated (procedural) or re-imported (imported models); `None` when unavailable.
fn asset_scene(project: &Project, asset: AssetRef) -> Option<ImportedScene> {
    match asset {
        AssetRef::Procedural { kind, seed } => Some(procedural::generate(kind, seed)),
        AssetRef::Imported { id } => {
            let model = project
                .document
                .objects
                .imported_assets
                .iter()
                .find(|m| m.id == id)?;
            custom_asset::import(Path::new(&model.source)).ok()
        }
    }
}

/// Places every object instance of the project. Each distinct asset adds its materials once.
fn add_objects(builder: &mut Builder, project: &Project) {
    let mut bases: HashMap<AssetRef, Option<(ImportedScene, usize)>> = HashMap::new();
    for instance in &project.document.objects.instances {
        let entry = bases.entry(instance.asset).or_insert_with(|| {
            asset_scene(project, instance.asset).map(|scene| {
                let base = builder.add_materials(&scene.materials);
                (scene, base)
            })
        });
        let Some((scene, base)) = entry else {
            continue;
        };
        let transform = instance.transform;
        builder.add_meshes(&scene.meshes, *base, &|p| transform_point(&transform, p));
    }
}

/// The scattered vegetation of `landscape` as meshes (material indices local to the returned list), thinned
/// evenly so they fit in `budget` triangles. Shared by the 3D view and the renderer.
pub fn plant_meshes(landscape: &Landscape, budget: usize) -> (Vec<Mesh>, Vec<SceneMaterial>) {
    let mut assets: HashMap<ProceduralKind, (ImportedScene, usize)> = HashMap::new();
    let (mut meshes, mut materials) = (Vec::new(), Vec::new());
    let mut remaining = budget;
    let total = landscape.vegetation.len().max(1);
    let stride = {
        let sample = procedural::generate(ProceduralKind::Tree, 0)
            .triangle_count()
            .max(1);
        (total * sample).div_ceil(budget.max(1)).max(1)
    };
    for (index, plant) in landscape.vegetation.iter().enumerate() {
        if index % stride != 0 {
            continue;
        }
        let kind = vegetation_kind(&plant.asset);
        let (scene, base) = assets.entry(kind).or_insert_with(|| {
            let scene = procedural::generate(kind, 0);
            let base = materials.len();
            materials.extend_from_slice(&scene.materials);
            (scene, base)
        });
        let cost = scene.triangle_count();
        if cost > remaining {
            break;
        }
        remaining -= cost;
        let transform = Transform {
            position: plant.position.map(f64::from),
            rotation: [0.0, 0.0, plant.rotation_deg],
            scale: [plant.scale; 3],
        };
        for mesh in &scene.meshes {
            meshes.push(Mesh {
                positions: mesh
                    .positions
                    .iter()
                    .map(|p| transform_point(&transform, *p))
                    .collect(),
                indices: mesh.indices.clone(),
                material: mesh.material.map(|m| m + *base),
                uvs: mesh.uvs.clone(),
            });
        }
    }
    (meshes, materials)
}

/// Places the scattered vegetation of the landscape within the triangle budget.
fn add_vegetation(builder: &mut Builder, landscape: &Landscape) {
    let (meshes, materials) = plant_meshes(landscape, VEGETATION_TRIANGLES);
    let base = builder.add_materials(&materials);
    builder.add_meshes(&meshes, base, &|p| p);
}

/// Half side of the plane that carries the ground out to the horizon, meters.
const HORIZON: f32 = 30_000.0;

/// A huge flat quad just under the terrain, with the terrain's first material, so the ground meets the sky at the
/// horizon instead of ending at the edge of the height grid.
fn horizon_plane(material: usize) -> Mesh {
    let z = -GROUND_SINK - 0.02;
    Mesh {
        positions: vec![
            [-HORIZON, -HORIZON, z],
            [HORIZON, -HORIZON, z],
            [HORIZON, HORIZON, z],
            [-HORIZON, HORIZON, z],
        ],
        indices: vec![0, 1, 2, 0, 2, 3],
        material: Some(material),
        uvs: Vec::new(),
    }
}

/// The world of `project`: its model, ground, landscape and objects, with the materials the project defines.
pub fn build_world(project: &Project) -> World {
    let document = &project.document;
    let model = convert::to_scene(document, &project.meshes, 0);
    let project_materials = model.materials.len();
    let mut builder = Builder { scene: model };
    add_objects(&mut builder, project);
    let focus = builder.scene.bounds();
    let landscape = document.landscape.clone().unwrap_or_default();
    if landscape.visible {
        let (meshes, materials) = landscape_meshes(&landscape, 0.0);
        let base = builder.add_materials(&materials);
        builder.add_meshes(&meshes, base, &|p| [p[0], p[1], p[2] - GROUND_SINK]);
        if !meshes.is_empty() {
            builder.scene.meshes.push(horizon_plane(base));
        }
        add_vegetation(&mut builder, &landscape);
    }
    let mut world = World::new(builder.scene);
    let library = library_from_docs(&document.materials);
    for (slot, material) in world
        .pbr
        .iter_mut()
        .zip(library.iter())
        .take(project_materials)
    {
        *slot = material.clone();
    }
    world.emissive = document.lighting.emissive.clone();
    world.focus = focus;
    world
}
