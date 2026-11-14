//! Use case: the scene the 3D view shows. It is the open project's geometry plus the landscape (terrain and water)
//! and every placed object, merged into one mesh list. The project's own meshes are copied once; only the extra
//! part is rebuilt when the landscape or the objects change.

use crate::features::build::logic::math::world_matrix;
use crate::features::build::models::ObjectsState;
use crate::features::import_export::models::Mesh;
use crate::features::landscape::logic::actions::landscape_meshes;
use crate::features::landscape::models::LandscapeState;
use crate::features::library::models::AssetRef;
use crate::features::projects::models::Project;
use crate::features::rendering::logic::world::plant_meshes;
use crate::features::viewport::ui::SceneSource;
use std::collections::hash_map::DefaultHasher;
use std::collections::HashMap;
use std::hash::{Hash, Hasher};

/// Identifier of the scene shown when no project is open (objects and terrain can still be tried out).
const SCRATCH_ID: [u8; 16] = [0; 16];

/// How far the ground is drawn below Z = 0, in meters.
const GROUND_SINK: f32 = 0.02;

/// Triangle budget of the vegetation drawn in the 3D view (the renderer has a larger one).
const VIEW_PLANT_TRIANGLES: usize = 400_000;

/// Identity of the copied project geometry.
type BaseKey = ([u8; 16], usize, usize);

/// Builds and caches the merged scene.
#[derive(Default)]
pub struct SceneComposer {
    base_key: Option<BaseKey>,
    base_len: usize,
    meshes: Vec<Mesh>,
    base_colors: Vec<[f32; 4]>,
    extra_colors: Vec<[f32; 4]>,
    extras_key: Option<(u64, u64, bool)>,
    revision: u64,
    framed: usize,
    id: [u8; 16],
}

/// Fingerprint of the placed objects: changes whenever one is added, removed, moved, rotated or scaled.
fn objects_fingerprint(objects: &ObjectsState) -> u64 {
    let mut hasher = DefaultHasher::new();
    objects.doc.imported_assets.len().hash(&mut hasher);
    for object in objects.instances() {
        object.id.hash(&mut hasher);
        object.asset.hash(&mut hasher);
        let t = &object.transform;
        for value in t.position {
            value.to_bits().hash(&mut hasher);
        }
        for value in t.rotation.iter().chain(t.scale.iter()) {
            value.to_bits().hash(&mut hasher);
        }
    }
    hasher.finish()
}

/// Applies a column-major 4x4 matrix to a point.
fn transform_point(m: &[[f32; 4]; 4], p: [f32; 3]) -> [f32; 3] {
    let mut out = [0.0; 3];
    for (axis, value) in out.iter_mut().enumerate() {
        *value = m[0][axis] * p[0] + m[1][axis] * p[1] + m[2][axis] * p[2] + m[3][axis];
    }
    out
}

impl SceneComposer {
    /// Forgets the copied project geometry (a mesh was edited in place); the next `compose` copies it again.
    pub fn invalidate(&mut self) {
        self.base_key = None;
    }

