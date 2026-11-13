//! Environment entities: color, calendar, location, sky, weather and the combined settings.

pub mod calendar;
pub mod color;
pub mod location;
pub mod settings;
pub mod sky;
pub mod weather;

pub use calendar::CalendarDate;
pub use color::Rgb;
pub use location::{Location, LocationPreset, LOCATION_PRESETS};
pub use settings::EnvironmentSettings;
pub use sky::{CloudSettings, SkyPreset, SkySettings};
pub use weather::{Precipitation, WeatherPreset, WeatherSettings};
