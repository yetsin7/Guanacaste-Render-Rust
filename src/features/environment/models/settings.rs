//! The complete environment of a scene.

use super::calendar::CalendarDate;
use super::location::Location;
use super::sky::SkySettings;
use super::weather::WeatherSettings;
use serde::{Deserialize, Serialize};

/// Date, time, place, sky and weather of a scene. Serializable so it can live in a project file.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct EnvironmentSettings {
    /// Day being simulated.
    pub date: CalendarDate,
    /// Local clock time in hours, 0..24.
    pub hour: f32,
    /// Where the scene is.
    pub location: Location,
    /// Sky and clouds.
    pub sky: SkySettings,
    /// Precipitation, fog, wind and humidity.
    pub weather: WeatherSettings,
}

impl Default for EnvironmentSettings {
    fn default() -> Self {
        Self {
            date: CalendarDate::new(2026, 2, 15),
            hour: 16.5,
            location: Location::GUANACASTE,
            sky: SkySettings::default(),
            weather: WeatherSettings::default(),
        }
    }
}

impl EnvironmentSettings {
    /// Clamps every value to its valid range (use after loading a file or editing numbers by hand).
    pub fn sanitize(&mut self) {
        self.hour = self.hour.rem_euclid(24.0);
        self.location.latitude_deg = self.location.latitude_deg.clamp(-90.0, 90.0);
        self.location.longitude_deg = self.location.longitude_deg.clamp(-180.0, 180.0);
        self.location.utc_offset_hours = self.location.utc_offset_hours.clamp(-12.0, 14.0);
        self.date = CalendarDate::new(self.date.year, self.date.month, self.date.day);
        self.sky.sanitize();
        self.weather.sanitize();
    }
}
