//! Settings UI state.

use crate::features::rendering::models::{RayTracingQuality, UpscaleMode};

/// Render profiles of the profile bar.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RenderProfile {
    /// Fast draft.
    LowLatency,
    /// Interactive viewport.
    Interactive,
    /// Studio production quality.
    Production,
}

impl RenderProfile {
    /// Profiles in display order.
    pub const ALL: [RenderProfile; 3] = [
        RenderProfile::LowLatency,
        RenderProfile::Interactive,
        RenderProfile::Production,
    ];
}

/// UI state of the settings screen.
#[derive(Debug, Clone, PartialEq)]
pub struct SettingsState {
    /// Ray tracing quality.
    pub quality: RayTracingQuality,
    /// Frame generation mode.
    pub upscale: UpscaleMode,
    /// V-Sync lock.
    pub vsync: bool,
    /// Viewport resolution scale, normalized over 50..100 %.
    pub resolution: f32,
    /// Reflection cascade size, normalized over 1..8k.
    pub reflections: f32,
    /// Auto-save interval, normalized over 5..60 min.
    pub autosave: f32,
    /// Selected profile.
    pub profile: RenderProfile,
    /// Parameter search text.
    pub search: String,
    /// True after "Apply config" until a value changes.
    pub applied: bool,
}

impl Default for SettingsState {
    fn default() -> Self {
        Self {
            quality: RayTracingQuality::Ultra,
            upscale: UpscaleMode::Balanced,
            vsync: false,
            resolution: 1.0,
            reflections: 0.5,
            autosave: 0.1818,
            profile: RenderProfile::Production,
            search: String::new(),
            applied: false,
        }
    }
}
