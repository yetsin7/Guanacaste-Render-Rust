//! Analytic sky: turbidity-dependent gradient, Mie glow, sun disc, moon, stars and clouds.
//!
//! This is a simplified Preetham-style model: the zenith and horizon colors follow the sun elevation through a
//! table of keyframes, turbidity whitens the sky and strengthens the forward-scattering glow around the sun.

use super::clouds::cloud_alpha;
use super::lighting::{clear_beam, daylight_factor, sun_color_at};
use super::math::{dot, lerp, normalize, smoothstep, Vec3};
use super::stars::{moon_disc, star_at};
use super::sun::{moon_direction, moon_illumination, sun_direction, sun_position};
use crate::features::environment::models::{CloudSettings, EnvironmentSettings, Rgb};

/// Angular radius of the sun disc in radians (real value, 0.267 degrees).
const SUN_RADIUS: f32 = 0.004_65;

/// `(sun elevation in degrees, zenith sRGB, horizon sRGB)` keyframes of a clear sky.
const KEYFRAMES: [(f32, [u8; 3], [u8; 3]); 7] = [
    (-18.0, [4, 6, 18], [8, 11, 28]),
    (-6.0, [20, 32, 76], [96, 72, 104]),
    (0.0, [46, 78, 148], [250, 150, 96]),
    (6.0, [58, 110, 196], [242, 190, 150]),
    (20.0, [50, 118, 218], [186, 214, 240]),
    (50.0, [34, 102, 220], [172, 206, 240]),
    (90.0, [30, 98, 218], [170, 204, 240]),
];

fn keyframe_colors(elevation_deg: f32) -> (Rgb, Rgb) {
    let first = KEYFRAMES[0];
    let last = KEYFRAMES[KEYFRAMES.len() - 1];
    let e = elevation_deg.clamp(first.0, last.0);
    for pair in KEYFRAMES.windows(2) {
        let ((e0, z0, h0), (e1, z1, h1)) = (pair[0], pair[1]);
        if e <= e1 {
            let t = smoothstep(e0, e1, e);
            let c = |a: [u8; 3]| Rgb::from_srgb8(a[0], a[1], a[2]);
            return (c(z0).mix(c(z1), t), c(h0).mix(c(h1), t));
        }
    }
    (
        Rgb::from_srgb8(last.1[0], last.1[1], last.1[2]),
        Rgb::from_srgb8(last.2[0], last.2[1], last.2[2]),
    )
}

/// Henyey-Greenstein phase function.
fn henyey_greenstein(cos_angle: f32, g: f32) -> f32 {
    let denominator = (1.0 + g * g - 2.0 * g * cos_angle).max(1e-4).powf(1.5);
    (1.0 - g * g) / (4.0 * std::f32::consts::PI * denominator)
}

/// Everything about the sky that does not depend on the viewing direction, precomputed once so that
/// [`SkyModel::sample`] is cheap enough to call per pixel.
#[derive(Debug, Clone)]
pub struct SkyModel {
    sun: Vec3,
    elevation: f32,
    sun_color: Rgb,
    beam: f32,
    daylight: f32,
    zenith: Rgb,
    horizon: Rgb,
    turbidity: f32,
    clouds: CloudSettings,
    overcast: f32,
    storm: f32,
    drift: [f32; 2],
    stars: bool,
    moon: Option<(Vec3, f32)>,
}

impl SkyModel {
    /// Prepares the sky of `env`.
    pub fn new(env: &EnvironmentSettings) -> Self {
        let elevation = sun_position(env).elevation_deg;
        let turbidity = env.sky.turbidity;
        let (mut zenith, mut horizon) = keyframe_colors(elevation);
        let haze = ((turbidity - 2.0) / 8.0).clamp(0.0, 1.0);
        let day = daylight_factor(elevation);
        let milk = Rgb::splat(horizon.luma().max(zenith.luma()));
        zenith = zenith.mix(milk, 0.35 * haze * day);
        horizon = horizon.mix(milk, 0.25 * haze * day);
        let clouds = env.sky.clouds;
        let overcast = (clouds.coverage * clouds.density).powf(1.5);
        let storm = overcast
            * smoothstep(
                0.0,
                0.8,
                env.weather
                    .precipitation_intensity
                    .max(env.weather.fog_density * 0.5),
            );
        let wind_to = (env.weather.wind_direction_deg + 180.0).to_radians();
        let travel =
            env.sky.time_seconds * (0.01 + 0.04 * clouds.speed + 0.002 * env.weather.wind_speed_ms);
        let moon_dir = moon_direction(env);
        Self {
            sun: sun_direction(env),
            elevation,
            sun_color: sun_color_at(elevation, turbidity),
            beam: clear_beam(elevation, turbidity),
            daylight: day,
            zenith,
            horizon,
            turbidity,
            clouds,
            overcast,
            storm,
            drift: [wind_to.sin() * travel, wind_to.cos() * travel],
            stars: env.sky.stars,
            moon: env
                .sky
                .moon
                .then(|| (moon_dir, 0.12 + 0.88 * moon_illumination(env))),
        }
    }

    /// Unit vector toward the sun.
    pub fn sun_direction(&self) -> Vec3 {
        self.sun
    }

