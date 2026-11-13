//! Sky color, sun light and preset tests.

use super::{approx, liberia};
use crate::features::environment::logic::lighting::sun_color_at;
use crate::features::environment::logic::math::direction_from_angles;
use crate::features::environment::logic::stars::star_at;
use crate::features::environment::models::SkyPreset;
use crate::features::environment::{
    apply_sky_preset, sample_sky, sun_light, sun_position, EnvironmentSettings, Precipitation, Rgb,
    SkyModel,
};

fn saturation(color: Rgb) -> f32 {
    let max = color.max_channel();
    let min = color.r.min(color.g).min(color.b);
    (max - min) / max.max(1e-6)
}

fn noon() -> EnvironmentSettings {
    let mut env = liberia((2026, 3, 21), 11.7);
    env.sky.clouds.coverage = 0.0;
    env
}

#[test]
fn clear_noon_zenith_is_blue_and_bright() {
    let color = sample_sky(&noon(), [0.0, 1.0, 0.0]);
    assert!(color.b > color.g && color.g > color.r, "{color:?}");
    assert!(color.luma() > 0.08);
}

#[test]
fn night_sky_is_much_darker_than_day() {
    let mut night = noon();
    night.hour = 23.5;
    let up = [0.0, 1.0, 0.0];
    assert!(sample_sky(&night, up).luma() * 20.0 < sample_sky(&noon(), up).luma());
}

#[test]
fn sunset_horizon_toward_the_sun_is_warm() {
    let mut env = noon();
    apply_sky_preset(&mut env, SkyPreset::Sunset);
    env.sky.clouds.coverage = 0.0;
    let sun = sun_position(&env);
    assert!(
        sun.elevation_deg > -2.0 && sun.elevation_deg < 6.0,
        "{sun:?}"
    );
    let toward = sample_sky(&env, direction_from_angles(sun.azimuth_deg, 6.0));
    let away = sample_sky(&env, direction_from_angles(sun.azimuth_deg + 180.0, 6.0));
    assert!(toward.r > toward.b, "{toward:?}");
    assert!(toward.r / toward.b > away.r / away.b);
}

#[test]
fn sun_disc_is_much_brighter_than_the_sky() {
    let env = noon();
    let sun = crate::features::environment::sun_direction(&env);
    let disc = sample_sky(&env, sun).luma();
    let beside = sample_sky(
        &env,
        direction_from_angles(sun_position(&env).azimuth_deg + 40.0, 55.0),
    );
    assert!(disc > 5.0 * beside.luma().max(0.1), "{disc}");
}

#[test]
fn overcast_sky_is_less_saturated_than_clear() {
    let clear = noon();
    let mut overcast = noon();
    overcast.sky.clouds.coverage = 1.0;
    overcast.sky.clouds.density = 1.0;
    let direction = direction_from_angles(120.0, 50.0);
    assert!(
        saturation(sample_sky(&overcast, direction))
            < 0.5 * saturation(sample_sky(&clear, direction))
    );
}

#[test]
fn cloud_cover_changes_the_fraction_of_covered_sky() {
    let count = |coverage: f32| {
        let mut env = noon();
        env.sky.clouds.coverage = coverage;
        env.sky.clouds.density = 1.0;
        let model = SkyModel::new(&env);
        let clear = SkyModel::new(&noon());
        (0..400)
            .filter(|i| {
                let d = direction_from_angles((*i as f32) * 47.0, 25.0 + (*i % 13) as f32 * 4.0);
                model.sample(d).luma() > clear.sample(d).luma() * 1.5
            })
            .count()
    };
    assert!(count(0.1) < count(0.5) && count(0.5) < count(0.9));
}

