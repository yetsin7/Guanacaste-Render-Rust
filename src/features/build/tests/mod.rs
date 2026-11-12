//! Unit tests of the build domain.

use super::logic::*;
use super::models::{BuildState, SliderSpec};
use crate::services::sample_data::build as sample;

const BRIGHTNESS: SliderSpec = SliderSpec {
    min: 0.0,
    max: 1923.0,
    unit: " cd",
    decimals: 0,
};

#[test]
fn thousands_separators_group_digits() {
    assert_eq!(with_thousands(1250.0, 0), "1,250");
    assert_eq!(with_thousands(999.0, 0), "999");
    assert_eq!(with_thousands(1234567.0, 0), "1,234,567");
    assert_eq!(with_thousands(-18.8, 2), "-18.80");
}

#[test]
fn slider_values_follow_the_range() {
    assert_eq!(value_of(&BRIGHTNESS, 0.0), 0.0);
    assert_eq!(value_of(&BRIGHTNESS, 1.5), 1923.0);
    assert_eq!(format_value(&BRIGHTNESS, 0.65), "1,250 cd");
}

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
