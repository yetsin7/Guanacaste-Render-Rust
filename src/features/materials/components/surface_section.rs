//! Base color and physical surface parameters, plus emission.

use super::param;
use super::properties_panel::section;
use crate::components::{icons, text};
use crate::config::theme::{colors, typography};
use crate::features::materials::models::MaterialState;
use crate::services::localization::tr;
use eframe::egui::Ui;

fn heading(ui: &mut Ui, glyph: char, key: &'static str) {
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 4.0;
        text::icon(ui, glyph, 14.0, colors::PRIMARY);
        text::label(
            ui,
            text::caps(tr(key)),
            typography::label_md(),
            colors::PRIMARY,
        );
    });
}

/// Draws the surface section.
pub fn show(ui: &mut Ui, state: &mut MaterialState) {
    let Some(material) = state.library.get_mut(state.selected) else {
        return;
    };
    let mut changed = false;
    section(ui, |ui| {
        ui.spacing_mut().item_spacing.y = 8.0;
        heading(ui, icons::LIBRARY, "material.surface");
        let mut rgb = [
            material.base_color[0],
            material.base_color[1],
            material.base_color[2],
        ];
        if param::color(ui, "material.base_color", &mut rgb) {
            material.base_color[..3].copy_from_slice(&rgb);
            changed = true;
        }
        changed |= param::slider(ui, "material.metallic", &mut material.metallic, 0.0, 1.0, 2);
        changed |= param::slider(
            ui,
            "material.roughness",
            &mut material.roughness,
            0.0,
            1.0,
            2,
        );
        changed |= param::slider(
            ui,
            "material.transparency",
            &mut material.transparency,
            0.0,
            1.0,
            2,
        );
        changed |= param::slider(ui, "material.ior", &mut material.ior, 1.0, 2.8, 2);
        changed |= param::slider(
            ui,
            "material.normal_strength",
            &mut material.normal_strength,
            0.0,
            2.0,
            2,
        );
    });
    section(ui, |ui| {
        ui.spacing_mut().item_spacing.y = 8.0;
        heading(ui, icons::LIGHT, "material.emission");
        changed |= param::color(ui, "material.emission_color", &mut material.emission);
        changed |= param::slider(
            ui,
            "material.emission_strength",
            &mut material.emission_strength,
            0.0,
            20.0,
            1,
        );
    });
    if changed {
        state.changed = true;
        state.confirm_delete = false;
    }
}
