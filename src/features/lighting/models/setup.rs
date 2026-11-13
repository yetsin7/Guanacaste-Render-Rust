//! Lights, groups, emissive materials and lighting scenes of a project, plus the editing state of the panel.

use super::Light;
use serde::{Deserialize, Serialize};

/// A named set of lights switched and dimmed together.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct LightGroup {
    /// Stable identifier (shares the id space of lights).
    pub id: u32,
    /// Display name.
    pub name: String,
    /// Switched on.
    pub enabled: bool,
    /// Dimmer applied on top of each member's own dimmer, 0..=1.
    pub dimmer: f32,
}

impl Default for LightGroup {
    fn default() -> Self {
        Self {
            id: 0,
            name: String::new(),
            enabled: true,
            dimmer: 1.0,
        }
    }
}

/// A material whose surfaces glow (screens, signage, light panels). The renderer adds
/// [`crate::features::lighting::logic::emissive_radiance`] to the surfaces using it.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct EmissiveMaterial {
    /// Material name as stored in the project.
    pub material: String,
    /// Luminance in candela per square meter (nits).
    pub luminance: f32,
    /// Color temperature in kelvin.
    pub kelvin: f32,
    /// Switched on.
    pub enabled: bool,
}

impl Default for EmissiveMaterial {
    fn default() -> Self {
        Self {
            material: String::new(),
            luminance: 500.0,
            kelvin: 6500.0,
            enabled: true,
        }
    }
}

/// On/off and dimmer of one light or group inside a scene.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct LightState {
    /// Light or group identifier.
    pub id: u32,
    /// Switched on.
    pub enabled: bool,
    /// Dimmer, 0..=1.
    pub dimmer: f32,
}

/// What a lighting scene is meant for; the environment uses it to pick sun and sky.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SceneKind {
    /// Daylight: artificial lights mostly off.
    Day,
    /// Night: artificial lights on.
    Night,
    /// Anything else.
    #[default]
    Custom,
}

/// A saved snapshot of which lights and groups are on and how dimmed.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct LightingScene {
    /// Display name.
    pub name: String,
    /// Purpose.
    pub kind: SceneKind,
    /// Light states.
    pub lights: Vec<LightState>,
    /// Group states.
    pub groups: Vec<LightState>,
}

/// Everything the lighting domain stores in a `.gr` project (the `lighting` field of the document).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct LightingSetup {
    /// Placed lights.
    pub lights: Vec<Light>,
    /// Light groups.
    pub groups: Vec<LightGroup>,
    /// Emissive materials.
    pub emissive: Vec<EmissiveMaterial>,
    /// Lighting scenes.
    pub scenes: Vec<LightingScene>,
    /// Index of the last applied scene.
    pub active_scene: Option<usize>,
    /// Next free identifier for lights and groups.
    pub next_id: u32,
}

impl Default for LightingSetup {
    fn default() -> Self {
        Self {
            lights: Vec::new(),
            groups: Vec::new(),
            emissive: Vec::new(),
            scenes: Vec::new(),
            active_scene: None,
            next_id: 1,
        }
    }
}

impl LightingSetup {
    /// True when nothing has been added (the document can omit it).
    pub fn is_empty(&self) -> bool {
        self.lights.is_empty()
            && self.groups.is_empty()
            && self.emissive.is_empty()
            && self.scenes.is_empty()
    }
}

/// State of the lighting panel: the project data plus transient editing fields.
#[derive(Debug, Clone, Default)]
pub struct LightingState {
    /// Project data.
    pub setup: LightingSetup,
    /// Index of the light being edited.
    pub selected: Option<usize>,
    /// Path typed for loading an IES file.
    pub ies_path: String,
    /// Name typed for a new group or scene.
    pub new_name: String,
    /// Localization key of the last message (IES errors, confirmations).
    pub message: Option<&'static str>,
}

impl LightingState {
    /// Panel state editing `setup`.
    pub fn new(setup: LightingSetup) -> Self {
        Self {
            setup,
            ..Self::default()
        }
    }
}
