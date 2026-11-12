//! The environmental simulation sliders.

use super::properties_panel::section;
use crate::components::{
    icons,
    slider::{Slider, Thumb},
    tag::Tag,
    text,
};
use crate::config::theme::{colors, typography};
use crate::features::materials::{logic::matches_filter, models::MaterialState};
use crate::models::slider_spec::SliderSpec;
use crate::services::localization::tr;
use eframe::egui::{vec2, Sense, Ui};

const WEATHERING: SliderSpec = SliderSpec {
    min: 0.0,
    max: 2.0,
    unit: "",
    decimals: 2,
};
const FOLIAGE: SliderSpec = SliderSpec {
    min: 0.0,
    max: 3.0,
    unit: "",
    decimals: 2,
};

/// Draws the section, hiding parameters excluded by the filter.
pub fn show(ui: &mut Ui, state: &mut MaterialState) {
    let filter = state.filter.clone();
    section(ui, |ui| {
        ui.spacing_mut().item_spacing.y = 12.0;
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = 4.0;
            text::icon(ui, icons::CLOUD, 14.0, colors::PRIMARY);
            text::label(
                ui,
                text::caps(tr("material.environment")),
                typography::label_md(),
                colors::PRIMARY,
            );
        });
        if matches_filter(tr("material.weathering"), &filter) {
            parameter(
                ui,
                "material.weathering",
                "material.weathering_note",
                &WEATHERING,
                &mut state.weathering,
            );
        }
        if matches_filter(tr("material.weathering"), &filter)
            && matches_filter(tr("material.foliage"), &filter)
        {
            let (line, _) = ui.allocate_exact_size(vec2(ui.available_width(), 1.0), Sense::hover());
            ui.painter().rect_filled(line, 0.0, colors::SURFACE_HIGHEST);
        }
        if matches_filter(tr("material.foliage"), &filter) {
            parameter(
                ui,
                "material.foliage",
                "material.foliage_note",
                &FOLIAGE,
                &mut state.foliage,
            );
        }
    });
}

fn parameter(
    ui: &mut Ui,
    name: &'static str,
    note: &'static str,
    spec: &SliderSpec,
    value: &mut f32,
) {
    ui.vertical(|ui| {
        ui.spacing_mut().item_spacing.y = 4.0;
        crate::components::layout::split_row(
            ui,
            22.0,
            |ui| {
                text::label(
                    ui,
                    tr(name),
                    typography::mono_medium(11.0),
                    colors::ON_SURFACE,
                );
            },
            |ui| {
                Tag::new(spec.format(*value), colors::PRIMARY)
                    .part(" m", colors::OUTLINE)
                    .fill(colors::SURFACE_LOWEST)
                    .padding(6.0, 2.0)
                    .show(ui);
            },
        );
        text::label(
            ui,
            tr(note),
            typography::body_sm(),
            colors::ON_SURFACE_VARIANT,
        );
        Slider::precision()
            .track(6.0, true)
            .thumb(Thumb::Circle(8.0))
            .colors(colors::PRIMARY, colors::SURFACE_LOWEST)
            .show(ui, value);
    });
}
