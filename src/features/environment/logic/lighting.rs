//! Sun light: color and intensity by elevation, day phases and ambient light.

use super::math::{smoothstep, Vec3};
use super::sky::SkyModel;
use super::sun::sun_position;
use crate::features::environment::models::{EnvironmentSettings, Rgb};

/// Direct sunlight reaching the scene, plus the ambient light of the sky.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SunLight {
    /// Unit vector pointing from the scene toward the sun.
    pub direction: Vec3,
    /// Sun elevation in degrees.
    pub elevation_deg: f32,
    /// Color of the direct light, normalized so its brightest channel is 1.
    pub color: Rgb,
    /// Strength of the direct light: 1 for a clear sun at the zenith, 0 when the sun is down or fully covered.
    pub intensity: f32,
    /// Average color of the sky dome (ambient light), linear.
    pub ambient: Rgb,
}

/// Time-of-day phase, from the sun elevation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DayPhase {
    /// Sun more than 12 degrees below the horizon.
    Night,
    /// Dawn or dusk: sun between 12 and 4 degrees below the horizon.
    Twilight,
    /// Golden hour: sun between 4 degrees below and 6 degrees above the horizon.
    GoldenHour,
    /// Full day.
    Day,
}

/// Phase of the day for a sun elevation in degrees.
pub fn day_phase(elevation_deg: f32) -> DayPhase {
    match elevation_deg {
        e if e < -12.0 => DayPhase::Night,
        e if e < -4.0 => DayPhase::Twilight,
        e if e <= 6.0 => DayPhase::GoldenHour,
        _ => DayPhase::Day,
    }
}

/// How much daylight there is, 0 (night) to 1 (day), smooth through twilight.
pub fn daylight_factor(elevation_deg: f32) -> f32 {
    smoothstep(-8.0, 12.0, elevation_deg)
}

/// Relative air mass along the line of sight to the sun (Kasten-Young).
fn air_mass(elevation_deg: f32) -> f32 {
    let e = elevation_deg.max(-2.0);
    1.0 / (e.to_radians().sin().max(0.0) + 0.505_72 * (e + 6.079_95).powf(-1.6364))
}

/// Per-channel optical depth of one air mass, scaled by turbidity (3 is the reference).
fn optical_depth(turbidity: f32) -> Rgb {
    Rgb::new(0.035, 0.06, 0.11) * (0.55 + 0.15 * turbidity)
}

/// Direct beam strength relative to the sun at the zenith, before cloud cover (0..=1).
pub fn clear_beam(elevation_deg: f32, turbidity: f32) -> f32 {
    let m = air_mass(elevation_deg);
    let tau = optical_depth(turbidity);
    let through = |m: f32| (-(tau.r + tau.g + tau.b) / 3.0 * m).exp();
    let horizon_fade = smoothstep(-1.5, 3.0, elevation_deg);
    (through(m) / through(1.0)).min(1.0) * horizon_fade
}

/// Color of the sun seen through the atmosphere: white high up, orange to red near the horizon.
pub fn sun_color_at(elevation_deg: f32, turbidity: f32) -> Rgb {
    let m = air_mass(elevation_deg);
    let tau = optical_depth(turbidity);
    let color = Rgb::new((-tau.r * m).exp(), (-tau.g * m).exp(), (-tau.b * m).exp());
    color * (1.0 / color.max_channel())
}

/// True while the sun is in the golden-hour band.
pub fn is_golden_hour(elevation_deg: f32) -> bool {
    day_phase(elevation_deg) == DayPhase::GoldenHour
}

/// The direct sun light and the ambient sky light of the scene.
pub fn sun_light(env: &EnvironmentSettings) -> SunLight {
    let model = SkyModel::new(env);
    let position = sun_position(env);
    let clouds = env.sky.clouds;
    let cloud_block = 1.0 - 0.9 * clouds.coverage * clouds.density;
    SunLight {
        direction: model.sun_direction(),
        elevation_deg: position.elevation_deg,
        color: sun_color_at(position.elevation_deg, env.sky.turbidity),
        intensity: clear_beam(position.elevation_deg, env.sky.turbidity) * cloud_block,
        ambient: model.ambient(),
    }
}
