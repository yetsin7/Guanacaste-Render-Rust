//! Unit tests of the movie domain.

use super::logic::*;
use super::models::{MovieState, DURATION};

#[test]
fn timecodes_use_thirty_frames() {
    assert_eq!(timecode(4.4), "00:04:12");
    assert_eq!(timecode(0.0), "00:00:00");
    assert_eq!(timecode(75.5), "01:15:15");
}

#[test]
fn playback_stops_at_the_end_unless_looping() {
    let mut state = MovieState {
        playing: true,
        time: 13.9,
        ..Default::default()
    };
    advance(&mut state, 0.5);
    assert!(!state.playing);
    assert_eq!(state.time, DURATION);
    let mut looping = MovieState {
        playing: true,
        looping: true,
        time: 13.9,
        ..Default::default()
    };
    advance(&mut looping, 0.5);
    assert!(looping.playing && looping.time < 1.0);
}

#[test]
fn paused_playback_does_not_move() {
    let mut state = MovieState::default();
    advance(&mut state, 1.0);
    assert_eq!(state.time, 4.4);
}

#[test]
fn seeking_and_fraction_round_trip() {
    let mut state = MovieState::default();
    seek(&mut state, 0.5);
    assert_eq!(state.time, 7.0);
    assert_eq!(fraction(&state), 0.5);
}

#[test]
fn default_focal_is_twenty_eight_millimeters() {
    assert!((focal_length(MovieState::default().focal) - 28.0).abs() < 1e-3);
}
