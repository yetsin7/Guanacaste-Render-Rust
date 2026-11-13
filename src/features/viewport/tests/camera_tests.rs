//! Camera model, math helpers and the frustum test.

use crate::features::viewport::logic::frustum;
use crate::features::viewport::logic::math::{self, Vec3};
use crate::features::viewport::logic::{CameraSlots, NavController};
use crate::features::viewport::models::Camera;
use crate::features::viewport::tests::bounds;

fn close(a: Vec3, b: Vec3) -> bool {
    math::length(math::sub(a, b)) < 1e-4
}

#[test]
fn default_camera_is_valid() {
    assert!(Camera::default().is_valid());
    let mut broken = Camera::default();
    broken.target = broken.position;
    assert!(!broken.is_valid());
    broken = Camera::default();
    broken.position[0] = f32::NAN;
    assert!(!broken.is_valid());
}

#[test]
fn basis_is_orthonormal_with_z_up() {
    let camera = Camera::looking_at([1.0, 2.0, 3.0], [6.0, -4.0, 5.0]);
    let (f, r, u) = (camera.forward(), camera.right(), camera.up());
    assert!(math::dot(f, r).abs() < 1e-5);
    assert!(math::dot(f, u).abs() < 1e-5);
    assert!(math::dot(r, u).abs() < 1e-5);
    assert!(r[2].abs() < 1e-6, "right must stay horizontal (no roll)");
    assert!(u[2] > 0.0, "up on screen points towards +Z");
}

#[test]
fn yaw_pitch_round_trips() {
    for (yaw, pitch) in [(0.3_f32, 0.2_f32), (-2.0, -0.7), (3.0, 1.2)] {
        let direction = Camera::direction(yaw, pitch);
        let camera = Camera::looking_at([0.0; 3], direction);
        let (y, p) = camera.yaw_pitch();
        assert!((y - yaw).abs() < 1e-4 && (p - pitch).abs() < 1e-4);
    }
}

#[test]
fn view_projection_puts_the_target_at_the_screen_center() {
    let camera = Camera::looking_at([10.0, 0.0, 2.0], [0.0, 0.0, 2.0]);
    let clip = math::transform_point(&camera.view_projection(1.5), camera.target);
    assert!((clip[0] / clip[3]).abs() < 1e-5);
    assert!((clip[1] / clip[3]).abs() < 1e-5);
    let ndc_z = clip[2] / clip[3];
    assert!((0.0..1.0).contains(&ndc_z));
}

#[test]
fn near_and_far_planes_map_to_depth_zero_and_one() {
    let camera = Camera::looking_at([0.0; 3], [0.0, 1.0, 0.0]);
    let m = camera.view_projection(1.0);
    let near = math::transform_point(&m, [0.0, camera.near, 0.0]);
    let far = math::transform_point(&m, [0.0, camera.far, 0.0]);
    assert!((near[2] / near[3]).abs() < 1e-4);
    assert!((far[2] / far[3] - 1.0).abs() < 1e-4);
}

#[test]
fn scene_right_of_the_view_projects_to_positive_x() {
    let camera = Camera::looking_at([0.0, -10.0, 0.0], [0.0, 0.0, 0.0]);
    // Looking along +Y, +X is on the right.
    let clip = math::transform_point(&camera.view_projection(1.0), [2.0, 0.0, 0.0]);
    assert!(clip[0] / clip[3] > 0.0);
    let clip = math::transform_point(&camera.view_projection(1.0), [0.0, 0.0, 2.0]);
    assert!(clip[1] / clip[3] > 0.0, "+Z is up on screen");
}

#[test]
fn frustum_culls_boxes_outside_the_view() {
    let camera = Camera::looking_at([0.0, -10.0, 0.0], [0.0, 0.0, 0.0]);
    let m = camera.view_projection(1.0);
    assert!(frustum::may_be_visible(&m, &bounds([-1.0; 3], [1.0; 3])));
    // Behind the camera.
    assert!(!frustum::may_be_visible(
        &m,
        &bounds([-1.0, -30.0, -1.0], [1.0, -20.0, 1.0])
    ));
    // Far to the side.
    assert!(!frustum::may_be_visible(
        &m,
        &bounds([500.0, 0.0, -1.0], [510.0, 1.0, 1.0])
    ));
    // Beyond the far plane.
    assert!(!frustum::may_be_visible(
        &m,
        &bounds([-1.0, 1e6, -1.0], [1.0, 2e6, 1.0])
    ));
    // A box that straddles the view is kept.
    assert!(frustum::may_be_visible(
        &m,
        &bounds([-1000.0, -5.0, -1000.0], [1000.0, 5.0, 1000.0])
    ));
}

#[test]
fn camera_slots_save_recall_and_reject_invalid_cameras() {
    let mut slots = CameraSlots::default();
    let camera = Camera::looking_at([1.0, 2.0, 3.0], [4.0, 5.0, 6.0]);
    assert!(slots.recall(2).is_none());
    assert!(slots.save(2, camera));
    assert_eq!(slots.recall(2), Some(camera));
    assert!(slots.is_filled(2) && !slots.is_filled(3));
    let mut bad = camera;
    bad.target = bad.position;
    assert!(!slots.save(0, bad));
    assert!(!slots.save(9, camera), "only nine slots exist");
    slots.clear();
    assert!(slots.recall(2).is_none());
}

#[test]
fn saved_camera_can_be_restored_by_the_controller() {
    let mut controller = NavController::default();
    let saved = Camera::looking_at([5.0, 5.0, 5.0], [0.0, 0.0, 0.0]);
    controller.go_to(saved, false);
    assert!(close(controller.camera.position, saved.position));
}
