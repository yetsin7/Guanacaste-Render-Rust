//! Buttons that add each kind of light and the selectable list of placed lights.

use crate::features::lighting::logic::add_light;
use crate::features::lighting::models::{LightKind, LightingState};
use crate::services::localization::{tr, trf};
use eframe::egui::Ui;

/// Localization keys of the light kinds, in selector order.
pub const KIND_KEYS: [&str; 4] = [
    "lighting.kind_point",
    "lighting.kind_spot",
    "lighting.kind_area",
    "lighting.kind_linear",
];

/// Display name of a light kind.
pub fn kind_name(kind: LightKind) -> &'static str {
    tr(KIND_KEYS[kind.index()])
}

/// The add buttons and the list of lights.
pub fn show(ui: &mut Ui, state: &mut LightingState) {
    ui.label(tr("lighting.add"));
    ui.horizontal_wrapped(|ui| {
        for (index, key) in KIND_KEYS.iter().enumerate() {
            if ui.button(tr(key)).clicked() {
                let kind = LightKind::from_index(index);
                let number = (state.setup.lights.len() + 1).to_string();
                let name = trf("lighting.new_light", &[tr(key), &number]);
                state.selected = Some(add_light(&mut state.setup, kind, name));
                state.message = None;
            }
        }
    });
    ui.add_space(4.0);
    if state.setup.lights.is_empty() {
        ui.label(tr("lighting.empty"));
        return;
    }
    let mut clicked = None;
    for (index, light) in state.setup.lights.iter_mut().enumerate() {
        ui.horizontal(|ui| {
            ui.checkbox(&mut light.enabled, "")
                .on_hover_text(tr("lighting.enabled"));
            let text = trf("lighting.list_item", &[&light.name, kind_name(light.kind)]);
            if ui
                .selectable_label(state.selected == Some(index), text)
                .clicked()
            {
                clicked = Some(index);
            }
        });
    }
    if let Some(index) = clicked {
        state.selected = (state.selected != Some(index)).then_some(index);
    }
}
