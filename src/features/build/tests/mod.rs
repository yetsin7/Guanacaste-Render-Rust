//! Unit tests of the build domain.

use super::logic::*;
use super::models::BuildState;
use crate::services::sample_data::build as sample;

#[test]
fn placing_advances_and_wraps_the_guide() {
    let guide = sample::guide();
    let mut state = BuildState::default();
    assert_eq!((state.step, state.placed), (1, 3));
    for _ in 0..5 {
        place_next(&mut state, &guide);
    }
    assert_eq!((state.step, state.placed), (2, 0));
}

#[test]
fn time_of_day_spans_six_to_twenty() {
    assert_eq!(format_hour(0.0), "06:00");
    assert_eq!(format_hour(1.0), "20:00");
    assert_eq!(format_hour(0.75), "16:30");
}
