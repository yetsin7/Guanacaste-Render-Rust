//! Light groups and lighting scenes (day/night) sections of the panel.

use super::fields::percent;
use crate::features::lighting::logic::{
    add_day_and_night, add_group, apply_scene, capture_scene, remove_group, remove_scene,
};
use crate::features::lighting::models::{LightingState, SceneKind};
use crate::services::localization::{tr, trf};
use eframe::egui::{self, Ui};

/// The groups section: switch, dimmer and delete per group, and a field to add one.
pub fn groups(ui: &mut Ui, state: &mut LightingState) {
    ui.strong(tr("lighting.groups"));
    let mut delete = None;
    for group in &mut state.setup.groups {
        ui.horizontal(|ui| {
            ui.checkbox(&mut group.enabled, &group.name);
            if ui.button(tr("lighting.delete")).clicked() {
                delete = Some(group.id);
            }
        });
        percent(ui, tr("lighting.dimmer"), &mut group.dimmer);
    }
    if let Some(id) = delete {
        remove_group(&mut state.setup, id);
    }
    ui.horizontal(|ui| {
        ui.add(egui::TextEdit::singleline(&mut state.new_name).hint_text(tr("lighting.name_hint")));
        if ui.button(tr("lighting.group_add")).clicked() && !state.new_name.trim().is_empty() {
            let name = std::mem::take(&mut state.new_name);
            add_group(&mut state.setup, name.trim().to_owned());
        }
    });
}

/// The scenes section: apply, capture, delete, and the day/night presets.
pub fn scenes(ui: &mut Ui, state: &mut LightingState) {
    ui.strong(tr("lighting.scenes"));
    let mut apply = None;
    let mut delete = None;
    for (index, scene) in state.setup.scenes.iter().enumerate() {
        ui.horizontal(|ui| {
            let active = state.setup.active_scene == Some(index);
            if ui.selectable_label(active, &scene.name).clicked() {
                apply = Some(index);
            }
            if ui.button(tr("lighting.delete")).clicked() {
                delete = Some(index);
            }
        });
    }
    if let Some(index) = apply {
        apply_scene(&mut state.setup, index);
    }
    if let Some(index) = delete {
        remove_scene(&mut state.setup, index);
    }
    ui.horizontal(|ui| {
        if ui.button(tr("lighting.scene_defaults")).clicked() {
            add_day_and_night(
                &mut state.setup,
                tr("lighting.scene_day").to_owned(),
                tr("lighting.scene_night").to_owned(),
            );
        }
        if ui.button(tr("lighting.scene_capture")).clicked() {
            let number = (state.setup.scenes.len() + 1).to_string();
            let name = if state.new_name.trim().is_empty() {
                trf("lighting.scene_numbered", &[&number])
            } else {
                std::mem::take(&mut state.new_name)
            };
            let scene = capture_scene(&state.setup, name, SceneKind::Custom);
            state.setup.scenes.push(scene);
        }
    });
}
