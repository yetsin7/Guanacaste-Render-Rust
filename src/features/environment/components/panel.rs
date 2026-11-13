//! The sky, sun and weather panel of the build dock.

use super::{controls, sky_preview};
use crate::components::{segmented::Segmented, text};
use crate::config::theme::{colors, typography};
use crate::features::environment::logic::format::format_clock;
use crate::features::environment::logic::lighting::{day_phase, DayPhase};
use crate::features::environment::logic::presets::{apply_sky_preset, apply_weather_preset};
use crate::features::environment::logic::sun::{sun_position, sun_times};
use crate::features::environment::models::{
    calendar::days_in_year, CalendarDate, EnvironmentSettings, Precipitation, SkyPreset,
    WeatherPreset, LOCATION_PRESETS,
};
use crate::services::localization::{tr, trf};
use eframe::egui::Ui;

const MONTHS: [&str; 12] = [
    "env.month_1",
    "env.month_2",
    "env.month_3",
    "env.month_4",
    "env.month_5",
    "env.month_6",
    "env.month_7",
    "env.month_8",
    "env.month_9",
    "env.month_10",
    "env.month_11",
    "env.month_12",
];

fn phase_key(phase: DayPhase) -> &'static str {
    match phase {
        DayPhase::Night => "env.phase_night",
        DayPhase::Twilight => "env.phase_twilight",
        DayPhase::GoldenHour => "env.phase_golden",
        DayPhase::Day => "env.phase_day",
    }
}

/// Draws the environment controls over `env`; returns true when anything changed.
pub fn show(ui: &mut Ui, env: &mut EnvironmentSettings) -> bool {
    let mut changed = false;
    ui.columns(3, |columns| {
        changed |= sky_column(&mut columns[0], env);
        changed |= time_column(&mut columns[1], env);
        changed |= weather_column(&mut columns[2], env);
    });
    changed
}

fn sky_column(ui: &mut Ui, env: &mut EnvironmentSettings) -> bool {
    let mut changed = false;
    ui.spacing_mut().item_spacing.y = 8.0;
    sky_preview::show(ui, env, 96.0);
    let names: Vec<&str> = SkyPreset::ALL.iter().map(|p| tr(p.key())).collect();
    let mut selected = env.sky.preset.index();
    if controls::dropdown_row(
        ui,
        "env.sky_type",
        "environment-sky-preset",
        &mut selected,
        &names,
    ) {
        apply_sky_preset(env, SkyPreset::ALL[selected]);
        changed = true;
    }
    let position = sun_position(env);
    let phase = day_phase(position.elevation_deg);
    text::label(
        ui,
        trf(
            "env.sun_readout",
            &[
                &format!("{:.1}", position.elevation_deg),
                &format!("{:.0}", position.azimuth_deg),
            ],
        ),
        typography::mono(10.0),
        colors::ON_SURFACE_VARIANT,
    );
    text::label(
        ui,
        tr(phase_key(phase)),
        typography::label_sm(),
        colors::TERTIARY,
    );
    text::label(
        ui,
        sun_summary(env),
        typography::mono(10.0),
        colors::OUTLINE,
    );
    changed
}

fn time_column(ui: &mut Ui, env: &mut EnvironmentSettings) -> bool {
    let mut changed = false;
    ui.spacing_mut().item_spacing.y = 8.0;
    changed |= controls::ranged_row(ui, "env.hour", &mut env.hour, (0.0, 23.99), format_clock);
    let mut day = f32::from(env.date.day_of_year());
    let last = f32::from(days_in_year(env.date.year));
    if controls::ranged_row(ui, "env.date", &mut day, (1.0, last), |d| {
        date_text(env.date.year, d)
    }) {
        env.date = CalendarDate::from_day_of_year(env.date.year, day.round() as u16);
        changed = true;
    }
    changed |= location_row(ui, env);
    let (low, high) = crate::features::environment::models::SkySettings::TURBIDITY_RANGE;
    changed |= controls::ranged_row(
        ui,
        "env.turbidity",
        &mut env.sky.turbidity,
        (low, high),
        |v| format!("{v:.1}"),
    );
    let clouds = &mut env.sky.clouds;
    changed |= controls::ranged_row(
        ui,
        "env.cloud_cover",
        &mut clouds.coverage,
        (0.0, 1.0),
        percent,
    );
    changed |= controls::ranged_row(
        ui,
        "env.cloud_density",
        &mut clouds.density,
        (0.0, 1.0),
        percent,
    );
    changed |= controls::ranged_row(
        ui,
        "env.cloud_speed",
        &mut clouds.speed,
        (0.0, 1.0),
        percent,
    );
    changed
}

