//! The surface shader selector.

use super::properties_panel::section;
use crate::components::{form, icons, layout::split_row, text};
use crate::config::theme::{colors, typography};
use crate::features::materials::models::MaterialState;
use crate::services::localization::tr;
use eframe::egui::Ui;

const SHADERS: [&str; 4] = [
    "material.shader_glass",
    "material.shader_concrete",
    "material.shader_travertine",
    "material.shader_timber",
];

/// Draws the section.
pub fn show(ui: &mut Ui, state: &mut MaterialState) {
    section(ui, |ui| {
        split_row(
            ui,
            20.0,
            |ui| {
                ui.spacing_mut().item_spacing.x = 4.0;
                text::icon(ui, icons::LIBRARY, 14.0, colors::PRIMARY);
                text::label(
                    ui,
                    text::caps(tr("material.shader")),
                    typography::label_md(),
                    colors::PRIMARY,
                );
            },
            |ui| {
                text::label(
                    ui,
                    text::caps(tr("material.core")),
                    typography::mono(10.0),
                    colors::OUTLINE,
                );
            },
        );
        let labels: Vec<&str> = SHADERS.iter().map(|key| tr(key)).collect();
        let width = ui.available_width() - 8.0;
        form::dropdown(ui, "material-shader", &mut state.shader, &labels, width);
        text::paragraph(
            ui,
            tr("material.shader_note"),
            typography::body_sm(),
            colors::ON_SURFACE_VARIANT,
            18.0,
        );
    });
}
