//! The lighting panel, in Spanish: light list, light editor, groups, scenes and emissive materials. The shell embeds
//! it with [`panel`].

mod emissive;
mod fields;
mod groups_scenes;
mod light_editor;
mod light_list;

use crate::components::section_title;
use crate::features::lighting::models::LightingState;
use crate::services::localization::tr;
use eframe::egui::{self, Ui};

/// Draws the whole lighting panel editing `state`.
pub fn panel(ui: &mut Ui, state: &mut LightingState) {
    if state
        .selected
        .is_some_and(|index| index >= state.setup.lights.len())
    {
        state.selected = None;
    }
    egui::ScrollArea::vertical()
        .id_salt("lighting_panel")
        .auto_shrink([false, false])
        .show(ui, |ui| {
            let count = state.setup.lights.len().to_string();
            section_title::with_count(ui, tr("lighting.title"), Some(&count));
            ui.add_space(6.0);
            light_list::show(ui, state);
            if let Some(index) = state.selected {
                ui.separator();
                light_editor::show(ui, state, index);
            }
            ui.separator();
            groups_scenes::groups(ui, state);
            ui.separator();
            groups_scenes::scenes(ui, state);
            ui.separator();
            emissive::show(ui, state);
            if let Some(key) = state.message {
                ui.separator();
                ui.label(tr(key));
            }
        });
}
