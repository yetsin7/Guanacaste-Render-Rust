//! Fog, visibility and weather preset tests.

use super::approx;
use crate::features::environment::logic::fog::extinction;
use crate::features::environment::models::{Precipitation, WeatherPreset, WeatherSettings};
use crate::features::environment::{
    apply_fog, apply_weather_preset, fog_color, fog_factor, visibility_m, EnvironmentSettings, Rgb,
};

fn foggy(density: f32) -> WeatherSettings {
    WeatherSettings {
        fog_density: density,
        ..WeatherSettings::default()
    }
}

#[test]
fn fog_factor_is_zero_at_the_camera_and_grows_with_distance() {
    let weather = foggy(0.5);
    assert_eq!(fog_factor(&weather, 0.0), 0.0);
    let mut last = 0.0;
    for distance in [10.0, 50.0, 200.0, 1000.0, 20_000.0] {
        let factor = fog_factor(&weather, distance);
        assert!(factor > last && factor <= 1.0, "{distance}: {factor}");
        last = factor;
    }
    assert!(last > 0.99);
}

#[test]
fn fog_factor_grows_with_density() {
    let weak = fog_factor(&foggy(0.2), 300.0);
    let strong = fog_factor(&foggy(0.8), 300.0);
    assert!(strong > weak);
}

#[test]
fn dense_fog_limits_visibility_to_about_sixty_five_meters() {
    approx(visibility_m(&foggy(1.0)), 65.0, 3.0);
    assert!(visibility_m(&foggy(0.0)) > 20_000.0);
    // At the visibility distance the scene keeps 2 % of its contrast.
    let weather = foggy(0.6);
    approx(fog_factor(&weather, visibility_m(&weather)), 0.98, 0.001);
}

#[test]
fn rain_snow_and_humidity_add_extinction() {
    let dry = extinction(&WeatherSettings::default());
    let rain = WeatherSettings {
        precipitation: Precipitation::Rain,
        precipitation_intensity: 1.0,
        ..WeatherSettings::default()
    };
    let snow = WeatherSettings {
        precipitation: Precipitation::Snow,
        ..rain
    };
    let humid = WeatherSettings {
        humidity: 1.0,
        ..WeatherSettings::default()
    };
    assert!(extinction(&rain) > 10.0 * dry);
    assert!(extinction(&snow) > extinction(&rain));
    assert!(
        extinction(&humid)
            > extinction(&WeatherSettings {
                humidity: 0.0,
                ..humid
            })
    );
}

#[test]
fn fog_color_is_overridable_and_apply_fog_blends() {
    let mut env = EnvironmentSettings::default();
    env.weather.fog_density = 0.8;
    let automatic = fog_color(&env);
    assert!(automatic.luma() > 0.0);
    env.weather.fog_color = Some(Rgb::new(1.0, 0.0, 0.0));
    assert_eq!(fog_color(&env), Rgb::new(1.0, 0.0, 0.0));
    let surface = Rgb::splat(0.5);
    assert_eq!(apply_fog(surface, &env, 0.0), surface);
    let far = apply_fog(surface, &env, 100_000.0);
    approx(far.r, 1.0, 0.01);
    approx(far.g, 0.0, 0.01);
}

#[test]
fn weather_presets_set_the_expected_conditions() {
    let mut env = EnvironmentSettings::default();
    apply_weather_preset(&mut env, WeatherPreset::Rain);
    assert_eq!(env.weather.precipitation, Precipitation::Rain);
    assert!(env.weather.precipitation_intensity > 0.3 && env.sky.clouds.coverage > 0.9);
    apply_weather_preset(&mut env, WeatherPreset::Snow);
    assert_eq!(env.weather.precipitation, Precipitation::Snow);
    apply_weather_preset(&mut env, WeatherPreset::Fog);
    assert!(visibility_m(&env.weather) < 500.0);
    assert_eq!(env.weather.precipitation, Precipitation::None);
    apply_weather_preset(&mut env, WeatherPreset::Mist);
    assert!(visibility_m(&env.weather) > 1000.0);
    apply_weather_preset(&mut env, WeatherPreset::Windy);
    assert!(env.weather.wind_speed_ms > 10.0);
    apply_weather_preset(&mut env, WeatherPreset::Clear);
    assert!(visibility_m(&env.weather) > 10_000.0);
    assert_eq!(env.weather.preset, WeatherPreset::Clear);
}
