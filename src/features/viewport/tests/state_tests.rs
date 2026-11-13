//! View state: when the scene is rebuilt and when the camera re-frames.

use crate::features::viewport::tests::cube;
use crate::features::viewport::ui::{SceneChange, SceneSource, ViewportState};

fn source<'a>(
    id: u8,
    meshes: &'a [crate::features::import_export::models::Mesh],
) -> SceneSource<'a> {
    SceneSource {
        id: [id; 16],
        revision: 0,
        framed: usize::MAX,
        meshes,
        material_colors: vec![[0.5, 0.5, 0.5, 1.0]],
    }
}

#[test]
fn loading_a_model_reports_loaded_once_and_tracks_bounds() {
    let meshes = vec![cube([0.0; 3], 4.0, Some(0))];
    let mut state = ViewportState::new();
    assert_eq!(state.sync(None), SceneChange::Unchanged);
    assert!(!state.has_geometry());
    assert_eq!(state.sync(Some(source(1, &meshes))), SceneChange::Loaded);
    assert!(state.has_geometry());
    assert_eq!(state.stats().triangles, 12);
    assert_eq!(state.sync(Some(source(1, &meshes))), SceneChange::Unchanged);
}

#[test]
fn a_material_change_rebuilds_without_reframing() {
    let meshes = vec![cube([0.0; 3], 4.0, Some(0))];
    let mut state = ViewportState::new();
    state.sync(Some(source(1, &meshes)));
    let first = state.upload().generation;
    let mut changed = source(1, &meshes);
    changed.material_colors = vec![[1.0, 0.0, 0.0, 1.0]];
    assert_eq!(state.sync(Some(changed)), SceneChange::Updated);
    assert!(state.upload().generation > first);
}

#[test]
fn a_different_project_is_loaded_and_clears_saved_cameras() {
    let meshes = vec![cube([0.0; 3], 4.0, None)];
    let mut state = ViewportState::new();
    state.sync(Some(source(1, &meshes)));
    let camera = *state.camera();
    assert!(state.slots.save(0, camera));
    assert_eq!(state.sync(Some(source(2, &meshes))), SceneChange::Loaded);
    assert!(!state.slots.is_filled(0));
    assert_eq!(state.sync(None), SceneChange::Updated);
    assert!(!state.has_geometry());
}

#[test]
fn the_gpu_takes_the_buffers_only_once() {
    let meshes = vec![cube([0.0; 3], 4.0, None)];
    let mut state = ViewportState::new();
    state.sync(Some(source(1, &meshes)));
    let upload = state.upload();
    assert!(upload.take().is_some());
    assert!(upload.take().is_none());
}