    /// Color of the sky at the horizon away from the sun, with overcast applied (also the default fog color).
    pub fn horizon_color(&self) -> Rgb {
        self.grey_out(self.horizon)
    }

    /// Average color of the sky dome, used as ambient light (never fully black: moonlight and airglow).
    pub fn ambient(&self) -> Rgb {
        let mut sum = Rgb::BLACK;
        let mut count = 0.0;
        for elevation in [25.0_f32, 55.0, 85.0] {
            for azimuth in [0.0_f32, 90.0, 180.0, 270.0] {
                let direction = super::math::direction_from_angles(azimuth, elevation);
                sum = sum + self.sample_without_discs(direction);
                count += 1.0;
            }
        }
        sum * (1.0 / count)
    }

    /// Pulls a clear-sky color toward grey as clouds close in; storms are darker.
    fn grey_out(&self, color: Rgb) -> Rgb {
        let grey = Rgb::splat(color.luma() * lerp(0.95, 0.55, self.storm));
        color.mix(grey, (self.overcast * 0.75).min(0.85))
    }

    fn sky_color(&self, d: Vec3) -> Rgb {
        let up = d[1].max(0.0);
        let mut color = self.horizon.mix(self.zenith, up.powf(0.45));
        let cos_sun = dot(d, self.sun);
        let sun_visible = smoothstep(-9.0, -1.0, self.elevation);
        let beam = self.beam.max(0.0);
        // Forward-scattering glow around the sun, stronger with haze.
        let glow = henyey_greenstein(cos_sun, 0.76) * 0.16 * (self.turbidity / 3.0);
        color = color + self.sun_color * (glow * sun_visible * (0.25 + 0.75 * beam));
        // Warm band along the horizon on the sun's side at dawn and dusk.
        let low_sun = 1.0 - smoothstep(0.0, 22.0, self.elevation);
        let warm = cos_sun.max(0.0).powf(3.0) * low_sun * (1.0 - up.powf(0.5)) * sun_visible;
        color = color + self.sun_color * (warm * 0.32);
        self.grey_out(color)
    }

    fn sample_without_discs(&self, d: Vec3) -> Rgb {
        let d = normalize(d);
        let sky = self.sky_color(d);
        let alpha = cloud_alpha(d, &self.clouds, self.drift);
        sky.mix(self.cloud_color(d), alpha)
    }

    fn cloud_color(&self, d: Vec3) -> Rgb {
        let lit = Rgb::splat(0.8) * self.daylight
            + self.sun_color * (smoothstep(-6.0, 2.0, self.elevation) * self.beam * 0.55);
        let night = Rgb::splat(0.012) + self.horizon * 0.25;
        let shade = 1.0 - 0.6 * self.storm - 0.25 * self.overcast;
        let dark = (1.0 - self.daylight).powi(2);
        let body = (lit * shade).mix(night, dark);
        body.mix(
            self.grey_out(self.horizon),
            1.0 - smoothstep(0.0, 0.3, d[1]),
        )
    }

    /// Color of the sky seen along `direction` (any length), linear RGB. Below the horizon it returns a dark
    /// ground-glow tone so reflections and backgrounds have something sensible.
    pub fn sample(&self, direction: Vec3) -> Rgb {
        let d = normalize(direction);
        if d[1] <= 0.0 {
            let ground = self.grey_out(self.horizon) * 0.32;
            return ground.mix(
                self.sky_color([d[0], 0.0, d[2]]) * 0.5,
                1.0 - smoothstep(-0.25, 0.0, d[1]),
            );
        }
        let mut color = self.sky_color(d);
        let alpha = cloud_alpha(d, &self.clouds, self.drift);
        let clear = 1.0 - alpha;
        // Sun disc and halo.
        let angle = dot(d, self.sun).clamp(-1.0, 1.0).acos();
        let sun_visible = smoothstep(-1.0, 0.5, self.elevation);
        let disc = 1.0 - smoothstep(SUN_RADIUS * 0.9, SUN_RADIUS, angle);
        let halo = (-angle / 0.035).exp() * 0.5;
        color = color
            + self.sun_color * ((disc * 40.0 + halo) * sun_visible * self.beam * clear.powi(3));
        // Moon and stars.
        let night = 1.0 - smoothstep(-16.0, -4.0, self.elevation);
        if let Some((moon, light)) = self.moon {
            let moon_angle = dot(d, moon).clamp(-1.0, 1.0).acos();
            let dimming = 1.0 - 0.9 * self.daylight;
            let glow = (-moon_angle / 0.05).exp() * 0.06 * light;
            let disc = moon_disc(moon_angle) * 0.9 * light;
            color = color + Rgb::new(0.9, 0.92, 1.0) * ((glow + disc) * dimming * clear);
        }
        if self.stars && night > 0.0 {
            if let Some((brightness, tint)) = star_at(d) {
                color = color + tint * (brightness * night * clear.powi(2) * 1.2);
            }
        }
        color.mix(self.cloud_color(d), alpha)
    }
}

/// Color of the sky of `env` seen along `direction` (`[east, up, north]`, any length), linear RGB.
/// For many samples of the same environment build a [`SkyModel`] once and call [`SkyModel::sample`].
pub fn sample_sky(env: &EnvironmentSettings, direction: Vec3) -> Rgb {
    SkyModel::new(env).sample(direction)
}
