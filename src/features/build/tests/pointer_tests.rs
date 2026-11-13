//! Picking and the pointer gizmos.

use crate::features::build::logic::math::{self, Ray};
use crate::features::build::models::{AxisLock, GizmoMode, ObjectsState};
use crate::features::library::models::{AssetRef, ProceduralKind};

fn down(x: f64, y: f64) -> Ray {
    Ray::new([x, y, 50.0], [0.0, 0.0, -1.0])
}

fn cube_at(x: f64, y: f64) -> (ObjectsState, u64) {
    let mut state = ObjectsState::default();
    state.browser.active = Some(AssetRef::Procedural {
        kind: ProceduralKind::Cube,
        seed: 0,
    });
    state.placement.random_rotation = false;
    state.placement.random_scale = 0.0;
    let id = state.place_active([x, y, 0.0]).unwrap();
    state.clear_selection();
    state.mode = GizmoMode::Move;
    (state, id)
}

#[test]
fn picking_hits_the_nearest_triangle_and_respects_exclusion() {
    let (mut state, id) = cube_at(3.0, 0.0);
    let hit = state.pick(&down(3.0, 0.1), &Default::default()).unwrap();
    assert_eq!(hit.id, Some(id));
    assert!((hit.point[2] - 1.0).abs() < 1e-9);
    assert!(state.pick(&down(10.0, 0.0), &Default::default()).is_none());
    let excluded = [id].into_iter().collect();
    assert!(state.pick(&down(3.0, 0.1), &excluded).is_none());
}

#[test]
fn picking_follows_rotation_and_scale() {
    let (mut state, id) = cube_at(0.0, 0.0);
    state.select(id, false);
    state.edit_selection(|_, t| {
        t.scale = [4.0, 1.0, 1.0];
        t.rotation[2] = 90.0;
    });
    // The 4 m side now runs along Y.
    assert!(state.pick(&down(0.0, 1.8), &Default::default()).is_some());
    assert!(state.pick(&down(1.8, 0.0), &Default::default()).is_none());
}

#[test]
fn dragging_moves_the_selection_and_is_one_undo_step() {
    let (mut state, id) = cube_at(0.0, 0.0);
    state.pointer_press(&down(0.0, 0.0), false);
    assert!(state.selection.contains(&id));
    state.pointer_drag(&down(2.0, 1.0));
    state.pointer_drag(&down(5.0, 3.0));
    let p = state.instance(id).unwrap().transform.position;
    assert!((p[0] - 5.0).abs() < 1e-6 && (p[1] - 3.0).abs() < 1e-6);
    state.pointer_release();
    state.undo();
    assert_eq!(state.instance(id).unwrap().transform.position, [0.0; 3]);
    state.undo();
    assert!(state.instance(id).is_none());
}

#[test]
fn axis_lock_restricts_the_move() {
    let (mut state, id) = cube_at(0.0, 0.0);
    state.axis = AxisLock::X;
    state.pointer_press(&down(0.0, 0.0), false);
    state.pointer_drag(&down(4.0, 4.0));
    state.pointer_release();
    let p = state.instance(id).unwrap().transform.position;
    assert!((p[0] - 4.0).abs() < 1e-6 && p[1].abs() < 1e-9);
}

#[test]
fn vertical_lock_moves_only_height() {
    let (mut state, id) = cube_at(0.0, 0.0);
    state.axis = AxisLock::Z;
    let side = Ray::new([-20.0, 0.1, 0.5], [1.0, 0.0, 0.0]);
    state.pointer_press(&side, false);
    state.pointer_drag(&Ray::new([-20.0, 0.1, 3.5], [1.0, 0.0, 0.0]));
    state.pointer_release();
    let p = state.instance(id).unwrap().transform.position;
    assert!((p[2] - 3.0).abs() < 1e-6 && p[0].abs() < 1e-9);
}

#[test]
fn cancelling_a_drag_restores_the_start() {
    let (mut state, id) = cube_at(1.0, 1.0);
    state.pointer_press(&down(1.0, 1.0), false);
    state.pointer_drag(&down(9.0, 9.0));
    state.pointer_cancel();
    assert_eq!(
        state.instance(id).unwrap().transform.position,
        [1.0, 1.0, 0.0]
    );
    // Only the placement is in the history.
    state.undo();
    assert!(state.instance(id).is_none());
}

#[test]
fn rotating_snaps_to_the_step() {
    let (mut state, id) = cube_at(0.0, 0.0);
    state.mode = GizmoMode::Rotate;
    state.placement.rotation_step = 15.0;
    state.pointer_press(&down(0.5, 0.0), false);
    state.pointer_drag(&down(0.4, 0.45));
    state.pointer_release();
    let yaw = state.instance(id).unwrap().transform.rotation[2];
    assert!(
        (yaw / 15.0 - (yaw / 15.0).round()).abs() < 1e-4,
        "yaw {yaw}"
    );
    assert!(yaw > 30.0 && yaw < 60.0);
}

#[test]
fn scaling_follows_the_distance_ratio() {
    let (mut state, id) = cube_at(0.0, 0.0);
    state.mode = GizmoMode::Scale;
    state.pointer_press(&down(0.5, 0.0), false);
    state.pointer_drag(&down(1.0, 0.0));
    state.pointer_release();
    let s = state.instance(id).unwrap().transform.scale;
    assert!((s[0] - 2.0).abs() < 1e-5 && s[0] == s[2]);
}

#[test]
fn shift_click_toggles_the_selection_and_empty_click_clears() {
    let (mut state, a) = cube_at(0.0, 0.0);
    let b = state.place_active([5.0, 0.0, 0.0]).unwrap();
    state.clear_selection();
    state.pointer_press(&down(0.0, 0.0), false);
    state.pointer_release();
    state.pointer_press(&down(5.0, 0.0), true);
    state.pointer_release();
    assert_eq!(state.selection.len(), 2);
    assert!(state.selection.contains(&a) && state.selection.contains(&b));
    state.pointer_press(&down(20.0, 20.0), false);
    assert!(state.selection.is_empty());
}

#[test]
fn rays_meet_planes_boxes_and_triangles() {
    let ray = Ray::new([0.0, 0.0, 5.0], [0.0, 0.0, -1.0]);
    assert_eq!(math::ray_plane(&ray, [0.0; 3], [0.0, 0.0, 1.0]), Some(5.0));
    assert_eq!(math::ray_box(&ray, [-1.0; 3], [1.0; 3]), Some(4.0));
    let t = math::ray_triangle(&ray, [-1.0, -1.0, 0.0], [2.0, -1.0, 0.0], [-1.0, 2.0, 0.0]);
    assert_eq!(t, Some(5.0));
    let sideways = Ray::new([0.0; 3], [1.0, 0.0, 0.0]);
    assert!(math::ray_plane(&sideways, [0.0; 3], [0.0, 0.0, 1.0]).is_none());
}
