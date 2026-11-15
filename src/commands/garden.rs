//! Use case: "Plantar jardín". Surrounds the model of the open project with hedges, palms and trees (the rendering
//! feature's `garden::plant_around`) and hands the new plants to the live objects state, so the 3D view shows
//! them, Save stores them and the user can move or delete each one.

use crate::features::landscape::models::Status;
use crate::features::rendering::logic::garden::plant_around;
use crate::models::AppState;
use crate::services::localization::{tr, trf};

/// Plants a garden around the model of the open project and reports the result in the landscape panel's status
/// line. Returns how many plants were added (zero without a project, for a read-only one, or for a project
/// without a measurable building).
pub fn plant_garden(app: &mut AppState) -> usize {
    let Some(project) = app.workspace.project.as_mut() else {
        app.landscape.status = Some(Status::Error(tr("landscape.garden.no_project").to_owned()));
        return 0;
    };
    if project.read_only {
        app.landscape.status = Some(Status::Error(tr("landscape.garden.read_only").to_owned()));
        return 0;
    }
    // The live objects are the single copy: bring the document up to date before adding to it.
    project.document.objects = app.objects.to_doc();
    let added = plant_around(project);
    if added == 0 {
        app.landscape.status = Some(Status::Error(tr("landscape.garden.no_building").to_owned()));
        return 0;
    }
    app.objects.load(project.document.objects.clone());
    app.composer.invalidate();
    app.landscape.status = Some(Status::Info(trf(
        "landscape.garden.planted",
        &[&added.to_string()],
    )));
    added
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::features::import_export::models::Mesh;
    use crate::features::projects::models::{Project, ProjectDocument};

    /// A 10 x 6 x 3 m block standing on Z = 0.
    fn block_project(read_only: bool) -> Project {
        let mut positions = Vec::new();
        for z in [0.0, 3.0] {
            for (x, y) in [(0.0, 0.0), (10.0, 0.0), (10.0, 6.0), (0.0, 6.0)] {
                positions.push([x, y, z]);
            }
        }
        let indices = vec![
            0, 1, 2, 0, 2, 3, 4, 6, 5, 4, 7, 6, 0, 4, 5, 0, 5, 1, 1, 5, 6, 1, 6, 2, 2, 6, 7, 2, 7,
            3, 3, 7, 4, 3, 4, 0,
        ];
        Project {
            id: [3; 16],
            document: ProjectDocument::default(),
            meshes: vec![Mesh {
                positions,
                indices,
                material: None,
                uvs: Vec::new(),
            }],
            path: None,
            legacy_source: None,
            read_only,
            dirty: false,
        }
    }

    fn app_with(project: Option<Project>) -> AppState {
        let mut app = AppState::with_data_dir(std::env::temp_dir().join("gr-garden-test"));
        app.workspace.project = project;
        app
    }

    #[test]
    fn the_button_plants_into_the_live_objects_and_the_document() {
        let mut app = app_with(Some(block_project(false)));
        let added = plant_garden(&mut app);
        assert!(added > 12, "{added} plants");
        assert_eq!(app.objects.instances().len(), added);
        let project = app.workspace.project.as_ref().expect("project");
        assert_eq!(project.document.objects.instances.len(), added);
        assert!(project.dirty);
        let message = trf("landscape.garden.planted", &[&added.to_string()]);
        assert_eq!(app.landscape.status, Some(Status::Info(message)));
    }

    #[test]
    fn plants_already_placed_by_hand_are_kept() {
        let mut app = app_with(Some(block_project(false)));
        let first = plant_garden(&mut app);
        let second = plant_garden(&mut app);
        assert_eq!(app.objects.instances().len(), first + second);
    }

    #[test]
    fn without_a_project_or_when_read_only_nothing_is_planted_and_the_user_is_told() {
        let mut app = app_with(None);
        assert_eq!(plant_garden(&mut app), 0);
        assert_eq!(
            app.landscape.status,
            Some(Status::Error(tr("landscape.garden.no_project").to_owned()))
        );
        let mut app = app_with(Some(block_project(true)));
        assert_eq!(plant_garden(&mut app), 0);
        assert!(app.objects.instances().is_empty());
        assert_eq!(
            app.landscape.status,
            Some(Status::Error(tr("landscape.garden.read_only").to_owned()))
        );
    }

    #[test]
    fn a_project_without_a_building_gets_no_garden() {
        let mut project = block_project(false);
        project.meshes.clear();
        let mut app = app_with(Some(project));
        assert_eq!(plant_garden(&mut app), 0);
        assert_eq!(
            app.landscape.status,
            Some(Status::Error(tr("landscape.garden.no_building").to_owned()))
        );
    }
}
