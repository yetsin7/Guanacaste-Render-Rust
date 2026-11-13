//! UV scale, rotation, offset and tiling mode.

use super::param;
use super::properties_panel::section;
use crate::components::{form, icons, text};
use crate::config::theme::{colors, typography};
use crate::features::materials::models::{MaterialState, Wrap};
use crate::services::localization::tr;
use eframe::egui::Ui;

/// Draws the section.
pub fn show(ui: &mut Ui, state: &mut MaterialState) {
    let Some(material) = state.library.get_mut(state.selected) else {
        return;
    };
    let mut changed = false;
    section(ui, |ui| {
        ui.spacing_mut().item_spacing.y = 8.0;
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = 4.0;
            text::icon(ui, icons::SCALE, 14.0, colors::PRIMARY);
            text::label(
                ui,
                text::caps(tr("material.uv")),
                typography::label_md(),
                colors::PRIMARY,
            );
        });
        let uv = &mut material.uv;
        changed |= param::slider(ui, "material.uv_scale_u", &mut uv.scale[0], 0.1, 20.0, 2);
        changed |= param::slider(ui, "material.uv_scale_v", &mut uv.scale[1], 0.1, 20.0, 2);
        changed |= param::slider(
            ui,
            "material.uv_rotation",
            &mut uv.rotation,
            -180.0,
            180.0,
            0,
        );
        changed |= param::slider(ui, "material.uv_offset_u", &mut uv.offset[0], -1.0, 1.0, 2);
        changed |= param::slider(ui, "material.uv_offset_v", &mut uv.offset[1], -1.0, 1.0, 2);
        form::caption(ui, tr("material.uv_wrap"));
        let labels: Vec<&str> = Wrap::ALL.iter().map(|w| tr(w.label_key())).collect();
        let mut index = Wrap::ALL.iter().position(|w| *w == uv.wrap).unwrap_or(0);
        let width = ui.available_width() - 8.0;
        if form::dropdown(ui, "material-wrap", &mut index, &labels, width) {
            uv.wrap = Wrap::ALL[index];
            changed = true;
        }
    });
    if changed {
        state.changed = true;
    }
}
