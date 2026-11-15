//! Panel sections for water and vegetation.

use crate::features::landscape::logic::actions;
use crate::features::landscape::models::{LandscapeState, WaterKind};
use crate::services::localization::{tr, trf};
use eframe::egui::{Slider, Ui};

/// List of water bodies with add/remove and the settings of the selected one.
pub fn water_section(ui: &mut Ui, state: &mut LandscapeState) {
    ui.horizontal(|ui| {
        if ui.button(tr("landscape.water.add_ocean")).clicked() {
            actions::add_water(state, WaterKind::Ocean);
        }
        if ui.button(tr("landscape.water.add_plane")).clicked() {
            actions::add_water(state, WaterKind::Plane);
        }
    });
    let mut remove = None;
    for index in 0..state.landscape.waters.len() {
        let name = trf(
            "landscape.water.item",
            &[
                &(index + 1).to_string(),
                tr(state.landscape.waters[index].kind.label_key()),
            ],
        );
        ui.horizontal(|ui| {
            let selected = state.selected_water == Some(index);
            if ui.selectable_label(selected, name).clicked() {
                state.selected_water = Some(index);
            }
            if ui.small_button(tr("landscape.water.remove")).clicked() {
                remove = Some(index);
            }
        });
    }
    if let Some(index) = remove {
        actions::remove_water(state, index);
    }
    let Some(water) = state
        .selected_water
        .and_then(|i| state.landscape.waters.get_mut(i))
    else {
        return;
    };
    let before = water.clone();
    ui.add(Slider::new(&mut water.height, -50.0..=100.0).text(tr("landscape.water.height")));
    ui.add(Slider::new(&mut water.wave_amplitude, 0.0..=3.0).text(tr("landscape.water.amplitude")));
    ui.add(Slider::new(&mut water.wave_length, 1.0..=100.0).text(tr("landscape.water.length")));
    ui.add(
        Slider::new(&mut water.wave_direction_deg, 0.0..=360.0)
            .text(tr("landscape.water.direction")),
    );
    ui.add(Slider::new(&mut water.opacity, 0.0..=1.0).text(tr("landscape.water.opacity")));
    ui.horizontal(|ui| {
        ui.label(tr("landscape.water.color"));
        ui.color_edit_button_rgb(&mut water.color);
    });
    if water.kind == WaterKind::Plane {
        ui.add(Slider::new(&mut water.size[0], 1.0..=2000.0).text(tr("landscape.water.width")));
        ui.add(Slider::new(&mut water.size[1], 1.0..=2000.0).text(tr("landscape.water.depth")));
    }
    if *water != before {
        state.touch();
    }
}

/// Scatter settings and whole-terrain actions. Returns true when the user asked to plant the garden around the
/// model (the panel does not know the project: the shell serves the request).
pub fn vegetation_section(ui: &mut Ui, state: &mut LandscapeState) -> bool {
    let settings = &mut state.scatter;
    ui.horizontal(|ui| {
        ui.label(tr("landscape.veg.asset"));
        ui.text_edit_singleline(&mut settings.asset);
    });
    ui.add(Slider::new(&mut settings.density, 0.05..=20.0).text(tr("landscape.veg.density")));
    ui.add(Slider::new(&mut settings.scale_min, 0.1..=5.0).text(tr("landscape.veg.scale_min")));
    ui.add(Slider::new(&mut settings.scale_max, 0.1..=5.0).text(tr("landscape.veg.scale_max")));
    ui.add(Slider::new(&mut settings.max_slope_deg, 0.0..=90.0).text(tr("landscape.veg.slope")));
    ui.add(
        Slider::new(&mut settings.min_height, -50.0..=100.0).text(tr("landscape.veg.min_height")),
    );
    ui.label(trf(
        "landscape.veg.count",
        &[&state.landscape.vegetation.len().to_string()],
    ));
    ui.horizontal(|ui| {
        if ui.button(tr("landscape.veg.scatter_all")).clicked() {
            actions::scatter_all(state);
        }
        if ui.button(tr("landscape.veg.clear")).clicked() {
            actions::clear_vegetation(state);
        }
    });
    ui.separator();
    ui.label(tr("landscape.garden.hint"));
    ui.button(tr("landscape.garden.plant")).clicked()
}
