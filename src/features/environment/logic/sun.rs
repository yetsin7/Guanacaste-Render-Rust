//! Astronomical sun and moon position from date, clock time and location (NOAA solar position equations,
//! accurate to a few hundredths of a degree for dates near the present; no atmospheric refraction).

use super::math::{direction_from_angles, Vec3};
use crate::features::environment::models::{CalendarDate, EnvironmentSettings, Location};
use std::f64::consts::PI;

/// Apparent position of a body in the sky.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SunPosition {
    /// Degrees clockwise from north, 0..360.
    pub azimuth_deg: f32,
    /// Degrees above the horizon; negative below it.
    pub elevation_deg: f32,
}

/// Local clock times of the sun's daily events, in hours.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SunTimes {
    /// Sunrise.
    pub sunrise: f32,
    /// Solar noon (highest point).
    pub solar_noon: f32,
    /// Sunset.
    pub sunset: f32,
}

/// Equation of time (minutes) and solar declination (radians) for a fractional year angle.
fn eqtime_and_declination(gamma: f64) -> (f64, f64) {
    let eqtime = 229.18
        * (0.000075 + 0.001868 * gamma.cos()
            - 0.032077 * gamma.sin()
            - 0.014615 * (2.0 * gamma).cos()
            - 0.040849 * (2.0 * gamma).sin());
    let declination = 0.006918 - 0.399912 * gamma.cos() + 0.070257 * gamma.sin()
        - 0.006758 * (2.0 * gamma).cos()
        + 0.000907 * (2.0 * gamma).sin()
        - 0.002697 * (3.0 * gamma).cos()
        + 0.00148 * (3.0 * gamma).sin();
    (eqtime, declination)
}

fn gamma_at(date: CalendarDate, hour: f64) -> f64 {
    let year_days =
        f64::from(crate::features::environment::models::calendar::days_in_year(date.year));
    2.0 * PI / year_days * (f64::from(date.day_of_year()) - 1.0 + (hour - 12.0) / 24.0)
}

/// Hour angle (radians, positive after solar noon) and declination for a clock time.
fn hour_angle_and_declination(date: CalendarDate, hour: f64, place: Location) -> (f64, f64, f64) {
    let (eqtime, declination) = eqtime_and_declination(gamma_at(date, hour));
    let offset =
        eqtime + 4.0 * f64::from(place.longitude_deg) - 60.0 * f64::from(place.utc_offset_hours);
    let solar_minutes = hour * 60.0 + offset;
    (
        (solar_minutes / 4.0 - 180.0).to_radians(),
        declination,
        eqtime,
    )
}

/// Azimuth and elevation in degrees for hour angle, declination and latitude (all radians).
fn horizontal(hour_angle: f64, declination: f64, latitude: f64) -> SunPosition {
    let cos_zenith =
        latitude.sin() * declination.sin() + latitude.cos() * declination.cos() * hour_angle.cos();
    let elevation = 90.0 - cos_zenith.clamp(-1.0, 1.0).acos().to_degrees();
    let azimuth = hour_angle
        .sin()
        .atan2(hour_angle.cos() * latitude.sin() - declination.tan() * latitude.cos())
        .to_degrees()
        + 180.0;
    SunPosition {
        azimuth_deg: azimuth.rem_euclid(360.0) as f32,
        elevation_deg: elevation as f32,
    }
}

/// Position of the sun for a date, local clock hour (0..24) and location.
pub fn solar_position(date: CalendarDate, hour: f32, place: Location) -> SunPosition {
    let (hour_angle, declination, _) = hour_angle_and_declination(date, f64::from(hour), place);
    horizontal(
        hour_angle,
        declination,
        f64::from(place.latitude_deg).to_radians(),
    )
}

/// Position of the sun for the scene settings.
pub fn sun_position(env: &EnvironmentSettings) -> SunPosition {
    solar_position(env.date, env.hour, env.location)
}

/// Unit vector pointing from the scene toward the sun (`[east, up, north]`); `y` is negative at night.
pub fn sun_direction(env: &EnvironmentSettings) -> Vec3 {
    let position = sun_position(env);
    direction_from_angles(position.azimuth_deg, position.elevation_deg)
}

/// Sunrise, solar noon and sunset of the scene's day; `None` during polar day or polar night.
pub fn sun_times(env: &EnvironmentSettings) -> Option<SunTimes> {
    let place = env.location;
    let (eqtime, declination) = eqtime_and_declination(gamma_at(env.date, 12.0));
    let latitude = f64::from(place.latitude_deg).to_radians();
    let cos_ha = ((-0.833_f64).to_radians().sin() - latitude.sin() * declination.sin())
        / (latitude.cos() * declination.cos());
    if !(-1.0..=1.0).contains(&cos_ha) {
        return None;
    }
    let half_day = cos_ha.acos().to_degrees();
    let clock = |ha_deg: f64| {
        let utc_minutes = 720.0 - 4.0 * (f64::from(place.longitude_deg) + ha_deg) - eqtime;
        ((utc_minutes / 60.0 + f64::from(place.utc_offset_hours)).rem_euclid(24.0)) as f32
    };
    Some(SunTimes {
        sunrise: clock(half_day),
        solar_noon: clock(0.0),
        sunset: clock(-half_day),
    })
}

/// Age of the moon in days since the new moon (0..29.53).
pub fn moon_age_days(env: &EnvironmentSettings) -> f32 {
    const SYNODIC: f64 = 29.530_588_86;
    const NEW_MOON_JD: f64 = 2_451_550.26;
    let utc_hour = f64::from(env.hour) - f64::from(env.location.utc_offset_hours);
    let jd = env.date.julian_day() + utc_hour / 24.0;
    ((jd - NEW_MOON_JD).rem_euclid(SYNODIC)) as f32
}

/// Illuminated fraction of the moon, 0 (new) to 1 (full).
pub fn moon_illumination(env: &EnvironmentSettings) -> f32 {
    let phase = f64::from(moon_age_days(env)) / 29.530_588_86 * 2.0 * PI;
    ((1.0 - phase.cos()) / 2.0) as f32
}

/// Approximate position of the moon: it follows the sun's hour angle lagged by its elongation and moves along
/// the ecliptic (declination within the usual +-23.4 degrees; the 5 degree orbital tilt is ignored).
pub fn moon_position(env: &EnvironmentSettings) -> SunPosition {
    let (hour_angle, _, _) =
        hour_angle_and_declination(env.date, f64::from(env.hour), env.location);
    let elongation = f64::from(moon_age_days(env)) / 29.530_588_86 * 2.0 * PI;
    let sun_longitude = 2.0 * PI * (f64::from(env.date.day_of_year()) - 80.0) / 365.25;
    let moon_longitude = sun_longitude + elongation;
    let declination = (23.44_f64.to_radians().sin() * moon_longitude.sin()).asin();
    horizontal(
        hour_angle - elongation,
        declination,
        f64::from(env.location.latitude_deg).to_radians(),
    )
}

/// Unit vector from the scene toward the moon.
pub fn moon_direction(env: &EnvironmentSettings) -> Vec3 {
    let position = moon_position(env);
    direction_from_angles(position.azimuth_deg, position.elevation_deg)
}
