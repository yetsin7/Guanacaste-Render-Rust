//! Unit tests of the materials domain.

use super::logic::*;
use super::models::MaterialState;

#[test]
fn reset_restores_defaults_but_keeps_the_tool() {
    let mut state = MaterialState {
        tool: 2,
        weathering: 0.9,
        ambient_occlusion: false,
        ..Default::default()
    };
    reset(&mut state);
    assert_eq!(state.tool, 2);
    assert_eq!(state.weathering, MaterialState::default().weathering);
    assert!(state.ambient_occlusion);
}

#[test]
fn filter_matches_substrings_ignoring_case() {
    assert!(matches_filter("Weathering Patina", "patina"));
    assert!(matches_filter("anything", "  "));
    assert!(!matches_filter("Metallic", "rough"));
}
