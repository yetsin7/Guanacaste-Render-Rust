//! Unit tests of the environment domain.

mod model_tests;
mod sky_tests;
mod sun_tests;
mod weather_tests;

use super::models::{CalendarDate, EnvironmentSettings, Location};

/// Settings for a date, clock hour and place, with a clear default sky.
fn at(date: (i32, u8, u8), hour: f32, location: Location) -> EnvironmentSettings {
    EnvironmentSettings {
        date: CalendarDate::new(date.0, date.1, date.2),
        hour,
        location,
        ..EnvironmentSettings::default()
    }
}

/// A clear-sky Liberia scene on a date at a clock hour.
fn liberia(date: (i32, u8, u8), hour: f32) -> EnvironmentSettings {
    at(date, hour, Location::GUANACASTE)
}

fn approx(actual: f32, expected: f32, tolerance: f32) {
    assert!(
        (actual - expected).abs() <= tolerance,
        "expected {expected} +- {tolerance}, got {actual}"
    );
}
