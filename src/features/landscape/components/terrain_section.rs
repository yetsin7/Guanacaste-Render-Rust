//! Panel sections for the terrain: brushes, generation, heightmap import and soil painting.

use super::picker::PickSlot;
use crate::features::landscape::logic::actions;
use crate::features::landscape::models::{LandscapeState, SoilLayer, Tool};
use crate::services::localization::{tr, trf};
use eframe::egui::{self, Slider, Ui};

/// Tool selector, brush parameters and the tool-specific options.
pub fn brush_section(ui: &mut Ui, state: &mut LandscapeState) {
    ui.horizontal_wrapped(|ui| {
        for tool in Tool::ALL {
            ui.selectable_value(&mut state.tool, tool, tr(tool.label_key()));
        }
    });
    let brush = &mut state.brush;
    ui.add(Slider::new(&mut brush.radius, 1.0..=100.0).text(tr("landscape.brush.radius")));
    ui.add(Slider::new(&mut brush.strength, 0.0..=1.0).text(tr("landscape.brush.strength")));
    ui.add(Slider::new(&mut brush.falloff, 0.0..=1.0).text(tr("landscape.brush.falloff")));
    match state.tool {
        Tool::Flatten => {
            ui.add(
                Slider::new(&mut brush.target_height, -100.0..=500.0)
                    .text(tr("landscape.brush.target")),
            );
        }
        Tool::Noise => {
            ui.add(
                Slider::new(&mut brush.noise_scale, 0.5..=100.0).text(tr("landscape.brush.noise")),
            );
        }
        Tool::Paint => {
            ui.horizontal(|ui| {
                for layer in SoilLayer::ALL {
                    ui.selectable_value(&mut state.paint_layer, layer, tr(layer.label_key()));
                }
            });
        }
        _ => {}
    }
}

/// Resolution, procedural generation and heightmap import.
pub fn generation_section(
    ui: &mut Ui,
    state: &mut LandscapeState,
    picker: &PickSlot,
    resolution: &mut usize,
) {
    let terrain = &state.landscape.terrain;
    let [width, depth] = terrain.size();
    let (low, high) = terrain.height_range();
    ui.label(trf(
        "landscape.terrain.summary",
        &[
            &format!("{width:.0}"),
            &format!("{depth:.0}"),
            &format!("{low:.1}"),
            &format!("{high:.1}"),
        ],
    ));
    ui.horizontal(|ui| {
        ui.label(tr("landscape.terrain.resolution"));
        ui.add(egui::DragValue::new(resolution).range(17..=513));
        if ui.button(tr("landscape.terrain.new")).clicked() {
            let spacing = state.landscape.terrain.spacing;
            actions::new_flat(state, *resolution, *resolution, spacing);
        }
    });
    let generator = &mut state.generator;
    ui.add(egui::DragValue::new(&mut generator.seed).prefix(tr("landscape.gen.seed")));
    ui.add(Slider::new(&mut generator.octaves, 1..=10).text(tr("landscape.gen.octaves")));
    ui.add(Slider::new(&mut generator.scale, 10.0..=1000.0).text(tr("landscape.gen.scale")));
    ui.add(Slider::new(&mut generator.amplitude, 0.0..=200.0).text(tr("landscape.gen.amplitude")));
    ui.add(Slider::new(&mut generator.base_height, -50.0..=50.0).text(tr("landscape.gen.base")));
    ui.horizontal(|ui| {
        if ui.button(tr("landscape.gen.generate")).clicked() {
            actions::generate(state);
        }
        if ui.button(tr("landscape.gen.auto_paint")).clicked() {
            actions::paint_automatically(state);
        }
    });
    ui.separator();
    ui.label(tr("landscape.heightmap.title"));
    ui.horizontal(|ui| {
        ui.add(
            egui::DragValue::new(&mut state.heightmap_range[0])
                .prefix(tr("landscape.heightmap.black")),
        );
        ui.add(
            egui::DragValue::new(&mut state.heightmap_range[1])
                .prefix(tr("landscape.heightmap.white")),
        );
    });
    if ui.button(tr("landscape.heightmap.import")).clicked() {
        picker.open(ui.ctx());
    }
    if let Some(path) = picker.take() {
        actions::import_heightmap_file(state, &path);
    }
}
