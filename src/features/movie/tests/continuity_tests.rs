//! Continuity of the sampled camera: no jumps in position or orientation, smooth velocity through the keys.

use super::{animation, near, pose};
use crate::features::movie::models::quat::angle_between;
use crate::features::movie::models::{Animation, CameraPose, Ease, PathMode, Timing};

/// Five keys with uneven times, turns across the 180-degree seam and a changing height and field of view.
fn winding(path: PathMode) -> Animation {
    let mut keys = vec![
        (0.0, pose(0.0, 0.0, 1.5, 0.0)),
        (1.5, pose(6.0, 2.0, 2.0, 120.0)),
        (4.0, pose(9.0, 9.0, 3.5, -150.0)),
        (5.0, pose(2.0, 12.0, 3.0, 170.0)),
        (9.0, pose(-4.0, 6.0, 1.5, 90.0)),
    ];
    keys[2].1.fov_deg = 35.0;
    animation(&keys, path)
}

fn max_steps(animation: &Animation, dt: f64) -> (f64, f64) {
    let total = animation.total_duration();
    let count = (total / dt) as usize;
    let (mut position, mut angle) = (0.0_f64, 0.0_f64);
    let mut last = animation.sample(0.0);
    for step in 1..=count {
        let now = animation.sample(step as f64 * dt);
        let d: f64 = (0..3)
            .map(|i| (now.position[i] - last.position[i]).powi(2))
            .sum();
        position = position.max(d.sqrt());
        angle = angle.max(angle_between(now.orientation, last.orientation));
        last = now;
    }
    (position, angle)
}

fn velocity(animation: &Animation, time: f64) -> [f64; 3] {
    let h = 1e-5;
    let (a, b) = (animation.sample(time - h), animation.sample(time + h));
    [0, 1, 2].map(|i| (b.position[i] - a.position[i]) / (2.0 * h))
}

#[test]
fn smooth_and_linear_paths_have_no_position_or_orientation_jumps() {
    for path in [PathMode::Smooth, PathMode::Linear] {
        let a = winding(path);
        let (position, angle) = max_steps(&a, 0.002);
        assert!(position < 0.06, "{path:?}: position step {position}");
        assert!(angle < 0.03, "{path:?}: orientation step {angle}");
    }
}

#[test]
fn the_smooth_path_velocity_is_continuous_through_every_key() {
    let a = winding(PathMode::Smooth);
    for key in &a.camera.keys[1..a.camera.keys.len() - 1] {
        let before = velocity(&a, key.time - 2e-4);
        let after = velocity(&a, key.time + 2e-4);
        for axis in 0..3 {
            assert!(
                near(before[axis], after[axis], 0.02),
                "t = {}, axis {axis}: {} vs {}",
                key.time,
                before[axis],
                after[axis]
            );
        }
    }
}

#[test]
fn the_linear_path_velocity_does_jump_at_corners_while_the_smooth_one_does_not() {
    let mut keys = winding(PathMode::Linear);
    let corner = keys.camera.keys[2].time;
    let (before, after) = (
        velocity(&keys, corner - 0.01),
        velocity(&keys, corner + 0.01),
    );
    assert!((0..3).any(|i| (before[i] - after[i]).abs() > 0.5));
    keys.camera.path = PathMode::Smooth;
    let (before, after) = (
        velocity(&keys, corner - 0.01),
        velocity(&keys, corner + 0.01),
    );
    assert!((0..3).all(|i| (before[i] - after[i]).abs() < 0.5));
}

#[test]
fn the_orientation_is_the_same_when_keys_store_the_negated_quaternion() {
    let reference = winding(PathMode::Smooth);
    let mut flipped = reference.clone();
    for (index, key) in flipped.camera.keys.iter_mut().enumerate() {
        if index % 2 == 1 {
            key.pose.orientation = key.pose.orientation.map(|v| -v);
        }
    }
    for step in 0..=90 {
        let time = step as f64 * 0.1;
        let (a, b) = (reference.sample(time), flipped.sample(time));
        assert!(
            angle_between(a.orientation, b.orientation) < 1e-9,
            "t = {time}"
        );
    }
}

#[test]
fn pauses_and_easing_keep_the_pose_continuous() {
    let mut a = winding(PathMode::Smooth);
    a.camera.keys[1].pause = 0.8;
    a.camera.keys[2].ease = Ease::EaseInOut;
    a.camera.keys[3].pause = 0.5;
    a.camera.keys[3].ease = Ease::EaseOut;
    let (position, angle) = max_steps(&a, 0.002);
    assert!(position < 0.1 && angle < 0.05, "{position} {angle}");
}

#[test]
fn constant_speed_timing_keeps_the_pose_continuous() {
    for path in [PathMode::Smooth, PathMode::Linear] {
        let mut a = winding(path);
        a.camera.timing = Timing::ConstantSpeed;
        let (position, angle) = max_steps(&a, 0.002);
        assert!(
            position < 0.1 && angle < 0.06,
            "{path:?}: {position} {angle}"
        );
        assert!(near(a.sample(0.0).position[0], 0.0, 1e-9));
        assert!(near(a.sample(9.0).position[0], -4.0, 1e-9));
    }
}

#[test]
fn a_looping_animation_wraps_the_time_and_never_leaves_the_path() {
    let mut a = winding(PathMode::Smooth);
    a.looping = true;
    let total = a.total_duration();
    let inside: CameraPose = a.sample(2.5);
    assert_eq!(a.sample(2.5 + total), inside);
    assert_eq!(a.sample(2.5 - total), inside);
    a.looping = false;
    assert_eq!(a.sample(total + 5.0), a.sample(total));
    assert_eq!(a.sample(-3.0), a.sample(0.0));
}