    /// The scene to show for `project` with the landscape (when it is visible) and `objects` merged in.
    /// Without a project the extras alone are shown.
    pub fn compose<'a>(
        &'a mut self,
        project: Option<&Project>,
        landscape: &LandscapeState,
        objects: &mut ObjectsState,
    ) -> Option<SceneSource<'a>> {
        let landscape_visible = landscape.landscape.visible;
        let id = project.map_or(SCRATCH_ID, |p| p.id);
        let base_key = project.map(|p| {
            (
                p.id,
                p.meshes.len(),
                p.meshes.iter().map(|m| m.positions.len()).sum::<usize>(),
            )
        });
        let base_changed = base_key != self.base_key;
        if base_changed {
            self.base_key = base_key;
            self.meshes = project.map_or_else(Vec::new, |p| p.meshes.clone());
            self.base_len = self.meshes.len();
            self.extras_key = None;
        }
        self.base_colors = project.map_or_else(Vec::new, |p| {
            p.document.materials.iter().map(|m| m.color).collect()
        });
        let key = (
            landscape.revision,
            objects_fingerprint(objects),
            landscape_visible,
        );
        if self.extras_key != Some(key) {
            self.extras_key = Some(key);
            self.revision += 1;
            self.meshes.truncate(self.base_len);
            self.extra_colors.clear();
            let offset = self.base_colors.len();
            self.add_objects(objects, offset);
            // The ground goes last so the camera frames the model and the objects, not the 200 m plane.
            self.framed = if self.meshes.is_empty() {
                usize::MAX
            } else {
                self.meshes.len()
            };
            if landscape_visible {
                self.add_landscape(landscape, offset);
            }
        }
        self.id = id;
        let mut colors = self.base_colors.clone();
        colors.extend_from_slice(&self.extra_colors);
        Some(SceneSource {
            id,
            revision: self.revision,
            meshes: &self.meshes,
            material_colors: colors,
            framed: self.framed,
        })
    }

    fn add_landscape(&mut self, landscape: &LandscapeState, offset: usize) {
        let (meshes, materials) = landscape_meshes(&landscape.landscape, 0.0);
        let first = self.extra_colors.len();
        self.extra_colors
            .extend(materials.iter().map(|material| material.color));
        for mut mesh in meshes {
            // Slightly below Z = 0 so the viewport grid and flat objects never fight with the ground.
            for position in &mut mesh.positions {
                position[2] -= GROUND_SINK;
            }
            mesh.material = mesh.material.map(|index| offset + first + index);
            self.meshes.push(mesh);
        }
        // The plants stand on the terrain, so they are not sunk with it.
        let (plants, materials) = plant_meshes(&landscape.landscape, VIEW_PLANT_TRIANGLES);
        let first = self.extra_colors.len();
        self.extra_colors
            .extend(materials.iter().map(|material| material.color));
        for mut mesh in plants {
            mesh.material = mesh.material.map(|index| offset + first + index);
            self.meshes.push(mesh);
        }
    }

    fn add_objects(&mut self, objects: &mut ObjectsState, offset: usize) {
        let mut slots: HashMap<(AssetRef, usize), usize> = HashMap::new();
        let placed: Vec<_> = objects
            .instances()
            .iter()
            .map(|object| (object.asset, object.transform))
            .collect();
        for (asset, transform) in placed {
            let Some(scene) = objects.cache.scene(asset) else {
                continue;
            };
            let matrix = world_matrix(&transform);
            for mesh in &scene.meshes {
                let material = mesh.material.map(|index| {
                    *slots.entry((asset, index)).or_insert_with(|| {
                        let color = scene
                            .materials
                            .get(index)
                            .map_or([0.7, 0.7, 0.7, 1.0], |m| m.color);
                        self.extra_colors.push(color);
                        offset + self.extra_colors.len() - 1
                    })
                });
                self.meshes.push(Mesh {
                    positions: mesh
                        .positions
                        .iter()
                        .map(|p| transform_point(&matrix, *p))
                        .collect(),
                    indices: mesh.indices.clone(),
                    material,
                    uvs: mesh.uvs.clone(),
                });
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn points_are_moved_by_the_matrix() {
        let mut matrix = [[0.0; 4]; 4];
        matrix[0][0] = 2.0;
        matrix[1][1] = 1.0;
        matrix[2][2] = 1.0;
        matrix[3] = [10.0, 0.0, 0.0, 1.0];
        assert_eq!(transform_point(&matrix, [1.0, 2.0, 3.0]), [12.0, 2.0, 3.0]);
    }

    #[test]
    fn a_new_scene_shows_the_base_ground_with_a_stable_revision() {
        let mut composer = SceneComposer::default();
        let landscape = LandscapeState::default();
        let mut objects = ObjectsState::default();
        let first = composer
            .compose(None, &landscape, &mut objects)
            .map(|s| (s.revision, s.meshes.len()));
        let second = composer
            .compose(None, &landscape, &mut objects)
            .map(|s| (s.revision, s.meshes.len()));
        assert_eq!(first, second);
        assert!(first.is_some_and(|f| f.1 > 0), "the base ground is drawn");
    }
}
