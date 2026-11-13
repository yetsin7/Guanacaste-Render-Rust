//! Sky parameters: preset names, turbidity and clouds.

use serde::{Deserialize, Serialize};

/// Sky presets offered by the interface.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum SkyPreset {
    /// Clear morning sky.
    #[default]
    Clear,
    /// Sun at its highest point.
    Midday,
    /// Low sun in the evening.
    Sunset,
    /// Low sun in the morning.
    Sunrise,
    /// Complete, light grey cloud cover.
    Overcast,
    /// Dark clouds with rain and wind.
    Storm,
    /// Night with stars and moon.
    Night,
}

impl SkyPreset {
    /// Presets in display order.
    pub const ALL: [SkyPreset; 7] = [
        SkyPreset::Clear,
        SkyPreset::Midday,
        SkyPreset::Sunset,
        SkyPreset::Sunrise,
        SkyPreset::Overcast,
        SkyPreset::Storm,
        SkyPreset::Night,
    ];

    /// Localization key of the preset name.
    pub fn key(self) -> &'static str {
        match self {
            SkyPreset::Clear => "env.sky_clear",
            SkyPreset::Midday => "env.sky_midday",
            SkyPreset::Sunset => "env.sky_sunset",
            SkyPreset::Sunrise => "env.sky_sunrise",
            SkyPreset::Overcast => "env.sky_overcast",
            SkyPreset::Storm => "env.sky_storm",
            SkyPreset::Night => "env.sky_night",
        }
    }

    /// Position in [`SkyPreset::ALL`].
    pub fn index(self) -> usize {
        Self::ALL.iter().position(|p| *p == self).unwrap_or(0)
    }
}

/// Cloud layer parameters.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct CloudSettings {
    /// Fraction of the sky covered, 0..=1.
    pub coverage: f32,
    /// How opaque the clouds are, 0..=1.
    pub density: f32,
    /// Own drift speed, 0..=1 (added to the wind drift).
    pub speed: f32,
}

impl Default for CloudSettings {
    fn default() -> Self {
        Self {
            coverage: 0.25,
            density: 0.7,
            speed: 0.2,
        }
    }
}

/// Sky model parameters.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct SkySettings {
    /// Last preset applied (informational: the other fields hold the real values).
    pub preset: SkyPreset,
    /// Atmospheric turbidity, 2 (very clear) to 10 (hazy).
    pub turbidity: f32,
    /// Clouds.
    pub clouds: CloudSettings,
    /// Draw stars at night.
    pub stars: bool,
    /// Draw the moon.
    pub moon: bool,
    /// Animation clock in seconds; makes clouds drift. Not part of the look of a still image.
    pub time_seconds: f32,
}

impl Default for SkySettings {
    fn default() -> Self {
        Self {
            preset: SkyPreset::Clear,
            turbidity: 3.0,
            clouds: CloudSettings::default(),
            stars: true,
            moon: true,
            time_seconds: 0.0,
        }
    }
}

impl SkySettings {
    /// Lowest and highest turbidity.
    pub const TURBIDITY_RANGE: (f32, f32) = (2.0, 10.0);

    /// Clamps every value to its valid range.
    pub fn sanitize(&mut self) {
        let (low, high) = Self::TURBIDITY_RANGE;
        self.turbidity = self.turbidity.clamp(low, high);
        self.clouds.coverage = self.clouds.coverage.clamp(0.0, 1.0);
        self.clouds.density = self.clouds.density.clamp(0.0, 1.0);
        self.clouds.speed = self.clouds.speed.clamp(0.0, 1.0);
    }
}
