//! Use case: show the open project in the 3D view.

use crate::features::projects::models::Project;
use crate::features::viewport::ui::SceneSource;

/// The geometry of `project` as the 3D view wants it (`None` when no project is open).
pub fn scene_source(project: Option<&Project>) -> Option<SceneSource<'_>> {
    let project = project?;
    Some(SceneSource {
        id: project.id,
        revision: 0,
        framed: usize::MAX,
        meshes: &project.meshes,
        material_colors: project
            .document
            .materials
            .iter()
            .map(|material| material.color)
            .collect(),
    })
}
