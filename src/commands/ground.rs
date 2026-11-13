//! Use cases around the base ground and the imported model that stands on it: resting a model on Z = 0 and
//! moving it. The meshes of the project are edited in place, so the render, the viewport and the saved file agree.

use crate::features::import_export::models::Mesh;
use crate::features::projects::models::Project;

/// Lowest Z of every vertex, `None` for a project without geometry.
pub fn lowest_z(meshes: &[Mesh]) -> Option<f32> {
    meshes
        .iter()
        .flat_map(|mesh| mesh.positions.iter())
        .map(|p| p[2])
        .filter(|z| z.is_finite())
        .reduce(f32::min)
}

/// Moves every vertex of the project by `delta` meters (X, Y, Z). Does nothing for a read-only project.
/// Returns whether anything moved; the caller marks the project as modified.
pub fn shift(project: &mut Project, delta: [f32; 3]) -> bool {
    if project.read_only || delta == [0.0; 3] {
        return false;
    }
    for mesh in &mut project.meshes {
        for position in &mut mesh.positions {
            for axis in 0..3 {
                position[axis] += delta[axis];
            }
        }
    }
    true
}

/// Rests the model on the ground: its lowest point goes to Z = 0. Returns whether it moved.
pub fn settle(project: &mut Project) -> bool {
    match lowest_z(&project.meshes) {
        Some(z) if z.abs() > 0.005 => shift(project, [0.0, 0.0, -z]),
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::features::projects::services::store;

    fn project_with_floor_at(z: f32) -> Project {
        let mut project = store::new_project("Casa");
        project.meshes = vec![Mesh {
            positions: vec![[0.0, 0.0, z], [1.0, 0.0, z + 3.0], [0.0, 1.0, z + 3.0]],
            indices: vec![0, 1, 2],
            material: None,
            uvs: Vec::new(),
        }];
        project
    }

    #[test]
    fn a_model_rests_on_the_ground() {
        let mut project = project_with_floor_at(12.5);
        assert!(settle(&mut project));
        assert_eq!(lowest_z(&project.meshes), Some(0.0));
        assert!(!settle(&mut project), "already resting");
    }

    #[test]
    fn a_model_can_be_moved_and_read_only_ones_cannot() {
        let mut project = project_with_floor_at(0.0);
        assert!(shift(&mut project, [2.0, 0.0, 1.0]));
        assert_eq!(project.meshes[0].positions[1], [3.0, 0.0, 4.0]);
        project.read_only = true;
        assert!(!shift(&mut project, [1.0, 0.0, 0.0]));
    }
}
