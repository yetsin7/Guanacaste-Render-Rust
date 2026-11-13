//! Timecodes, the playhead, loops, speed and frame times.

use super::{animation, near, pose};
use crate::features::movie::logic::*;
use crate::features::movie::models::{Animation, MovieState, PathMode};

fn two_keys() -> Animation {
    let mut a = animation(
        &[
            (0.0, pose(0.0, 0.0, 0.0, 0.0)),
            (4.0, pose(8.0, 0.0, 0.0, 0.0)),
        ],
        PathMode::Linear,
    );
    a.duration = 4.0;
    a
}

#[test]
fn timecodes_follow_the_frame_rate() {
    assert_eq!(timecode(4.4, 30), "00:04:12");
    assert_eq!(timecode(0.0, 30), "00:00:00");
    assert_eq!(timecode(75.5, 30), "01:15:15");
    assert_eq!(timecode(1.5, 24), "00:01:12");
    assert_eq!(timecode(2.0, 0), "00:02:00");
}

#[test]
fn playback_stops_at_the_end_unless_looping() {
    let mut a = two_keys();
    let mut state = MovieState {
        playing: true,
        time: 3.9,
        ..Default::default()
    };
    advance(&mut state, &a, 0.5);
    assert!(!state.playing);
    assert_eq!(state.time, 4.0);
    a.looping = true;
    state.playing = true;
    state.time = 3.9;
    advance(&mut state, &a, 0.5);
    assert!(state.playing && near(state.time, 0.4, 1e-9));
}

#[test]
fn paused_playback_does_not_move_and_speed_scales_it() {
    let mut a = two_keys();
    let mut state = MovieState::default();
    advance(&mut state, &a, 1.0);
    assert_eq!(state.time, 0.0);
    state.playing = true;
    a.speed = 2.0;
    advance(&mut state, &a, 0.5);
    assert!(near(state.time, 1.0, 1e-12));
}

#[test]
fn seeking_toggling_and_key_jumps() {
    let mut a = two_keys();
    a.add_camera_key(1.0, pose(2.0, 0.0, 0.0, 0.0));
    let mut state = MovieState::default();
    seek(&mut state, &a, 0.5);
    assert_eq!((state.time, fraction(&state, &a)), (2.0, 0.5));
    state.time = 4.0;
    toggle_play(&mut state, &a);
    assert!(state.playing && state.time == 0.0);
    toggle_play(&mut state, &a);
    assert!(!state.playing);
    assert_eq!(next_key_time(&a, 0.0), 1.0);
    assert_eq!(next_key_time(&a, 1.0), 4.0);
    assert_eq!(next_key_time(&a, 4.0), 4.0);
    assert_eq!(previous_key_time(&a, 4.0), 1.0);
    assert_eq!(previous_key_time(&a, 0.5), 0.0);
}

#[test]
fn frame_counts_and_times_follow_fps_speed_and_loop() {
    let mut a = two_keys();
    a.fps = 25;
    assert_eq!(a.frame_count(), 101);
    assert!(near(a.frame_time(50), 2.0, 1e-12));
    assert!(near(a.frame_time(100), 4.0, 1e-12));
    a.looping = true;
    assert_eq!(a.frame_count(), 100);
    a.looping = false;
    a.speed = 2.0;
    assert!(near(a.video_duration(), 2.0, 1e-12));
    assert_eq!(a.frame_count(), 51);
    assert!(near(a.frame_time(25), 2.0, 1e-12));
    assert!(near(a.sample(a.frame_time(50)).position[0], 8.0, 1e-9));
}

#[test]
fn the_duration_grows_to_cover_the_last_key_and_never_exceeds_the_limit() {
    let mut a = two_keys();
    a.duration = 1.0;
    assert_eq!(a.total_duration(), 4.0);
    a.add_camera_key(9_999.0, pose(0.0, 0.0, 0.0, 0.0));
    assert_eq!(a.total_duration(), 3600.0);
    assert!(Animation::default().is_empty());
}
