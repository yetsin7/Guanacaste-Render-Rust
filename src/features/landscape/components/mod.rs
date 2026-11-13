//! Landscape interface: the panel embedded by the shell and its sections.

pub mod picker;
pub mod terrain_section;
pub mod water_section;

use crate::features::landscape::models::{LandscapeState, Status};
use crate::services::localization::tr;
use eframe::egui::{self, Color32, Ui};
use picker::PickSlot;

/// Interface memory of the panel that is not part of the saved landscape.
#[derive(Debug, Clone)]
struct PanelMemory {
    picker: PickSlot,
    resolution: usize,
}

/// The landscape panel: terrain tools, generation, soil, water and vegetation. Edits `state` in place and bumps
/// `state.revision` whenever geometry changes; the 3D view calls `logic::actions::apply_stroke` for brush strokes.
pub fn panel(ui: &mut Ui, state: &mut LandscapeState) {
    let id = ui.id().with("landscape_panel");
    let mut memory = ui.data_mut(|data| {
        data.get_temp::<PanelMemory>(id).unwrap_or(PanelMemory {
            picker: PickSlot::default(),
            resolution: state.landscape.terrain.columns,
        })
    });
    ui.heading(tr("landscape.title"));
    egui::CollapsingHeader::new(tr("landscape.section.brush"))
        .default_open(true)
        .show(ui, |ui| terrain_section::brush_section(ui, state));
    egui::CollapsingHeader::new(tr("landscape.section.terrain"))
        .default_open(true)
        .show(ui, |ui| {
            terrain_section::generation_section(ui, state, &memory.picker, &mut memory.resolution)
        });
    egui::CollapsingHeader::new(tr("landscape.section.water"))
        .default_open(true)
        .show(ui, |ui| water_section::water_section(ui, state));
    egui::CollapsingHeader::new(tr("landscape.section.vegetation"))
        .default_open(true)
        .show(ui, |ui| water_section::vegetation_section(ui, state));
    match &state.status {
        Some(Status::Info(text)) => {
            ui.label(text);
        }
        Some(Status::Error(text)) => {
            ui.colored_label(Color32::LIGHT_RED, text);
        }
        None => {}
    }
    ui.data_mut(|data| data.insert_temp(id, memory));
}
