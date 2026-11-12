//! The precision viewport controls card: three scrub fields.

use super::render_engine_panel::card;
use crate::components::{
    icons,
    layout::split_row,
    slider::{Slider, Thumb},
    tag::Tag,
    text, text_layout,
};
use crate::config::theme::{colors, typography};
use crate::features::settings::logic::{
    autosave_minutes, matches_search, reflection_k, resolution_percent,
};
use crate::features::settings::models::SettingsState;
use crate::services::localization::{tr, trf};
use eframe::egui::{self, Color32, CornerRadius, Margin, Ui};

struct Scrub {
    title: &'static str,
    subtitle: &'static str,
    chip: String,
    chip_color: Color32,
    ends: (&'static str, &'static str),
    notes: (String, String),
    fill: Color32,
}

/// Draws the card.
pub fn show(ui: &mut Ui, state: &mut SettingsState) {
    card(
        ui,
        icons::SCRUB,
        colors::SECONDARY,
        "settings.precision",
        "settings.precision_caption",
        |ui| {
            ui.spacing_mut().item_spacing.y = 12.0;
            let search = state.search.clone();
            let k = reflection_k(state.reflections);
            let scrubs = [
                Scrub {
                    title: "settings.resolution",
                    subtitle: "settings.resolution_note",
                    chip: format!("{:.0}%", resolution_percent(state.resolution)),
                    chip_color: colors::PRIMARY,
                    ends: ("50%", "100%"),
                    notes: (
                        tr("settings.supersampling").to_owned(),
                        format!(
                            "{:.1}x {}",
                            resolution_percent(state.resolution) / 100.0,
                            tr("settings.dynamic_factor")
                        ),
                    ),
                    fill: colors::PRIMARY_CONTAINER,
                },
                Scrub {
                    title: "settings.reflections",
                    subtitle: "settings.reflections_note",
                    chip: format!("{k}k"),
                    chip_color: colors::SECONDARY,
                    ends: ("1k", "8k"),
                    notes: (
                        trf("settings.pool", &[&(k * 512).to_string()]),
                        trf("settings.glossy", &[&(k * 16).to_string()]),
                    ),
                    fill: colors::SECONDARY,
                },
                Scrub {
                    title: "settings.autosave",
                    subtitle: "settings.autosave_note",
                    chip: format!("{} min", autosave_minutes(state.autosave)),
                    chip_color: colors::TERTIARY,
                    ends: ("5m", "60m"),
                    notes: (
                        tr("settings.path").to_owned(),
                        tr("settings.rollbacks").to_owned(),
                    ),
                    fill: colors::TERTIARY,
                },
            ];
            let mut targets = [state.resolution, state.reflections, state.autosave];
            for (index, scrub) in scrubs.iter().enumerate() {
                if !matches_search(&[tr(scrub.title), tr(scrub.subtitle)], &search) {
                    continue;
                }
                scrub_card(ui, scrub, &mut targets[index]);
            }
            let changed = targets != [state.resolution, state.reflections, state.autosave];
            state.resolution = targets[0];
            state.reflections = targets[1];
            state.autosave = targets[2];
            if changed {
                state.applied = false;
            }
        },
    );
}

fn scrub_card(ui: &mut Ui, scrub: &Scrub, value: &mut f32) {
    egui::Frame::new()
        .fill(colors::SURFACE_CONTAINER)
        .corner_radius(CornerRadius::same(2))
        .inner_margin(Margin::same(12))
        .show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.spacing_mut().item_spacing.y = 4.0;
            split_row(
                ui,
                22.0,
                |ui| {
                    ui.spacing_mut().item_spacing.x = 4.0;
                    text::label(
                        ui,
                        tr(scrub.title),
                        typography::body_md(),
                        colors::ON_SURFACE,
                    );
                    text::label(
                        ui,
                        tr(scrub.subtitle),
                        typography::label_sm(),
                        colors::OUTLINE,
                    );
                },
                |ui| {
                    Tag::new(&scrub.chip, scrub.chip_color)
                        .font(typography::label_md())
                        .fill(colors::SURFACE_LOWEST)
                        .padding(8.0, 2.0)
                        .show(ui);
                },
            );
            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing.x = 12.0;
                text::label(ui, scrub.ends.0, typography::label_sm(), colors::OUTLINE);
                let width = ui.available_width() - 44.0;
                ui.allocate_ui(egui::vec2(width, 14.0), |ui| {
                    Slider::bar(6.0)
                        .thumb(Thumb::None)
                        .colors(scrub.fill, colors::SURFACE_HIGHEST)
                        .show(ui, value);
                });
                text::label(ui, scrub.ends.1, typography::label_sm(), colors::OUTLINE);
            });
            text_layout::space_between_row(
                ui,
                14.0,
                &[
                    (&scrub.notes.0, typography::label_sm(), colors::OUTLINE),
                    (&scrub.notes.1, typography::label_sm(), colors::OUTLINE),
                ],
            );
        });
}
