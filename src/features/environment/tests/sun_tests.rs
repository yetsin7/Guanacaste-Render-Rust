//! Sun and moon position tests.

use super::{approx, at, liberia};
use crate::features::environment::logic::math::{angles_from_direction, dot};
use crate::features::environment::logic::sun::{moon_illumination, solar_position};
use crate::features::environment::models::{CalendarDate, Location};
use crate::features::environment::{day_phase, sun_direction, sun_position, sun_times, DayPhase};

fn from_north(azimuth: f32) -> f32 {
    azimuth.min(360.0 - azimuth)
}

#[test]
fn liberia_june_solstice_solar_noon_is_near_the_zenith_to_the_north() {
    let env = liberia((2026, 6, 21), 12.0);
    let noon = sun_times(&env).expect("sun rises in Liberia").solar_noon;
    // Local solar noon is about 11:41 in Liberia in June (UTC-6 is ahead of the meridian).
    approx(noon, 11.69, 0.1);
    let position = solar_position(env.date, noon, env.location);
    // Declination 23.44 N over latitude 10.63 N: 90 - 12.8 = 77.2 degrees.
    approx(position.elevation_deg, 77.2, 0.3);
    assert!(
        from_north(position.azimuth_deg) < 3.0,
        "sun should be due north, azimuth {}",
        position.azimuth_deg
    );
}

#[test]
fn liberia_at_clock_noon_on_the_solstice_is_high() {
    let position = sun_position(&liberia((2026, 6, 21), 12.0));
    assert!(position.elevation_deg > 76.5, "{}", position.elevation_deg);
}

#[test]
fn liberia_december_noon_is_south_and_lower() {
    let env = liberia((2026, 12, 21), 12.0);
    let noon = sun_times(&env).unwrap().solar_noon;
    let position = solar_position(env.date, noon, env.location);
    approx(position.elevation_deg, 90.0 - (10.63 + 23.44), 0.4);
    approx(position.azimuth_deg, 180.0, 3.0);
}

#[test]
fn sun_is_below_the_horizon_at_night() {
    let env = liberia((2026, 6, 21), 0.0);
    let position = sun_position(&env);
    assert!(position.elevation_deg < -50.0, "{}", position.elevation_deg);
    assert!(sun_direction(&env)[1] < -0.7);
    assert_eq!(day_phase(position.elevation_deg), DayPhase::Night);
}

#[test]
fn liberia_sunrise_and_sunset_in_june() {
    let times = sun_times(&liberia((2026, 6, 21), 12.0)).unwrap();
    approx(times.sunrise, 5.32, 0.15);
    approx(times.sunset, 18.06, 0.15);
    let env = liberia((2026, 6, 21), times.sunrise);
    approx(sun_position(&env).elevation_deg, -0.83, 0.25);
}

#[test]
fn equator_equinox_noon_is_overhead_and_sunrise_near_six() {
    let env = at((2026, 3, 20), 12.0, Location::new(0.0, 0.0, 0.0));
    assert!(sun_position(&env).elevation_deg > 87.0);
    approx(sun_times(&env).unwrap().sunrise, 6.0, 0.15);
}

#[test]
fn polar_day_and_night_have_no_sunrise() {
    let summer = at((2026, 6, 21), 12.0, Location::new(80.0, 0.0, 0.0));
    let winter = at((2026, 12, 21), 12.0, Location::new(80.0, 0.0, 0.0));
    assert!(sun_times(&summer).is_none() && sun_times(&winter).is_none());
    assert!(sun_position(&summer).elevation_deg > 0.0);
    assert!(sun_position(&winter).elevation_deg < 0.0);
}

#[test]
fn southern_hemisphere_summer_sun_is_north_at_noon() {
    let env = at((2026, 12, 21), 12.0, Location::new(-34.604, -58.382, -3.0));
    let noon = sun_times(&env).unwrap().solar_noon;
    let position = solar_position(env.date, noon, env.location);
    assert!(from_north(position.azimuth_deg) < 5.0);
    approx(position.elevation_deg, 78.8, 0.4);
}

#[test]
fn direction_matches_angles_and_is_unit_length() {
    let env = liberia((2026, 2, 15), 16.5);
    let direction = sun_direction(&env);
    approx(dot(direction, direction), 1.0, 1e-5);
    let (azimuth, elevation) = angles_from_direction(direction);
    let position = sun_position(&env);
    approx(azimuth, position.azimuth_deg, 0.01);
    approx(elevation, position.elevation_deg, 0.01);
    // Afternoon in the dry season: sun in the west-southwest.
    assert!(direction[0] < 0.0 && direction[2] < 0.5);
}

#[test]
fn golden_hour_band() {
    assert_eq!(day_phase(3.0), DayPhase::GoldenHour);
    assert_eq!(day_phase(-2.0), DayPhase::GoldenHour);
    assert_eq!(day_phase(-8.0), DayPhase::Twilight);
    assert_eq!(day_phase(30.0), DayPhase::Day);
}

#[test]
fn moon_phase_matches_known_new_and_full_moons() {
    let origin = Location::new(0.0, 0.0, 0.0);
    assert!(moon_illumination(&at((2024, 1, 11), 12.0, origin)) < 0.02);
    assert!(moon_illumination(&at((2024, 1, 25), 18.0, origin)) > 0.98);
}

#[test]
fn julian_day_of_j2000() {
    approx(
        CalendarDate::new(2000, 1, 1).julian_day() as f32,
        2_451_544.5,
        0.01,
    );
}