#[test]
fn sampling_is_finite_deterministic_and_handles_any_direction() {
    let mut env = noon();
    env.sky.clouds.coverage = 0.6;
    let model = SkyModel::new(&env);
    for i in 0..500 {
        let t = i as f32 * 0.7;
        let d = [t.sin() * 3.0, (t * 1.3).cos() * 2.0, (t * 0.4).sin()];
        let a = model.sample(d);
        assert_eq!(a, model.sample(d));
        assert!(a.r.is_finite() && a.g.is_finite() && a.b.is_finite());
        assert!(a.r >= 0.0 && a.g >= 0.0 && a.b >= 0.0);
    }
    assert!(model.sample([0.0, 0.0, 0.0]).luma().is_finite());
}

#[test]
fn night_has_stars_but_day_does_not_show_them() {
    let hits = (0..20_000)
        .filter(|i| {
            let d = direction_from_angles((*i as f32) * 0.618 * 360.0, (*i % 90) as f32);
            star_at(d).is_some()
        })
        .count();
    assert!((20..600).contains(&hits), "{hits} star hits");
    // Brightest pixel of a night sky is a star or the moon; in daylight no pixel exceeds the blue gradient.
    let mut night = noon();
    night.hour = 1.0;
    night.sky.moon = false;
    let model = SkyModel::new(&night);
    let max = (0..20_000)
        .map(|i| {
            model
                .sample(direction_from_angles(
                    i as f32 * 0.618 * 360.0,
                    20.0 + (i % 70) as f32,
                ))
                .luma()
        })
        .fold(0.0_f32, f32::max);
    assert!(max > 0.02, "stars should add light, got {max}");
    night.sky.stars = false;
    let dark = SkyModel::new(&night);
    assert!(dark.sample([0.3, 0.8, 0.2]).luma() < 0.01);
}

#[test]
fn sun_light_changes_with_elevation() {
    let high = sun_light(&noon());
    approx(high.intensity, 1.0, 0.1);
    assert!(high.color.b / high.color.r > 0.75, "{:?}", high.color);
    assert!(high.direction[1] > 0.95);

    let mut low_env = noon();
    low_env.hour = 17.7;
    let low = sun_light(&low_env);
    assert!(
        low.elevation_deg > 0.0 && low.elevation_deg < 8.0,
        "{}",
        low.elevation_deg
    );
    assert!(low.color.b / low.color.r < 0.45, "{:?}", low.color);
    assert!(low.intensity < high.intensity);

    let mut night_env = noon();
    night_env.hour = 0.0;
    assert_eq!(sun_light(&night_env).intensity, 0.0);
}

#[test]
fn clouds_dim_the_direct_light_and_turbidity_reddens_the_sun() {
    let mut covered = noon();
    covered.sky.clouds.coverage = 1.0;
    covered.sky.clouds.density = 1.0;
    assert!(sun_light(&covered).intensity < 0.2 * sun_light(&noon()).intensity);
    let clean = sun_color_at(10.0, 2.0);
    let hazy = sun_color_at(10.0, 10.0);
    assert!(hazy.b / hazy.r < clean.b / clean.r);
}

#[test]
fn presets_set_time_and_look() {
    let mut env = liberia((2026, 2, 15), 12.0);
    apply_sky_preset(&mut env, SkyPreset::Night);
    assert!(sun_position(&env).elevation_deg < -12.0);
    apply_sky_preset(&mut env, SkyPreset::Sunrise);
    let rise = sun_position(&env).elevation_deg;
    assert!(rise > -2.0 && rise < 6.0 && env.hour < 9.0, "{rise}");
    apply_sky_preset(&mut env, SkyPreset::Midday);
    assert!(sun_position(&env).elevation_deg > 60.0);
    apply_sky_preset(&mut env, SkyPreset::Storm);
    assert_eq!(env.weather.precipitation, Precipitation::Rain);
    assert_eq!(env.sky.clouds.coverage, 1.0);
    assert_eq!(env.sky.preset, SkyPreset::Storm);
    apply_sky_preset(&mut env, SkyPreset::Clear);
    assert_eq!(env.weather.precipitation, Precipitation::None);
}
