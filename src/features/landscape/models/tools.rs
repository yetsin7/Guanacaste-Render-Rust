//! Settings of the landscape tools: brushes, procedural generation and vegetation scattering.

use serde::{Deserialize, Serialize};

/// What a stroke over the terrain does.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Tool {
    /// Raise the ground.
    #[default]
    Raise,
    /// Lower the ground.
    Lower,
    /// Pull the ground toward a target height.
    Flatten,
    /// Average each vertex with its neighbors.
    Smooth,
    /// Add seeded noise.
    Noise,
    /// Paint the selected soil layer.
    Paint,
    /// Place vegetation instances.
    Scatter,
    /// Remove vegetation instances.
    Erase,
}

impl Tool {
    /// Every tool in display order.
    pub const ALL: [Tool; 8] = [
        Tool::Raise,
        Tool::Lower,
        Tool::Flatten,
        Tool::Smooth,
        Tool::Noise,
        Tool::Paint,
        Tool::Scatter,
        Tool::Erase,
    ];

    /// Localization key of the tool name.
    pub fn label_key(self) -> &'static str {
        match self {
            Tool::Raise => "landscape.tool.raise",
            Tool::Lower => "landscape.tool.lower",
            Tool::Flatten => "landscape.tool.flatten",
            Tool::Smooth => "landscape.tool.smooth",
            Tool::Noise => "landscape.tool.noise",
            Tool::Paint => "landscape.tool.paint",
            Tool::Scatter => "landscape.tool.scatter",
            Tool::Erase => "landscape.tool.erase",
        }
    }

    /// Whether the tool changes heights.
    pub fn sculpts(self) -> bool {
        matches!(
            self,
            Tool::Raise | Tool::Lower | Tool::Flatten | Tool::Smooth | Tool::Noise
        )
    }
}

/// A circular brush.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Brush {
    /// Radius in meters.
    pub radius: f32,
    /// Strength, 0..=1.
    pub strength: f32,
    /// Soft fraction of the radius, 0 (hard edge) ..= 1 (soft from the center).
    pub falloff: f32,
    /// Target height of the flatten tool, in meters.
    pub target_height: f32,
    /// Feature size of the noise tool, in meters.
    pub noise_scale: f32,
    /// Seed of the noise tool.
    pub seed: u64,
}

impl Default for Brush {
    fn default() -> Self {
        Self {
            radius: 12.0,
            strength: 0.5,
            falloff: 0.6,
            target_height: 0.0,
            noise_scale: 8.0,
            seed: 7,
        }
    }
}

/// Fractal (fBm) terrain generation.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct FbmSettings {
    /// Random seed.
    pub seed: u64,
    /// Number of octaves, 1..=10.
    pub octaves: u32,
    /// Size of the largest features, in meters.
    pub scale: f32,
    /// Height difference between the lowest and highest possible point, in meters.
    pub amplitude: f32,
    /// Height of the lowest possible point, in meters.
    pub base_height: f32,
    /// Frequency multiplier between octaves.
    pub lacunarity: f32,
    /// Amplitude multiplier between octaves.
    pub gain: f32,
}

impl Default for FbmSettings {
    fn default() -> Self {
        Self {
            seed: 2026,
            octaves: 5,
            scale: 120.0,
            amplitude: 18.0,
            base_height: -4.0,
            lacunarity: 2.0,
            gain: 0.5,
        }
    }
}

/// Vegetation scattering.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct ScatterSettings {
    /// Library asset name of the placed plant.
    pub asset: String,
    /// Plants per 100 m².
    pub density: f32,
    /// Smallest uniform scale.
    pub scale_min: f32,
    /// Largest uniform scale.
    pub scale_max: f32,
    /// Steepest slope accepted, in degrees.
    pub max_slope_deg: f32,
    /// Lowest height accepted (keeps plants out of the water), in meters.
    pub min_height: f32,
    /// Random seed.
    pub seed: u64,
}

impl Default for ScatterSettings {
    fn default() -> Self {
        Self {
            asset: "Guanacaste".into(),
            density: 1.0,
            scale_min: 0.8,
            scale_max: 1.2,
            max_slope_deg: 30.0,
            min_height: 0.2,
            seed: 11,
        }
    }
}
