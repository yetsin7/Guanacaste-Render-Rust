//! Settings rules: profile presets, derived readouts and search matching.

use super::models::{RenderProfile, SettingsState};
use crate::features::rendering::models::{RayTracingQuality, UpscaleMode};

/// Resolution scale in percent for a normalized value.
pub fn resolution_percent(t: f32) -> f32 {
    50.0 + 50.0 * t.clamp(0.0, 1.0)
}

/// Reflection cascade size in thousands for a normalized value.
pub fn reflection_k(t: f32) -> u32 {
    (1.0 + 7.0 * t.clamp(0.0, 1.0)).floor() as u32
}

/// Auto-save interval in minutes for a normalized value.
pub fn autosave_minutes(t: f32) -> u32 {
    (5.0 + 55.0 * t.clamp(0.0, 1.0)).round() as u32
}

/// Applies a render profile preset to the state.
pub fn apply_profile(state: &mut SettingsState, profile: RenderProfile) {
    state.profile = profile;
    state.applied = false;
    match profile {
        RenderProfile::LowLatency => {
            state.quality = RayTracingQuality::Low;
            state.upscale = UpscaleMode::Performance;
            state.resolution = 0.0;
            state.reflections = 0.0;
        }
        RenderProfile::Interactive => {
            state.quality = RayTracingQuality::Medium;
            state.upscale = UpscaleMode::Balanced;
            state.resolution = 0.6;
            state.reflections = 0.3;
        }
        RenderProfile::Production => {
            state.quality = RayTracingQuality::Ultra;
            state.upscale = UpscaleMode::Balanced;
            state.resolution = 1.0;
            state.reflections = 0.5;
        }
    }
}

/// Whether a settings entry whose visible texts are `texts` matches the search (case-insensitive).
pub fn matches_search(texts: &[&str], search: &str) -> bool {
    let needle = search.trim().to_lowercase();
    needle.is_empty()
        || texts
            .iter()
            .any(|text| text.to_lowercase().contains(&needle))
}
