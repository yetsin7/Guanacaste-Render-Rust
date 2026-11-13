//! Values produced by `sample`: curves, orientation, pauses, constant speed and channels.

use super::{animation, near, pose};
use crate::features::movie::models::quat::{self, angle_between};
use crate::features::movie::models::{CameraPose, Ease, PathMode, Timing, ValueKey};

#[test]
fn an_empty_track_gives_the_default_pose_and_one_key_holds_everywhere() {
    let empty = animation(&[], PathMode::Smooth);
    assert_eq!(empty.sample(3.0), CameraPose::default());
    let single = animation(&[(2.0, pose(1.0, 2.0, 3.0, 40.0))], PathMode::Smooth);
    for time in [0.0, 2.0, 9.0] {
        assert!(near(single.sample(time).position[1], 2.0, 1e-12));
    }
}

#[test]
fn linear_paths_interpolate_position_and_orientation_halfway() {
    let a = animation(
        &[
            (0.0, pose(0.0, 0.0, 0.0, 0.0)),
            (2.0, pose(10.0, 0.0, 4.0, 90.0)),
        ],
        PathMode::Linear,
    );
    let middle = a.sample(1.0);
    assert!(near(middle.position[0], 5.0, 1e-9) && near(middle.position[2], 2.0, 1e-9));
    assert!(near(middle.euler_deg()[0], 45.0, 1e-6));
    assert!(near(middle.fov_deg, 60.0, 1e-9));
}

#[test]
fn the_end_poses_are_held_outside_the_keys() {
    let a = animation(
        &[
            (1.0, pose(0.0, 0.0, 0.0, 0.0)),
            (3.0, pose(8.0, 0.0, 0.0, 0.0)),
        ],
        PathMode::Smooth,
    );
    assert_eq!(a.sample(0.0).position, [0.0, 0.0, 0.0]);
    assert!(near(a.sample(3.0).position[0], 8.0, 1e-9));
    assert!(near(a.sample(1.0).position[0], 0.0, 1e-9));
}

#[test]
fn easing_curves_shape_the_segment() {
    let mut a = animation(
        &[
            (0.0, pose(0.0, 0.0, 0.0, 0.0)),
            (2.0, pose(8.0, 0.0, 0.0, 0.0)),
        ],
        PathMode::Linear,
    );
    a.camera.keys[0].ease = Ease::EaseIn;
    assert!(near(a.sample(1.0).position[0], 8.0 * 0.125, 1e-9));
    a.camera.keys[0].ease = Ease::EaseOut;
    assert!(near(a.sample(1.0).position[0], 8.0 * 0.875, 1e-9));
    a.camera.keys[0].ease = Ease::EaseInOut;
    assert!(near(a.sample(1.0).position[0], 4.0, 1e-9));
    a.camera.keys[0].ease = Ease::Hold;
    assert!(near(a.sample(1.999).position[0], 0.0, 1e-9));
    assert!(near(a.sample(2.0).position[0], 8.0, 1e-9));
}

#[test]
fn a_smooth_path_passes_through_every_key_and_straight_keys_stay_straight() {
    let keys: Vec<_> = (0..5)
        .map(|i| (i as f64 * 1.5, pose(i as f64 * 3.0, 0.0, 1.0, 0.0)))
        .collect();
    let a = animation(&keys, PathMode::Smooth);
    for (time, expected) in &keys {
        assert!(near(
            a.sample(*time).position[0],
            expected.position[0],
            1e-9
        ));
    }
    for step in 0..=60 {
        let sample = a.sample(step as f64 * 0.1);
        assert!(near(sample.position[1], 0.0, 1e-9) && near(sample.position[2], 1.0, 1e-9));
    }
}

#[test]
fn a_smooth_path_bends_through_a_corner_without_overshooting_the_keys_much() {
    let a = animation(
        &[
            (0.0, pose(0.0, 0.0, 0.0, 0.0)),
            (1.0, pose(10.0, 0.0, 0.0, 0.0)),
            (2.0, pose(10.0, 10.0, 0.0, 0.0)),
        ],
        PathMode::Smooth,
    );
    let middle = a.sample(1.0);
    assert!(near(middle.position[0], 10.0, 1e-9) && near(middle.position[1], 0.0, 1e-9));
    let before = a.sample(0.9);
    assert!(before.position[1] > 0.0 || before.position[0] < 10.0);
    for step in 0..=20 {
        let p = a.sample(step as f64 * 0.1).position;
        assert!((-1.5..=11.5).contains(&p[0]) && (-1.5..=11.5).contains(&p[1]));
    }
}

#[test]
fn orientation_takes_the_short_way_even_when_a_key_stores_the_negated_quaternion() {
    let mut a = animation(
        &[
            (0.0, pose(0.0, 0.0, 0.0, 170.0)),
            (2.0, pose(0.0, 0.0, 0.0, -170.0)),
        ],
        PathMode::Linear,
    );
    let middle = a.sample(1.0).orientation;
    let expected = quat::from_euler_deg(180.0, 0.0, 0.0);
    assert!(angle_between(middle, expected) < 1e-6);
    let flipped = a.camera.keys[1].pose.orientation.map(|v| -v);
    a.camera.keys[1].pose.orientation = flipped;
    assert!(angle_between(a.sample(1.0).orientation, expected) < 1e-6);
}

