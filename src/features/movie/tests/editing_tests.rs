//! Adding, removing and moving keys, channels and repairing data that came from a file.

use super::{animation, pose};
use crate::features::movie::models::{Animation, PathMode};

fn times(animation: &Animation) -> Vec<f64> {
    animation.camera.keys.iter().map(|key| key.time).collect()
}

#[test]
fn keys_are_inserted_in_time_order_and_a_key_at_the_same_instant_is_replaced() {
    let mut a = Animation::default();
    assert_eq!(a.add_camera_key(5.0, pose(5.0, 0.0, 0.0, 0.0)), 0);
    assert_eq!(a.add_camera_key(1.0, pose(1.0, 0.0, 0.0, 0.0)), 0);
    assert_eq!(a.add_camera_key(3.0, pose(3.0, 0.0, 0.0, 0.0)), 1);
    assert_eq!(times(&a), vec![1.0, 3.0, 5.0]);
    assert_eq!(a.add_camera_key(3.0004, pose(9.0, 0.0, 0.0, 0.0)), 1);
    assert_eq!(a.camera.keys.len(), 3);
    assert_eq!(a.camera.keys[1].pose.position[0], 9.0);
}

#[test]
fn moving_a_key_reorders_the_list_and_returns_the_new_index() {
    let mut a = animation(
        &[
            (0.0, pose(0.0, 0.0, 0.0, 0.0)),
            (2.0, pose(2.0, 0.0, 0.0, 0.0)),
            (4.0, pose(4.0, 0.0, 0.0, 0.0)),
        ],
        PathMode::Smooth,
    );
    assert_eq!(a.move_camera_key(0, 3.0), 1);
    assert_eq!(times(&a), vec![2.0, 3.0, 4.0]);
    assert_eq!(a.camera.keys[1].pose.position[0], 0.0);
    assert_eq!(a.move_camera_key(2, -5.0), 0);
    assert_eq!(a.camera.keys[0].time, 0.0);
    assert_eq!(a.move_camera_key(99, 1.0), 99);
}

#[test]
fn removing_a_key_reports_whether_it_existed() {
    let mut a = animation(&[(0.0, pose(0.0, 0.0, 0.0, 0.0))], PathMode::Smooth);
    assert!(!a.remove_camera_key(3));
    assert!(a.remove_camera_key(0));
    assert!(a.is_empty());
}

#[test]
fn the_live_camera_is_recorded_as_a_key_at_the_playhead() {
    let mut a = Animation::default();
    let live = pose(7.0, 8.0, 9.0, 33.0);
    let index = a.record_camera(2.5, &live);
    assert_eq!(a.camera.keys[index].time, 2.5);
    assert_eq!(a.camera.keys[index].pose.position, [7.0, 8.0, 9.0]);
    assert!((a.sample(2.5).euler_deg()[0] - 33.0).abs() < 1e-6);
}

#[test]
fn channels_get_unique_names_and_their_keys_stay_sorted() {
    let mut a = Animation::default();
    assert_eq!(a.add_channel("nubosidad"), 0);
    assert_eq!(a.add_channel("nubosidad"), 1);
    assert_eq!(a.channels[1].name, "nubosidad 2");
    assert_eq!(a.add_channel("  "), 2);
    assert_eq!(a.channels[2].name, "canal");
    assert_eq!(a.add_value_key(0, 4.0, 1.0), Some(0));
    assert_eq!(a.add_value_key(0, 2.0, 0.5), Some(0));
    assert_eq!(a.add_value_key(0, 2.0, 0.7), Some(0));
    assert_eq!(a.channels[0].keys.len(), 2);
    assert_eq!(a.move_value_key(0, 0, 9.0), 1);
    assert_eq!(a.channels[0].keys[0].time, 4.0);
    assert!(a.remove_value_key(0, 0));
    assert!(!a.remove_value_key(0, 5));
    assert_eq!(a.add_value_key(9, 1.0, 1.0), None);
    assert!(a.remove_channel(1) && !a.remove_channel(7));
}

#[test]
fn normalize_repairs_unsorted_and_non_finite_data() {
    let mut a = animation(
        &[
            (6.0, pose(f64::NAN, 0.0, 0.0, 0.0)),
            (1.0, pose(1.0, 0.0, 0.0, 0.0)),
        ],
        PathMode::Smooth,
    );
    a.fps = 0;
    a.speed = f64::NAN;
    a.duration = f64::INFINITY;
    a.camera.keys[0].pose.fov_deg = 900.0;
    a.normalize();
    assert_eq!(times(&a), vec![1.0, 6.0]);
    assert!(a
        .camera
        .keys
        .iter()
        .all(|k| k.pose.position.iter().all(|v| v.is_finite())));
    assert_eq!((a.fps, a.speed), (1, 1.0));
    assert!(a.duration.is_finite() && a.camera.keys[1].pose.fov_deg <= 170.0);
}

#[test]
fn sampling_unsorted_keys_does_not_panic_or_jump() {
    let mut a = animation(
        &[
            (0.0, pose(0.0, 0.0, 0.0, 0.0)),
            (4.0, pose(4.0, 0.0, 0.0, 0.0)),
        ],
        PathMode::Linear,
    );
    a.camera.keys.reverse();
    assert!((a.sample(2.0).position[0] - 2.0).abs() < 1e-9);
}
