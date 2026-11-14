//! Save-and-open checks of everything the shell keeps in the project document.

use super::*;
use crate::commands::camera_link;
use crate::features::build::models::{PlacedObject, Transform};
use crate::features::landscape::logic::actions;
use crate::features::library::models::{AssetRef, ProceduralKind};
use crate::features::lighting::models::Light;
use crate::features::movie::models::CameraPose;
use crate::features::projects::services::util;

fn temp_folder(label: &str) -> std::path::PathBuf {
    let folder = std::env::temp_dir().join(format!(
        "gr-sync-{label}-{}",
        util::hex(&util::new_document_id())
    ));
    std::fs::create_dir_all(&folder).unwrap();
    folder
}

#[test]
fn everything_the_shell_edits_survives_save_and_open() {
    let folder = temp_folder("roundtrip");
    let mut app = AppState::with_data_dir(folder.join("datos"));
    app.workspace.start_new("Casa");
    step(&mut app);
    app.objects.doc.instances.push(PlacedObject {
        id: 1,
        asset: AssetRef::Procedural {
            kind: ProceduralKind::Tree,
            seed: 7,
        },
        transform: Transform::at([4.0, 5.0, 0.0]),
    });
    app.objects.doc.next_id = 2;
    app.objects.dirty = true;
    actions::new_flat(&mut app.landscape, 51, 51, 2.0);
    set_ground_visible(&mut app, false);
    app.lighting.setup.lights.push(Light::default());
    app.animation.record_camera(1.0, &CameraPose::default());
    app.environment.hour = 7.5;
    camera_link::save_current(&mut app);
    step(&mut app);
    assert!(app.workspace.dirty());
    let file = folder.join("Casa.gr");
    assert!(app.workspace.save_to(&file, false));

    let mut other = AppState::with_data_dir(folder.join("otros"));
    assert!(other.workspace.open_file(&file));
    step(&mut other);
    assert_eq!(other.objects.instances().len(), 1);
    assert_eq!(
        other.objects.instances()[0].transform.position,
        [4.0, 5.0, 0.0]
    );
    assert_eq!(other.landscape.landscape.terrain.columns, 51);
    assert!(
        !other.landscape.landscape.visible,
        "the hidden ground stays hidden"
    );
    assert_eq!(other.lighting.setup.lights.len(), 1);
    assert_eq!(other.animation.camera.keys.len(), 1);
    assert_eq!(other.environment.hour, 7.5);
    assert_eq!(other.cameras.len(), 1);
    assert!(!other.workspace.dirty(), "opening changes nothing");
    let _ = std::fs::remove_dir_all(&folder);
}

#[test]
fn an_untouched_project_keeps_a_visible_base_ground_and_stays_clean() {
    let folder = temp_folder("ground");
    let mut app = AppState::with_data_dir(folder.join("datos"));
    app.workspace.start_new("Vacío");
    step(&mut app);
    step(&mut app);
    assert!(app.landscape.landscape.visible);
    assert_eq!(app.landscape.landscape.terrain.columns, 101);
    let project = app.workspace.project.as_ref().unwrap();
    assert!(project.document.landscape.is_none());
    let _ = std::fs::remove_dir_all(&folder);
}

#[test]
fn photo_filters_cameras_and_studio_survive_save_and_open() {
    use crate::features::camera::models::filter::FilterKind;
    let folder = temp_folder("photo");
    let mut app = AppState::with_data_dir(folder.join("datos"));
    app.workspace.start_new("Foto");
    step(&mut app);
    assert!(app
        .workspace
        .project
        .as_ref()
        .unwrap()
        .document
        .studio
        .is_none());
    app.camera_studio.stack.push(FilterKind::ALL[0]);
    app.camera_studio.camera.set_aperture(2.0);
    app.studio.state.quality = 2;
    app.studio.state.resolution = 3;
    app.environment.hour = 18.0;
    camera_link::save_current(&mut app);
    app.cameras[0].thumbnail = Some("AAAA".into());
    step(&mut app);
    let file = folder.join("Foto.gr");
    assert!(app.workspace.save_to(&file, false));
    let mut other = AppState::with_data_dir(folder.join("otros"));
    assert!(other.workspace.open_file(&file));
    step(&mut other);
    assert_eq!(other.camera_studio.stack, app.camera_studio.stack);
    assert_eq!(other.camera_studio.camera.lens.aperture, 2.0);
    assert_eq!(other.studio.state.quality, 2);
    assert_eq!(other.studio.state.resolution, 3);
    assert_eq!(other.environment.hour, 18.0);
    assert_eq!(
        other.cameras[0].filters,
        Some(app.camera_studio.stack.clone())
    );
    assert_eq!(other.cameras[0].thumbnail.as_deref(), Some("AAAA"));
    assert!(!other.workspace.dirty());
    let _ = std::fs::remove_dir_all(&folder);
}

#[test]
fn an_environment_set_back_to_the_default_is_still_saved() {
    let folder = temp_folder("env-default");
    let mut app = AppState::with_data_dir(folder.join("datos"));
    app.workspace.start_new("Cielo");
    step(&mut app);
    app.environment.hour = 7.0;
    step(&mut app);
    app.environment = EnvironmentSettings::default();
    step(&mut app);
    let file = folder.join("Cielo.gr");
    assert!(app.workspace.save_to(&file, false));
    let mut other = AppState::with_data_dir(folder.join("otros"));
    assert!(other.workspace.open_file(&file));
    step(&mut other);
    assert_eq!(other.environment, EnvironmentSettings::default());
    let _ = std::fs::remove_dir_all(&folder);
}
