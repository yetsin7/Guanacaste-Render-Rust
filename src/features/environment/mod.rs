//! Environment domain: sun position, sky, clouds and weather.
//!
//! The pure part (`models` and `logic`) has no UI dependency so the CPU renderer and the viewport preview can ask
//! for the sky color in any direction ([`sample_sky`]), the sun light ([`sun_light`]) or the fog amount at a
//! distance ([`fog_factor`]).

pub mod components;
pub mod logic;
pub mod models;

#[cfg(test)]
mod tests;

pub use logic::fog::{apply_fog, fog_color, fog_factor, visibility_m};
pub use logic::lighting::{day_phase, sun_light, DayPhase, SunLight};
pub use logic::presets::{apply_sky_preset, apply_weather_preset};
pub use logic::sky::{sample_sky, SkyModel};
pub use logic::sun::{sun_direction, sun_position, sun_times, SunPosition, SunTimes};
pub use models::{EnvironmentSettings, Precipitation, Rgb, SkyPreset, WeatherPreset};
