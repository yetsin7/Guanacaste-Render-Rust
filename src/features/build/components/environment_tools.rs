//! The Landscape and Weather panels of the build dock.

use crate::components::{
    choice_row::{self, Choice},
    icons,
    layout::split_row,
    slider::Slider,
    text,
};
use crate::config::theme::{colors, typography};
use crate::features::build::logic::format_hour;
use crate::features::build::models::BuildState;
use crate::services::localization::tr;
use eframe::egui::Ui;

const TERRAIN_TOOLS: [(char, &str); 5] = [
    (icons::TERRAIN, "build.terrain_raise"),
    (icons::BLUR, "build.terrain_smooth"),
    (icons::SWAP_HORIZ, "build.terrain_flatten"),
    (icons::PALETTE, "build.terrain_paint"),
    (icons::CLOUD, "build.terrain_water"),
];
const WEATHER_PRESETS: [(char, &str); 5] = [
    (icons::SUN, "build.weather_clear"),
    (icons::CLOUD, "build.weather_cloudy"),
    (icons::BLUR, "build.weather_fog"),
    (icons::WIND, "build.weather_wind"),
    (icons::SNOW, "build.weather_snow"),
];

fn slider_row(ui: &mut Ui, key: &'static str, value_text: String, value: &mut f32) {
    ui.vertical(|ui| {
        ui.spacing_mut().item_spacing.y = 6.0;
        split_row(
            ui,
            14.0,
            |ui| {
                text::label(
                    ui,
                    text::caps(tr(key)),
                    typography::label_sm(),
                    colors::ON_SURFACE_VARIANT,
                );
            },
            |ui| {
                text::label(
                    ui,
                    value_text,
                    typography::mono_medium(11.0),
                    colors::ON_SURFACE,
                );
            },
        );
        Slider::bar(8.0).show(ui, value);
    });
}

/// The terrain sculpting panel.
pub fn landscape(ui: &mut Ui, state: &mut BuildState) {
    let choices: Vec<_> = TERRAIN_TOOLS
        .iter()
        .map(|(glyph, key)| Choice {
            icon: Some(*glyph),
            label: tr(key),
        })
        .collect();
    choice_row::show(
        ui,
        &choices,
        &mut state.terrain_tool,
        &choice_row::tool_style(typography::label_md()),
        4.0,
    );
    slider_row(
        ui,
        "build.brush_size",
        format!("{:.0} m", 1.0 + state.brush * 49.0),
        &mut state.brush,
    );
    slider_row(
        ui,
        "build.brush_strength",
        format!("{:.0}%", state.strength * 100.0),
        &mut state.strength,
    );
}

/// The weather and time-of-day panel.
pub fn weather(ui: &mut Ui, state: &mut BuildState) {
    let choices: Vec<_> = WEATHER_PRESETS
        .iter()
        .map(|(glyph, key)| Choice {
            icon: Some(*glyph),
            label: tr(key),
        })
        .collect();
    choice_row::show(
        ui,
        &choices,
        &mut state.weather,
        &choice_row::pill_style(typography::label_sm()),
        4.0,
    );
    slider_row(ui, "build.hour", format_hour(state.hour), &mut state.hour);
    slider_row(
        ui,
        "build.clouds",
        format!("{:.0}%", state.clouds * 100.0),
        &mut state.clouds,
    );
    slider_row(
        ui,
        "build.fog",
        format!("{:.0}%", state.fog * 100.0),
        &mut state.fog,
    );
}
