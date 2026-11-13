//! Editor of the selected light: name, shape, placement, power, color, falloff, shadows, group and IES profile.

use super::fields::{kelvin, number, percent, vector};
use super::light_list::{kind_name, KIND_KEYS};
use crate::features::lighting::logic::{duplicate_light, remove_light};
use crate::features::lighting::models::{Light, LightKind, LightingState, Power};
use crate::features::lighting::services::load_ies;
use crate::services::localization::tr;
use eframe::egui::{self, Ui};
use std::path::Path;

/// Draws the editor of the light at `index`.
pub fn show(ui: &mut Ui, state: &mut LightingState, index: usize) {
    let groups: Vec<(u32, String)> = state
        .setup
        .groups
        .iter()
        .map(|group| (group.id, group.name.clone()))
        .collect();
    let Some(light) = state.setup.lights.get_mut(index) else {
        return;
    };
    ui.horizontal(|ui| {
        ui.label(tr("lighting.name"));
        ui.text_edit_singleline(&mut light.name);
    });
    kind_selector(ui, light);
    shape_fields(ui, &mut light.kind);
    ui.label(tr("lighting.placement"));
    vector(
        ui,
        tr("lighting.position"),
        &mut light.position,
        0.05,
        tr("lighting.unit_m"),
    );
    ui.horizontal(|ui| {
        ui.label(tr("lighting.orientation"));
        let unit = tr("lighting.unit_deg");
        let o = &mut light.orientation;
        for (key, value) in [
            ("lighting.yaw", &mut o.yaw_deg),
            ("lighting.pitch", &mut o.pitch_deg),
            ("lighting.roll", &mut o.roll_deg),
        ] {
            ui.add(
                egui::DragValue::new(value)
                    .range(-360.0..=360.0)
                    .speed(0.5)
                    .prefix(tr(key))
                    .suffix(unit),
            );
        }
    });
    power_fields(ui, light);
    kelvin(ui, tr("lighting.temperature"), &mut light.kelvin);
    percent(ui, tr("lighting.dimmer"), &mut light.dimmer);
    number(
        ui,
        tr("lighting.cutoff"),
        &mut light.cutoff_radius,
        0.0..=1000.0,
        0.1,
        tr("lighting.unit_m"),
    );
    ui.checkbox(&mut light.cast_shadows, tr("lighting.shadows"));
    ui.checkbox(&mut light.enabled, tr("lighting.enabled"));
    group_selector(ui, light, &groups);
    ies_section(ui, light, &mut state.ies_path, &mut state.message);
    ui.horizontal(|ui| {
        if ui.button(tr("lighting.duplicate")).clicked() {
            state.selected = duplicate_light(&mut state.setup, index);
        }
        if ui.button(tr("lighting.delete")).clicked() {
            remove_light(&mut state.setup, index);
            state.selected = None;
        }
    });
}

fn kind_selector(ui: &mut Ui, light: &mut Light) {
    let mut selected = light.kind.index();
    ui.horizontal(|ui| {
        ui.label(tr("lighting.kind"));
        egui::ComboBox::from_id_salt("lighting_kind")
            .selected_text(kind_name(light.kind))
            .show_ui(ui, |ui| {
                for (index, key) in KIND_KEYS.iter().enumerate() {
                    ui.selectable_value(&mut selected, index, tr(key));
                }
            });
    });
    if selected != light.kind.index() {
        light.kind = LightKind::from_index(selected);
    }
}

fn shape_fields(ui: &mut Ui, kind: &mut LightKind) {
    let meters = tr("lighting.unit_m");
    match kind {
        LightKind::Point => {}
        LightKind::Spot {
            inner_deg,
            outer_deg,
        } => {
            let unit = tr("lighting.unit_deg");
            number(ui, tr("lighting.outer"), outer_deg, 1.0..=179.0, 0.5, unit);
            number(
                ui,
                tr("lighting.inner"),
                inner_deg,
                0.0..=*outer_deg,
                0.5,
                unit,
            );
        }
        LightKind::Area {
            width,
            height,
            two_sided,
        } => {
            number(ui, tr("lighting.width"), width, 0.01..=100.0, 0.01, meters);
            number(
                ui,
                tr("lighting.height"),
                height,
                0.01..=100.0,
                0.01,
                meters,
            );
            ui.checkbox(two_sided, tr("lighting.two_sided"));
        }
        LightKind::Linear { length } => {
            number(
                ui,
                tr("lighting.length"),
                length,
                0.01..=100.0,
                0.01,
                meters,
            );
        }
    }
}

fn power_fields(ui: &mut Ui, light: &mut Light) {
    let mut watts = matches!(light.power, Power::Watts { .. });
    ui.horizontal(|ui| {
        ui.label(tr("lighting.power"));
        ui.selectable_value(&mut watts, false, tr("lighting.lumens"));
        ui.selectable_value(&mut watts, true, tr("lighting.watts"));
    });
    let lumens = light.power.lumens();
    match (&mut light.power, watts) {
        (Power::Watts { .. }, false) => light.power = Power::Lumens { value: lumens },
        (Power::Lumens { .. }, true) => {
            let efficacy = crate::features::lighting::models::LED_EFFICACY;
            light.power = Power::Watts {
                value: lumens / efficacy,
                efficacy,
            };
        }
        (Power::Lumens { value }, false) => {
            number(ui, "", value, 0.0..=1.0e7, 10.0, tr("lighting.unit_lm"));
        }
        (Power::Watts { value, efficacy }, true) => {
            number(ui, "", value, 0.0..=1.0e5, 0.5, tr("lighting.unit_w"));
            number(
                ui,
                tr("lighting.efficacy"),
                efficacy,
                1.0..=683.0,
                1.0,
                tr("lighting.unit_lm_per_w"),
            );
        }
    }
}

fn group_selector(ui: &mut Ui, light: &mut Light, groups: &[(u32, String)]) {
    let none = tr("lighting.no_group");
    let current = light
        .group
        .and_then(|id| groups.iter().find(|(gid, _)| *gid == id))
        .map_or(none, |(_, name)| name.as_str());
    ui.horizontal(|ui| {
        ui.label(tr("lighting.group"));
        egui::ComboBox::from_id_salt("lighting_group")
            .selected_text(current)
            .show_ui(ui, |ui| {
                ui.selectable_value(&mut light.group, None, none);
                for (id, name) in groups {
                    ui.selectable_value(&mut light.group, Some(*id), name);
                }
            });
    });
}

fn ies_section(
    ui: &mut Ui,
    light: &mut Light,
    path: &mut String,
    message: &mut Option<&'static str>,
) {
    ui.label(tr("lighting.ies"));
    match &light.ies {
        Some(profile) if !profile.label.is_empty() => ui.label(&profile.label),
        Some(_) => ui.label(tr("lighting.ies_loaded")),
        None => ui.label(tr("lighting.ies_none")),
    };
    ui.horizontal(|ui| {
        ui.add(egui::TextEdit::singleline(path).hint_text(tr("lighting.ies_path")));
        if ui.button(tr("lighting.ies_load")).clicked() {
            match load_ies(Path::new(path.trim())) {
                Ok(profile) => {
                    light.ies = Some(profile);
                    *message = Some("lighting.ies_ok");
                }
                Err(key) => *message = Some(key),
            }
        }
        if light.ies.is_some() && ui.button(tr("lighting.ies_clear")).clicked() {
            light.ies = None;
        }
    });
}
