//! Sun, sky, clouds and fog of a render, taken from the environment domain and expressed in render units.
//!
//! Render units: a white diffuse surface lit by the reference illuminance has radiance 1. The reference follows
//! the light regime so one exposure works from noon to a night lit by lamps: in daylight it is a third of the
//! clear-sun illuminance, at night a typical interior level.

use super::vec3::Vec3;
use crate::features::environment::logic::sky::SkyModel;
use crate::features::environment::models::{EnvironmentSettings, Rgb};
use crate::features::environment::{fog_color, fog_factor, sun_light};

/// Illuminance of a clear sun at the zenith, in lux.
pub const CLEAR_SUN_LUX: f32 = 100_000.0;
/// Reference illuminance in full daylight (the sun then renders at three units).
const DAY_REFERENCE_LUX: f32 = CLEAR_SUN_LUX / 3.0;
/// Reference illuminance at night.
const NIGHT_REFERENCE_LUX: f32 = 250.0;
/// How much brighter the night sky is rendered than the sky model says, so it stays readable.
const NIGHT_SKY_BOOST: f32 = 18.0;

fn rgb(c: Rgb) -> Vec3 {
    Vec3::new(c.r, c.g, c.b)
}

/// Environment-space `[east, up, north]` to scene space (X east, Y north, Z up).
pub fn to_scene(d: [f32; 3]) -> Vec3 {
    Vec3::new(d[0], d[2], d[1])
}

fn to_environment(d: Vec3) -> [f32; 3] {
    [d.x, d.z, d.y]
}

/// The prepared sky of one render.
pub struct Atmosphere {
    env: EnvironmentSettings,
    model: SkyModel,
    /// Unit direction toward the sun, scene space.
    pub sun_direction: Vec3,
    /// Radiance a white diffuse surface facing the sun gets from it, render units.
    pub sun_light: Vec3,
    /// Illuminance that renders as 1, lux.
    pub reference_lux: f32,
    sky_scale: f32,
    fog_tint: Vec3,
    fogged: bool,
}

impl Atmosphere {
    /// Prepares the sun, sky and fog of `env`.
    pub fn new(env: &EnvironmentSettings) -> Self {
        let sun = sun_light(env);
        let day = (sun.intensity / 0.12).clamp(0.0, 1.0);
        let day = day * day * (3.0 - 2.0 * day);
        let reference_lux =
            (NIGHT_REFERENCE_LUX.ln() * (1.0 - day) + DAY_REFERENCE_LUX.ln() * day).exp();
        let sky_scale = 1.0 + (NIGHT_SKY_BOOST - 1.0) * (1.0 - day);
        let fog_tint = rgb(fog_color(env)) * sky_scale;
        let weather = &env.weather;
        let fogged = weather.fog_density > 0.0 || weather.precipitation_intensity > 0.0;
        Self {
            env: *env,
            model: SkyModel::new(env),
            sun_direction: to_scene(sun.direction).normalized(),
            sun_light: rgb(sun.color) * (sun.intensity * CLEAR_SUN_LUX / reference_lux),
            reference_lux,
            sky_scale,
            fog_tint,
            fogged,
        }
    }

    /// Radiance of the sky seen along the scene-space direction `dir`. The sun disc only appears when `with_disc`
    /// is set (camera and mirror rays); diffuse light from the sun is sampled separately.
    pub fn sky(&self, dir: Vec3, with_disc: bool) -> Vec3 {
        let d = to_environment(dir);
        let color = if with_disc {
            self.model.sample(d)
        } else {
            self.model.sample_without_discs(d)
        };
        rgb(color) * self.sky_scale
    }

    /// Blends `color` with the fog after `distance` meters of air (the weather settings decide how thick it is).
    pub fn fog(&self, color: Vec3, distance: f32) -> Vec3 {
        if !self.fogged && distance < 200.0 {
            // Clear air still hazes the far distance a little; skip the cost where it cannot be seen.
            return color;
        }
        color.lerp(self.fog_tint, fog_factor(&self.env.weather, distance))
    }

    /// Converts a luminance in cd/m² to render units.
    pub fn nits_to_units(&self, nits: f32) -> f32 {
        nits * std::f32::consts::PI / self.reference_lux
    }
}
