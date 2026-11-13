//! Fog, mist and precipitation haze: extinction, visibility and distance attenuation.

use super::sky::SkyModel;
use crate::features::environment::models::{
    EnvironmentSettings, Precipitation, Rgb, WeatherSettings,
};

/// Koschmieder constant: visibility is where contrast falls to 2 %.
const KOSCHMIEDER: f32 = 3.912;

/// Total extinction coefficient in 1/m: clear-air haze with humidity, fog and precipitation added.
pub fn extinction(weather: &WeatherSettings) -> f32 {
    let humidity = weather.humidity.clamp(0.0, 1.0);
    let intensity = weather.precipitation_intensity.clamp(0.0, 1.0);
    let density = weather.fog_density.clamp(0.0, 1.0);
    let precipitation = match weather.precipitation {
        Precipitation::None => 0.0,
        Precipitation::Rain => 0.004 * intensity,
        Precipitation::Snow => 0.006 * intensity,
    };
    0.000_05 + 0.000_3 * humidity.powi(3) + 0.06 * density * density + precipitation
}

/// Meteorological visibility in meters.
pub fn visibility_m(weather: &WeatherSettings) -> f32 {
    KOSCHMIEDER / extinction(weather)
}

/// Fraction of the scene color replaced by fog at `distance_m` meters (0 = clear, 1 = fully fogged),
/// from Beer-Lambert extinction.
pub fn fog_factor(weather: &WeatherSettings, distance_m: f32) -> f32 {
    1.0 - (-extinction(weather) * distance_m.max(0.0)).exp()
}

/// Color of the fog: the explicit color, or the horizon of the sky softened toward grey.
pub fn fog_color(env: &EnvironmentSettings) -> Rgb {
    if let Some(color) = env.weather.fog_color {
        return color;
    }
    let horizon = SkyModel::new(env).horizon_color();
    let milk = Rgb::splat(horizon.luma());
    horizon.mix(milk, (0.3 + env.weather.fog_density).min(0.8))
}

/// Blends a shaded surface color with the fog at `distance_m` meters.
pub fn apply_fog(color: Rgb, env: &EnvironmentSettings, distance_m: f32) -> Rgb {
    color.mix(fog_color(env), fog_factor(&env.weather, distance_m))
}
