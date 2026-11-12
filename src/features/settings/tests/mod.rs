//! Unit tests of the settings domain.

use super::logic::*;
use super::models::*;
use crate::features::rendering::models::RayTracingQuality;

#[test]
fn defaults_match_the_prototype_readouts() {
    let state = SettingsState::default();
    assert_eq!(resolution_percent(state.resolution), 100.0);
    assert_eq!(reflection_k(state.reflections), 4);
    assert_eq!(autosave_minutes(state.autosave), 15);
}

#[test]
fn profiles_change_quality_and_scale() {
    let mut state = SettingsState::default();
    apply_profile(&mut state, RenderProfile::LowLatency);
    assert_eq!(state.quality, RayTracingQuality::Low);
    assert_eq!(resolution_percent(state.resolution), 50.0);
    apply_profile(&mut state, RenderProfile::Production);
    assert_eq!(state.quality, RayTracingQuality::Ultra);
}

#[test]
fn search_matches_any_text_case_insensitively() {
    assert!(matches_search(&["Ray Tracing", "DXR"], "dxr"));
    assert!(matches_search(&["x"], ""));
    assert!(!matches_search(&["V-Sync"], "vram"));
}
