//! Placing, editing, undo/redo and the mass tools.

use crate::features::build::logic::math::Ray;
use crate::features::build::models::{AlignMode, ObjectsState};
use crate::features::library::models::{AssetRef, ProceduralKind};

fn state_with(kind: ProceduralKind) -> ObjectsState {
    let mut state = ObjectsState::default();
    state.browser.active = Some(AssetRef::Procedural { kind, seed: 0 });
    state
}

#[test]
fn placing_on_the_ground_uses_the_ray() {
    let mut state = state_with(ProceduralKind::Tree);
    let ray = Ray::new([4.0, 5.0, 10.0], [0.0, 0.0, -1.0]);
    let id = state.place_at_ray(&ray).expect("placed");
    let object = state.instance(id).unwrap();
    assert!((object.transform.position[0] - 4.0).abs() < 1e-9);
    assert!(object.transform.position[2].abs() < 1e-9);
    assert_eq!(state.selection.len(), 1);
}

#[test]
fn placing_without_an_active_asset_does_nothing() {
    let mut state = ObjectsState::default();
    assert!(state.place_active([0.0; 3]).is_none());
    assert!(!state.history.can_undo());
}

#[test]
fn snapping_lands_on_top_of_another_object() {
    let mut state = state_with(ProceduralKind::Cube);
    state.placement.random_rotation = false;
    state.placement.random_scale = 0.0;
    state.place_active([0.0; 3]);
    let ray = Ray::new([0.0, 0.0, 20.0], [0.0, 0.0, -1.0]);
    let id = state.place_at_ray(&ray).unwrap();
    let z = state.instance(id).unwrap().transform.position[2];
    assert!((z - 1.0).abs() < 1e-6, "cube top at 1 m, got {z}");
}

#[test]
fn undo_and_redo_restore_every_kind_of_edit() {
    let mut state = state_with(ProceduralKind::Bush);
    state.place_active([1.0, 0.0, 0.0]);
    state.place_active([2.0, 0.0, 0.0]);
    state.select_all();
    state.edit_selection(|_, t| t.position[1] = 9.0);
    assert!(state
        .instances()
        .iter()
        .all(|i| i.transform.position[1] == 9.0));
    state.undo();
    assert!(state
        .instances()
        .iter()
        .all(|i| i.transform.position[1] == 0.0));
    state.redo();
    assert!(state
        .instances()
        .iter()
        .all(|i| i.transform.position[1] == 9.0));
    state.delete_selected();
    assert!(state.instances().is_empty());
    state.undo();
    assert_eq!(state.instances().len(), 2);
    assert_eq!(state.instances()[0].transform.position[0], 1.0);
    state.undo();
    state.undo();
    state.undo();
    assert!(state.instances().is_empty());
    assert!(!state.history.can_undo());
    state.redo();
    assert_eq!(state.instances().len(), 1);
}

#[test]
fn a_new_edit_clears_redo() {
    let mut state = state_with(ProceduralKind::Bush);
    state.place_active([0.0; 3]);
    state.undo();
    assert!(state.history.can_redo());
    state.place_active([1.0; 3]);
    assert!(!state.history.can_redo());
}

#[test]
fn duplicate_array_offsets_each_copy() {
    let mut state = state_with(ProceduralKind::Bench);
    state.place_active([0.0; 3]);
    state.mass.copies = 3;
    state.mass.offset = [2.0, 0.0, 0.0];
    state.duplicate_array();
    let mut xs: Vec<f64> = state
        .instances()
        .iter()
        .map(|i| i.transform.position[0])
        .collect();
    xs.sort_by(f64::total_cmp);
    assert_eq!(xs, vec![0.0, 2.0, 4.0, 6.0]);
    state.undo();
    assert_eq!(state.instances().len(), 1);
}

#[test]
fn scatter_stays_inside_the_radius_and_is_one_step() {
    let mut state = state_with(ProceduralKind::Palm);
    state.mass.scatter_count = 40;
    state.mass.scatter_radius = 10.0;
    state.scatter([100.0, 50.0, 0.0]);
    assert_eq!(state.instances().len(), 40);
    for object in state.instances() {
        let p = object.transform.position;
        assert!(((p[0] - 100.0).powi(2) + (p[1] - 50.0).powi(2)).sqrt() <= 10.0 + 1e-9);
    }
    state.undo();
    assert!(state.instances().is_empty());
}

#[test]
fn scattered_vegetation_gets_different_seeds() {
    let mut state = state_with(ProceduralKind::Tree);
    state.mass.scatter_count = 10;
    state.scatter([0.0; 3]);
    let seeds: std::collections::HashSet<_> = state
        .instances()
        .iter()
        .map(|i| match i.asset {
            AssetRef::Procedural { seed, .. } => seed,
            AssetRef::Imported { .. } => 0,
        })
        .collect();
    assert!(seeds.len() > 1);
}

#[test]
fn align_uses_min_center_and_max() {
    let mut state = state_with(ProceduralKind::Cube);
    for x in [0.0, 4.0, 8.0] {
        state.place_active([x, x, 0.0]);
    }
    state.select_all();
    state.align(0, AlignMode::Center);
    assert!(state
        .instances()
        .iter()
        .all(|i| (i.transform.position[0] - 4.0).abs() < 1e-9));
    state.align(1, AlignMode::Max);
    assert!(state
        .instances()
        .iter()
        .all(|i| i.transform.position[1] == 8.0));
    state.align(1, AlignMode::Min);
    assert!(state
        .instances()
        .iter()
        .all(|i| i.transform.position[1] == 8.0));
}

#[test]
fn snap_selection_drops_onto_the_object_below() {
    let mut state = state_with(ProceduralKind::Cube);
    state.placement.random_scale = 0.0;
    state.place_active([0.0; 3]);
    let floating = state.place_active([0.0, 0.0, 7.0]).unwrap();
    state.selection = [floating].into_iter().collect();
    state.snap_selection_to_surface();
    let z = state.instance(floating).unwrap().transform.position[2];
    assert!((z - 1.0).abs() < 1e-6);
}

#[test]
fn grid_snapping_rounds_placement() {
    let mut state = state_with(ProceduralKind::Cube);
    state.placement.grid = 0.5;
    let id = state.place_active([1.26, 0.74, 0.0]).unwrap();
    let p = state.instance(id).unwrap().transform.position;
    assert_eq!((p[0], p[1]), (1.5, 0.5));
}

#[test]
fn random_placement_is_reproducible_for_the_same_start() {
    let place = || {
        let mut state = state_with(ProceduralKind::Tree);
        let id = state.place_active([0.0; 3]).unwrap();
        state.instance(id).unwrap().clone()
    };
    assert_eq!(place(), place());
}
