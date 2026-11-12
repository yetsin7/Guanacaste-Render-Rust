//! Unit tests of the help domain.

use super::logic::*;
use super::models::HelpState;
use crate::services::sample_data::help as sample;

#[test]
fn tutorial_navigation_stays_in_range() {
    let total = sample::steps().len();
    let mut state = HelpState::default();
    previous_step(&mut state);
    assert_eq!(state.step, 0);
    for _ in 0..total + 3 {
        next_step(&mut state, total);
    }
    assert_eq!(state.step, total - 1);
}

#[test]
fn shortcut_search_checks_action_and_keys() {
    let list = sample::placement_shortcuts();
    assert_eq!(filter_shortcuts(&list, "terrain").len(), 1);
    assert_eq!(filter_shortcuts(&list, "space").len(), 1);
    assert_eq!(filter_shortcuts(&list, "").len(), list.len());
    assert!(filter_shortcuts(&list, "zzz").is_empty());
}

#[test]
fn tutorial_has_six_steps() {
    assert_eq!(sample::steps().len(), 6);
}
