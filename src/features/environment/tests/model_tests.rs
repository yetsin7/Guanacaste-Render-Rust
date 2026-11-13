//! Serialization, calendar and formatting tests.

use crate::features::environment::logic::format::format_clock;
use crate::features::environment::models::calendar::{days_in_year, is_leap_year};
use crate::features::environment::models::{
    CalendarDate, Location, Precipitation, Rgb, SkyPreset, WeatherPreset, LOCATION_PRESETS,
};
use crate::features::environment::EnvironmentSettings;

#[test]
fn settings_round_trip_through_json() {
    let mut env = EnvironmentSettings {
        hour: 7.25,
        ..EnvironmentSettings::default()
    };
    env.sky.preset = SkyPreset::Storm;
    env.sky.clouds.coverage = 0.83;
    env.weather.precipitation = Precipitation::Snow;
    env.weather.preset = WeatherPreset::Snow;
    env.weather.fog_color = Some(Rgb::new(0.2, 0.3, 0.4));
    env.location = Location::new(-33.869, 151.209, 11.0);
    let json = serde_json::to_string(&env).unwrap();
    let back: EnvironmentSettings = serde_json::from_str(&json).unwrap();
    assert_eq!(env, back);
}

#[test]
fn missing_fields_fall_back_to_defaults() {
    let env: EnvironmentSettings = serde_json::from_str("{}").unwrap();
    assert_eq!(env, EnvironmentSettings::default());
    let partial: EnvironmentSettings =
        serde_json::from_str(r#"{"hour": 9.5, "sky": {"turbidity": 5.0}}"#).unwrap();
    assert_eq!(partial.hour, 9.5);
    assert_eq!(partial.sky.turbidity, 5.0);
    assert_eq!(partial.location, Location::GUANACASTE);
}

#[test]
fn sanitize_clamps_out_of_range_values() {
    let mut env = EnvironmentSettings {
        hour: 25.5,
        ..EnvironmentSettings::default()
    };
    env.sky.turbidity = 99.0;
    env.sky.clouds.coverage = -1.0;
    env.weather.fog_density = 4.0;
    env.weather.wind_direction_deg = -90.0;
    env.location.latitude_deg = 123.0;
    env.date = CalendarDate {
        year: 2026,
        month: 2,
        day: 40,
    };
    env.sanitize();
    assert_eq!(env.hour, 1.5);
    assert_eq!(env.sky.turbidity, 10.0);
    assert_eq!(env.sky.clouds.coverage, 0.0);
    assert_eq!(env.weather.fog_density, 1.0);
    assert_eq!(env.weather.wind_direction_deg, 270.0);
    assert_eq!(env.location.latitude_deg, 90.0);
    assert_eq!(env.date.day, 28);
}

#[test]
fn calendar_day_of_year_round_trips() {
    assert!(is_leap_year(2024) && !is_leap_year(2026) && !is_leap_year(1900));
    assert_eq!(days_in_year(2024), 366);
    assert_eq!(CalendarDate::new(2026, 12, 31).day_of_year(), 365);
    assert_eq!(CalendarDate::new(2024, 3, 1).day_of_year(), 61);
    for year in [2026, 2024] {
        for day in 1..=days_in_year(year) {
            let date = CalendarDate::from_day_of_year(year, day);
            assert_eq!(date.day_of_year(), day, "{date:?}");
        }
    }
}

#[test]
fn clock_formatting_wraps_and_rounds() {
    assert_eq!(format_clock(0.0), "00:00");
    assert_eq!(format_clock(16.5), "16:30");
    assert_eq!(format_clock(23.999), "00:00");
}

#[test]
fn rgb_srgb_round_trip() {
    let color = Rgb::from_srgb8(120, 200, 30);
    assert_eq!(color.to_srgb8(), [120, 200, 30]);
    assert_eq!(Rgb::splat(5.0).to_srgb8(), [255, 255, 255]);
}

#[test]
fn location_presets_start_with_guanacaste() {
    assert_eq!(LOCATION_PRESETS[0].location, Location::GUANACASTE);
    assert!(LOCATION_PRESETS
        .iter()
        .all(|p| p.location.latitude_deg.abs() <= 90.0));
}