#[test]
fn euler_angles_and_focal_length_round_trip() {
    for (yaw, pitch, roll) in [
        (30.0, 10.0, -5.0),
        (-120.0, 45.0, 20.0),
        (170.0, -60.0, 0.0),
    ] {
        let pose = CameraPose::from_euler([0.0; 3], yaw, pitch, roll, 60.0);
        let [y, p, r] = pose.euler_deg();
        assert!(near(y, yaw, 1e-6) && near(p, pitch, 1e-6) && near(r, roll, 1e-6));
    }
    let mut pose = CameraPose::default();
    pose.set_focal_mm(50.0);
    assert!(near(pose.fov_deg, 39.597, 1e-2));
    assert!(near(pose.focal_mm(), 50.0, 1e-9));
    let forward = CameraPose::from_euler([0.0; 3], 90.0, 0.0, 0.0, 60.0).forward();
    assert!(near(forward[0], -1.0, 1e-9) && near(forward[1], 0.0, 1e-9));
}

#[test]
fn a_pause_keeps_the_camera_still_and_then_moves_on() {
    let mut a = animation(
        &[
            (0.0, pose(0.0, 0.0, 0.0, 0.0)),
            (4.0, pose(8.0, 0.0, 0.0, 0.0)),
        ],
        PathMode::Linear,
    );
    a.camera.keys[0].pause = 2.0;
    assert!(near(a.sample(1.9).position[0], 0.0, 1e-9));
    assert!(near(a.sample(3.0).position[0], 4.0, 1e-9));
    assert!(near(a.sample(4.0).position[0], 8.0, 1e-9));
    a.camera.keys[1].pause = 3.0;
    a.duration = 0.0;
    assert!(near(a.total_duration(), 7.0, 1e-9));
}

#[test]
fn constant_speed_spreads_the_keys_by_distance() {
    let mut a = animation(
        &[
            (0.0, pose(0.0, 0.0, 0.0, 0.0)),
            (5.0, pose(1.0, 0.0, 0.0, 0.0)),
            (10.0, pose(10.0, 0.0, 0.0, 0.0)),
        ],
        PathMode::Linear,
    );
    assert!(near(a.sample(5.0).position[0], 1.0, 1e-9));
    a.camera.timing = Timing::ConstantSpeed;
    for time in [0.0, 2.5, 5.0, 7.5, 10.0] {
        assert!(near(a.sample(time).position[0], time, 1e-6), "t = {time}");
    }
}

#[test]
fn smooth_constant_speed_keeps_the_pace_nearly_even() {
    let mut a = animation(
        &[
            (0.0, pose(0.0, 0.0, 0.0, 0.0)),
            (1.0, pose(2.0, 0.0, 0.0, 0.0)),
            (9.0, pose(20.0, 6.0, 0.0, 0.0)),
            (10.0, pose(22.0, 6.0, 0.0, 0.0)),
        ],
        PathMode::Smooth,
    );
    a.camera.timing = Timing::ConstantSpeed;
    let mut speeds = Vec::new();
    for step in 0..100 {
        let (p, q) = (
            a.sample(step as f64 * 0.1),
            a.sample(step as f64 * 0.1 + 0.1),
        );
        let d: f64 = (0..3)
            .map(|i| (q.position[i] - p.position[i]).powi(2))
            .sum();
        speeds.push(d.sqrt() / 0.1);
    }
    let mean = speeds.iter().sum::<f64>() / speeds.len() as f64;
    assert!(
        speeds.iter().all(|s| (s - mean).abs() < 0.35 * mean),
        "{speeds:?}"
    );
}

#[test]
fn channels_interpolate_hold_and_clamp() {
    let mut a = animation(&[], PathMode::Smooth);
    let channel = a.add_channel("hora_del_dia");
    a.add_value_key(channel, 0.0, 6.0);
    a.add_value_key(channel, 10.0, 18.0);
    assert_eq!(a.sample_channel("hora_del_dia", 5.0), Some(12.0));
    assert_eq!(a.sample_channel("hora_del_dia", 99.0), Some(18.0));
    assert_eq!(a.sample_channel("no_existe", 1.0), None);
    a.channels[0].keys[0].ease = Ease::Hold;
    assert_eq!(a.sample_channel("hora_del_dia", 9.9), Some(6.0));
    a.channels[0].keys[0] = ValueKey {
        time: 0.0,
        value: 6.0,
        ease: Ease::EaseIn,
    };
    assert_eq!(
        a.sample_channel("hora_del_dia", 5.0),
        Some(6.0 + 12.0 * 0.125)
    );
    assert_eq!(
        a.sample_channels(0.0),
        vec![("hora_del_dia".to_owned(), 6.0)]
    );
}