fn weather_column(ui: &mut Ui, env: &mut EnvironmentSettings) -> bool {
    let mut changed = false;
    ui.spacing_mut().item_spacing.y = 8.0;
    let names: Vec<&str> = WeatherPreset::ALL.iter().map(|p| tr(p.key())).collect();
    let mut selected = env.weather.preset.index();
    if controls::dropdown_row(
        ui,
        "env.weather",
        "environment-weather-preset",
        &mut selected,
        &names,
    ) {
        apply_weather_preset(env, WeatherPreset::ALL[selected]);
        changed = true;
    }
    let options: Vec<&str> = Precipitation::ALL.iter().map(|p| tr(p.key())).collect();
    let mut precipitation = env.weather.precipitation.index();
    if Segmented::text(&options).show(ui, &mut precipitation) {
        env.weather.precipitation = Precipitation::ALL[precipitation];
        if env.weather.precipitation != Precipitation::None
            && env.weather.precipitation_intensity < 0.05
        {
            env.weather.precipitation_intensity = 0.5;
        }
        changed = true;
    }
    let weather = &mut env.weather;
    changed |= controls::ranged_row(
        ui,
        "env.precip_intensity",
        &mut weather.precipitation_intensity,
        (0.0, 1.0),
        percent,
    );
    changed |= controls::ranged_row(ui, "env.fog", &mut weather.fog_density, (0.0, 1.0), percent);
    changed |= controls::ranged_row(
        ui,
        "env.wind_speed",
        &mut weather.wind_speed_ms,
        (0.0, 30.0),
        |v| trf("env.unit_ms", &[&format!("{v:.0}")]),
    );
    changed |= controls::ranged_row(
        ui,
        "env.wind_direction",
        &mut weather.wind_direction_deg,
        (0.0, 359.0),
        |v| format!("{v:.0}\u{b0}"),
    );
    changed |= controls::ranged_row(
        ui,
        "env.humidity",
        &mut weather.humidity,
        (0.0, 1.0),
        percent,
    );
    changed
}

fn location_row(ui: &mut Ui, env: &mut EnvironmentSettings) -> bool {
    let current = LOCATION_PRESETS
        .iter()
        .position(|preset| preset.location == env.location);
    let mut names: Vec<&str> = LOCATION_PRESETS.iter().map(|p| tr(p.key)).collect();
    if current.is_none() {
        names.push(tr("env.loc_custom"));
    }
    let mut selected = current.unwrap_or(LOCATION_PRESETS.len());
    let changed = controls::dropdown_row(
        ui,
        "env.location",
        "environment-location",
        &mut selected,
        &names,
    );
    if changed && selected < LOCATION_PRESETS.len() {
        env.location = LOCATION_PRESETS[selected].location;
    }
    changed
}

fn sun_summary(env: &EnvironmentSettings) -> String {
    match sun_times(env) {
        Some(times) => trf(
            "env.sun_times",
            &[&format_clock(times.sunrise), &format_clock(times.sunset)],
        ),
        None => tr("env.sun_polar").to_owned(),
    }
}

fn percent(value: f32) -> String {
    format!("{:.0}%", value * 100.0)
}

fn date_text(year: i32, day_of_year: f32) -> String {
    let date = CalendarDate::from_day_of_year(year, day_of_year.round() as u16);
    trf(
        "env.date_format",
        &[
            &date.day.to_string(),
            tr(MONTHS[usize::from(date.month) - 1]),
        ],
    )
}
