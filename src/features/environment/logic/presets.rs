//! Sky and weather presets. Time-of-day presets depend on the scene's date and location.

use super::lighting::day_phase;
use super::lighting::DayPhase;
use super::sun::{sun_position, sun_times};
use crate::features::environment::models::{
    CloudSettings, EnvironmentSettings, Precipitation, SkyPreset, WeatherPreset, WeatherSettings,
};

fn wrap_hour(hour: f32) -> f32 {
    hour.rem_euclid(24.0)
}

/// Applies a weather preset: precipitation, fog, wind, humidity and a fitting cloud layer.
pub fn apply_weather_preset(env: &mut EnvironmentSettings, preset: WeatherPreset) {
    let keep = (env.weather.fog_color, env.weather.wind_direction_deg);
    let (clouds, weather) = match preset {
        WeatherPreset::Clear => (
            CloudSettings {
                coverage: 0.2,
                density: 0.7,
                speed: 0.2,
            },
            WeatherSettings {
                humidity: 0.45,
                ..WeatherSettings::default()
            },
        ),
        WeatherPreset::Cloudy => (
            CloudSettings {
                coverage: 0.9,
                density: 0.8,
                speed: 0.3,
            },
            WeatherSettings {
                wind_speed_ms: 4.0,
                humidity: 0.65,
                ..WeatherSettings::default()
            },
        ),
        WeatherPreset::Mist => (
            CloudSettings {
                coverage: 0.4,
                density: 0.5,
                speed: 0.1,
            },
            WeatherSettings {
                fog_density: 0.2,
                wind_speed_ms: 1.0,
                humidity: 0.85,
                ..WeatherSettings::default()
            },
        ),
        WeatherPreset::Fog => (
            CloudSettings {
                coverage: 0.7,
                density: 0.6,
                speed: 0.05,
            },
            WeatherSettings {
                fog_density: 0.6,
                wind_speed_ms: 0.5,
                humidity: 0.95,
                ..WeatherSettings::default()
            },
        ),
        WeatherPreset::Rain => (
            CloudSettings {
                coverage: 1.0,
                density: 0.9,
                speed: 0.4,
            },
            WeatherSettings {
                precipitation: Precipitation::Rain,
                precipitation_intensity: 0.6,
                fog_density: 0.08,
                wind_speed_ms: 6.0,
                humidity: 0.92,
                ..WeatherSettings::default()
            },
        ),
        WeatherPreset::Snow => (
            CloudSettings {
                coverage: 0.95,
                density: 0.75,
                speed: 0.15,
            },
            WeatherSettings {
                precipitation: Precipitation::Snow,
                precipitation_intensity: 0.5,
                fog_density: 0.12,
                wind_speed_ms: 3.0,
                humidity: 0.8,
                ..WeatherSettings::default()
            },
        ),
        WeatherPreset::Windy => (
            CloudSettings {
                coverage: 0.5,
                density: 0.6,
                speed: 0.8,
            },
            WeatherSettings {
                wind_speed_ms: 16.0,
                humidity: 0.4,
                ..WeatherSettings::default()
            },
        ),
    };
    env.sky.clouds = clouds;
    env.weather = WeatherSettings {
        preset,
        fog_color: keep.0,
        wind_direction_deg: keep.1,
        ..weather
    };
}

/// Applies a sky preset: moves the clock for time-of-day presets (using the scene's date and location), sets
/// turbidity and clouds, and for storms also the weather.
pub fn apply_sky_preset(env: &mut EnvironmentSettings, preset: SkyPreset) {
    let times = sun_times(env);
    let noon = times.map_or(12.0, |t| t.solar_noon);
    let daytime = !matches!(day_phase(sun_position(env).elevation_deg), DayPhase::Night);
    let (turbidity, coverage, density) = match preset {
        SkyPreset::Clear => (2.5, 0.12, 0.7),
        SkyPreset::Midday => (3.0, 0.2, 0.75),
        SkyPreset::Sunset => (4.0, 0.35, 0.8),
        SkyPreset::Sunrise => (3.5, 0.3, 0.75),
        SkyPreset::Overcast => (6.0, 1.0, 0.85),
        SkyPreset::Storm => (8.0, 1.0, 1.0),
        SkyPreset::Night => (2.5, 0.12, 0.6),
    };
    match preset {
        SkyPreset::Clear => env.hour = wrap_hour(noon - 2.5),
        SkyPreset::Midday => env.hour = wrap_hour(noon),
        SkyPreset::Sunset => env.hour = wrap_hour(times.map_or(18.0, |t| t.sunset - 0.2)),
        SkyPreset::Sunrise => env.hour = wrap_hour(times.map_or(6.0, |t| t.sunrise + 0.2)),
        SkyPreset::Overcast | SkyPreset::Storm if !daytime => env.hour = wrap_hour(noon - 1.0),
        SkyPreset::Overcast | SkyPreset::Storm => {}
        SkyPreset::Night => env.hour = wrap_hour(noon + 12.0),
    }
    if preset == SkyPreset::Storm {
        apply_weather_preset(env, WeatherPreset::Rain);
        env.weather.precipitation_intensity = 0.85;
        env.weather.wind_speed_ms = 14.0;
        env.weather.humidity = 0.97;
    } else {
        env.weather.precipitation = Precipitation::None;
        env.weather.precipitation_intensity = 0.0;
        if preset == SkyPreset::Overcast {
            env.weather.preset = WeatherPreset::Cloudy;
        }
    }
    env.sky.preset = preset;
    env.sky.turbidity = turbidity;
    env.sky.clouds.coverage = coverage;
    env.sky.clouds.density = density;
}
