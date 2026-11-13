//! Weather parameters: precipitation, fog, wind and humidity.

use super::color::Rgb;
use serde::{Deserialize, Serialize};

/// What falls from the sky.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum Precipitation {
    /// Dry.
    #[default]
    None,
    /// Rain.
    Rain,
    /// Snow.
    Snow,
}

impl Precipitation {
    /// Options in display order.
    pub const ALL: [Precipitation; 3] = [
        Precipitation::None,
        Precipitation::Rain,
        Precipitation::Snow,
    ];

    /// Localization key of the name.
    pub fn key(self) -> &'static str {
        match self {
            Precipitation::None => "env.precip_none",
            Precipitation::Rain => "env.precip_rain",
            Precipitation::Snow => "env.precip_snow",
        }
    }

    /// Position in [`Precipitation::ALL`].
    pub fn index(self) -> usize {
        Self::ALL.iter().position(|p| *p == self).unwrap_or(0)
    }
}

/// Weather presets offered by the interface.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum WeatherPreset {
    /// Dry and clear.
    #[default]
    Clear,
    /// Cloudy but dry.
    Cloudy,
    /// Light haze.
    Mist,
    /// Dense fog.
    Fog,
    /// Steady rain.
    Rain,
    /// Snowfall.
    Snow,
    /// Strong wind, dry.
    Windy,
}

impl WeatherPreset {
    /// Presets in display order.
    pub const ALL: [WeatherPreset; 7] = [
        WeatherPreset::Clear,
        WeatherPreset::Cloudy,
        WeatherPreset::Mist,
        WeatherPreset::Fog,
        WeatherPreset::Rain,
        WeatherPreset::Snow,
        WeatherPreset::Windy,
    ];

    /// Localization key of the preset name.
    pub fn key(self) -> &'static str {
        match self {
            WeatherPreset::Clear => "env.weather_clear",
            WeatherPreset::Cloudy => "env.weather_cloudy",
            WeatherPreset::Mist => "env.weather_mist",
            WeatherPreset::Fog => "env.weather_fog",
            WeatherPreset::Rain => "env.weather_rain",
            WeatherPreset::Snow => "env.weather_snow",
            WeatherPreset::Windy => "env.weather_windy",
        }
    }

    /// Position in [`WeatherPreset::ALL`].
    pub fn index(self) -> usize {
        Self::ALL.iter().position(|p| *p == self).unwrap_or(0)
    }
}

/// Weather parameters.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct WeatherSettings {
    /// Last preset applied (informational: the other fields hold the real values).
    pub preset: WeatherPreset,
    /// Kind of precipitation.
    pub precipitation: Precipitation,
    /// Precipitation intensity, 0..=1.
    pub precipitation_intensity: f32,
    /// Fog density, 0..=1 (1 is about 65 m of visibility).
    pub fog_density: f32,
    /// Fog color; `None` derives it from the horizon of the sky.
    pub fog_color: Option<Rgb>,
    /// Wind speed in m/s.
    pub wind_speed_ms: f32,
    /// Direction the wind blows from, in degrees clockwise from north.
    pub wind_direction_deg: f32,
    /// Relative humidity, 0..=1 (adds haze).
    pub humidity: f32,
}

impl Default for WeatherSettings {
    fn default() -> Self {
        Self {
            preset: WeatherPreset::Clear,
            precipitation: Precipitation::None,
            precipitation_intensity: 0.0,
            fog_density: 0.0,
            fog_color: None,
            wind_speed_ms: 2.0,
            wind_direction_deg: 70.0,
            humidity: 0.5,
        }
    }
}

impl WeatherSettings {
    /// Highest wind speed offered, in m/s.
    pub const MAX_WIND_MS: f32 = 30.0;

    /// Clamps every value to its valid range.
    pub fn sanitize(&mut self) {
        self.precipitation_intensity = self.precipitation_intensity.clamp(0.0, 1.0);
        self.fog_density = self.fog_density.clamp(0.0, 1.0);
        self.wind_speed_ms = self.wind_speed_ms.clamp(0.0, Self::MAX_WIND_MS);
        self.wind_direction_deg = self.wind_direction_deg.rem_euclid(360.0);
        self.humidity = self.humidity.clamp(0.0, 1.0);
    }
}
