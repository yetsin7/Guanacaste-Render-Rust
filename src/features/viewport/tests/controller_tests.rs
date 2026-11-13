//! Navigation controller: fly, look, orbit, pan, zoom, speed and animated moves.

use crate::features::viewport::logic::math;
use crate::features::viewport::logic::NavController;
use crate::features::viewport::models::{Camera, MoveKeys, NavInput};

fn controller() -> NavController {
    let mut c = NavController::new(Camera::looking_at([0.0, -10.0, 2.0], [0.0, 0.0, 2.0]));
    c.settings.inertia = false;
    c.settings.move_speed = 5.0;
    c
}

fn input() -> NavInput {
    NavInput {
        dt: 0.1,
        view_size: [800.0, 600.0],
        ..Default::default()
    }
}

fn run(c: &mut NavController, i: &NavInput, frames: usize) {
    for _ in 0..frames {
        c.update(i);
    }
}

#[test]
fn w_flies_forward_at_the_set_speed() {
    let mut c = controller();
    let mut i = input();
    i.keys = MoveKeys {
        forward: true,
        ..Default::default()
    };
    run(&mut c, &i, 10);
    assert!((c.camera.position[1] - -5.0).abs() < 1e-3, "{:?}", c.camera);
    assert!(
        (c.camera.distance() - 10.0).abs() < 1e-3,
        "target travels along"
    );
}

#[test]
fn e_goes_up_and_q_goes_down_in_world_z() {
    let mut c = controller();
    let mut i = input();
    i.keys.up = true;
    run(&mut c, &i, 10);
    assert!((c.camera.position[2] - 7.0).abs() < 1e-3);
    i.keys.up = false;
    i.keys.down = true;
    run(&mut c, &i, 20);
    assert!((c.camera.position[2] - -3.0).abs() < 1e-3);
}

#[test]
fn a_and_d_strafe() {
    let mut c = controller();
    let mut i = input();
    i.keys.right = true;
    run(&mut c, &i, 10);
    assert!(
        (c.camera.position[0] - 5.0).abs() < 1e-3,
        "looking +Y, right is +X"
    );
}

#[test]
fn diagonal_movement_is_not_faster() {
    let mut c = controller();
    let mut i = input();
    i.keys.forward = true;
    i.keys.right = true;
    run(&mut c, &i, 10);
    let moved = math::length(math::sub(c.camera.position, [0.0, -10.0, 2.0]));
    assert!((moved - 5.0).abs() < 1e-3);
}

#[test]
fn shift_speeds_up_and_ctrl_slows_only_while_flying_with_the_right_button() {
    let forward = MoveKeys {
        forward: true,
        ..Default::default()
    };
    let mut c = controller();
    let mut i = input();
    i.keys = forward;
    i.modifiers.shift = true;
    run(&mut c, &i, 5);
    assert!((c.camera.position[1] - (-10.0 + 5.0 * 4.0 * 0.5)).abs() < 1e-3);

    let mut c = controller();
    let mut i = input();
    i.keys = forward;
    i.modifiers.ctrl = true;
    run(&mut c, &i, 10);
    assert_eq!(
        c.camera.position,
        [0.0, -10.0, 2.0],
        "Ctrl+W is a command, not movement"
    );
    i.buttons.right = true;
    run(&mut c, &i, 10);
    assert!((c.camera.position[1] - (-10.0 + 5.0 * 0.25)).abs() < 1e-3);
}

#[test]
fn right_drag_turns_the_view_without_moving_the_eye() {
    let mut c = controller();
    let mut i = input();
    i.buttons.right = true;
    i.pointer_delta = [100.0, 0.0];
    c.update(&i);
    assert_eq!(c.camera.position, [0.0, -10.0, 2.0]);
    let (yaw, _) = c.camera.yaw_pitch();
    assert!(
        yaw < std::f32::consts::FRAC_PI_2,
        "dragging right turns right"
    );
    assert!((c.camera.distance() - 10.0).abs() < 1e-3);
}

#[test]
fn vertical_look_is_clamped_and_invertible() {
    let mut c = controller();
    let mut i = input();
    i.buttons.right = true;
    i.pointer_delta = [0.0, -100000.0];
    c.update(&i);
    let (_, pitch) = c.camera.yaw_pitch();
    assert!(pitch.to_degrees() <= 89.01 && pitch > 1.4);
    assert!(c.camera.is_valid());
    let mut c = controller();
    c.settings.invert_y = true;
    i.pointer_delta = [0.0, 20.0];
    c.update(&i);
    assert!(
        c.camera.yaw_pitch().1 > 0.0,
        "inverted: dragging down looks up"
    );
}

