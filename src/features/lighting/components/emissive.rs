//! Emissive materials section of the panel.

use super::fields::{kelvin, number};
use crate::features::lighting::models::{EmissiveMaterial, LightingState};
use crate::services::localization::tr;
use eframe::egui::{self, Ui};

/// Lists the emissive materials with their luminance and color temperature.
pub fn show(ui: &mut Ui, state: &mut LightingState) {
    ui.strong(tr("lighting.emissive"));
    let mut delete = None;
    for (index, item) in state.setup.emissive.iter_mut().enumerate() {
        ui.push_id(index, |ui| {
            ui.horizontal(|ui| {
                ui.checkbox(&mut item.enabled, "");
                ui.add(
                    egui::TextEdit::singleline(&mut item.material)
                        .hint_text(tr("lighting.emissive_material")),
                );
                if ui.button(tr("lighting.delete")).clicked() {
                    delete = Some(index);
                }
            });
            number(
                ui,
                tr("lighting.emissive_luminance"),
                &mut item.luminance,
                0.0..=1.0e6,
                5.0,
                tr("lighting.unit_nits"),
            );
            kelvin(ui, tr("lighting.temperature"), &mut item.kelvin);
        });
    }
    if let Some(index) = delete {
        state.setup.emissive.remove(index);
    }
    if ui.button(tr("lighting.emissive_add")).clicked() {
        state.setup.emissive.push(EmissiveMaterial::default());
    }
}
