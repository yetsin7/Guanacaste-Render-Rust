//! The surface shader selector.

use super::properties_panel::section;
use crate::components::{icons, layout::split_row, text};
use crate::config::theme::{colors, typography};
use crate::features::materials::models::MaterialState;
use crate::services::localization::tr;
use eframe::egui::{self, Ui};

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
        egui::ComboBox::from_id_salt("material-shader")
            .selected_text(
                egui::RichText::new(tr(SHADERS[state.shader])).font(typography::body_md()),
            )
            .width(ui.available_width() - 8.0)
            .show_ui(ui, |ui| {
                for (index, key) in SHADERS.into_iter().enumerate() {
                    ui.selectable_value(
                        &mut state.shader,
                        index,
                        egui::RichText::new(tr(key)).font(typography::body_md()),
                    );
                }
            });
        text::paragraph(
            ui,
            tr("material.shader_note"),
            typography::body_sm(),
            colors::ON_SURFACE_VARIANT,
            18.0,
        );
    });
}