#[test]
fn middle_drag_orbits_around_the_target() {
    let mut c = controller();
    let target = c.camera.target;
    let mut i = input();
    i.buttons.middle = true;
    i.pointer_delta = [80.0, 30.0];
    c.update(&i);
    assert_eq!(c.camera.target, target);
    assert!((c.camera.distance() - 10.0).abs() < 1e-3);
    assert!(
        c.camera.position[0] < -0.1,
        "dragging right moves the eye to the left"
    );
}

#[test]
fn alt_left_drag_also_orbits() {
    let mut c = controller();
    let mut i = input();
    i.buttons.left = true;
    i.modifiers.alt = true;
    i.pointer_delta = [50.0, 0.0];
    c.update(&i);
    assert_ne!(c.camera.position, [0.0, -10.0, 2.0]);
    assert_eq!(c.camera.target, [0.0, 0.0, 2.0]);
}

#[test]
fn shift_middle_drag_pans_keeping_distance_and_direction() {
    let mut c = controller();
    let forward = c.camera.forward();
    let mut i = input();
    i.buttons.middle = true;
    i.modifiers.shift = true;
    i.pointer_delta = [60.0, 0.0];
    c.update(&i);
    assert!(
        c.camera.target[0] < -0.1,
        "scene follows the pointer: camera moves left"
    );
    assert!((c.camera.distance() - 10.0).abs() < 1e-3);
    assert!(math::length(math::sub(c.camera.forward(), forward)) < 1e-5);
}

#[test]
fn wheel_zooms_and_with_right_button_changes_speed() {
    let mut c = controller();
    let mut i = input();
    i.scroll = 1.0;
    c.update(&i);
    assert!(c.camera.distance() < 10.0);
    assert!(c.camera.distance() > 8.0);
    let mut c = controller();
    i.buttons.right = true;
    c.update(&i);
    assert!((c.camera.distance() - 10.0).abs() < 1e-3);
    assert!(c.speed() > 5.0);
    i.scroll = -100.0;
    c.update(&i);
    assert!(
        (c.speed() - c.settings.min_speed).abs() < 1e-6,
        "speed is clamped"
    );
}

#[test]
fn zooming_in_can_pass_through_the_pivot() {
    let mut c = controller();
    let mut i = input();
    i.scroll = 1.0;
    for _ in 0..200 {
        c.update(&i);
    }
    assert!(
        c.camera.position[1] > 0.0,
        "eye went past the original target"
    );
    assert!(c.camera.is_valid());
}

#[test]
fn inertia_glides_to_a_stop_and_reports_rest() {
    let mut c = controller();
    c.settings.inertia = true;
    let mut i = input();
    i.keys.forward = true;
    run(&mut c, &i, 10);
    assert!(c.is_animating());
    i.keys.forward = false;
    let before = c.camera.position[1];
    c.update(&i);
    assert!(c.camera.position[1] > before, "still gliding after release");
    let mut busy = true;
    for _ in 0..200 {
        busy = c.update(&i);
        if !busy {
            break;
        }
    }
    assert!(!busy && !c.is_animating());
}

#[test]
fn animated_move_reaches_the_goal_and_input_cancels_it() {
    let goal = Camera::looking_at([20.0, 20.0, 20.0], [0.0, 0.0, 0.0]);
    let mut c = controller();
    c.go_to(goal, true);
    assert!(c.is_animating());
    run(&mut c, &input(), 10);
    assert!(math::length(math::sub(c.camera.position, goal.position)) < 1e-3);
    assert!(!c.is_animating());

    let mut c = controller();
    c.go_to(goal, true);
    let mut i = input();
    i.keys.forward = true;
    c.update(&i);
    i.keys.forward = false;
    run(&mut c, &i, 20);
    assert!(math::length(math::sub(c.camera.position, goal.position)) > 1.0);
}

#[test]
fn garbage_input_never_breaks_the_camera() {
    let mut c = controller();
    let mut i = input();
    i.dt = f32::NAN;
    i.scroll = f32::INFINITY;
    i.pointer_delta = [f32::NAN, 1.0e30];
    i.buttons.right = true;
    i.keys.forward = true;
    for _ in 0..5 {
        c.update(&i);
    }
    assert!(c.camera.is_valid());
}

#[test]
fn invalid_goal_is_ignored() {
    let mut c = controller();
    let before = c.camera;
    let mut bad = before;
    bad.position = [f32::NAN; 3];
    c.go_to(bad, false);
    assert_eq!(c.camera, before);
}
