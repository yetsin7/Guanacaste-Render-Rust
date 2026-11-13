//! The filter chain editor: presets, the ordered list with switches, parameters and the "add filter" selector.

use super::controls;
use crate::components::{
    button::{Button, ButtonStyle},
    form,
    icon_button::IconButton,
    icons,
    layout::split_row,
    slider::Slider,
    text, toggle,
};
use crate::config::theme::{colors, typography};
use crate::features::camera::models::filter::FilterKind;
use crate::features::camera::models::preset::FilterPreset;
use crate::features::camera::models::state::CameraStudioState;
use crate::services::localization::tr;
use eframe::egui::{vec2, Sense, Ui};
use std::cell::Cell;

/// Draws the whole filter section; returns true when the stack changed.
pub fn show(ui: &mut Ui, state: &mut CameraStudioState) -> bool {
    form::caption(ui, tr("camera.filters"));
    let changed = preset_picker(ui, state);
    let mut edited = false;
    let mut action = None;
    for index in 0..state.stack.len() {
        action = action.or(row(ui, state, index));
    }
    if state.stack.is_empty() {
        text::label(
            ui,
            tr("camera.no_filters"),
            typography::label_sm(),
            colors::OUTLINE,
        );
    }
    if let Some(action) = action {
        edited |= apply(state, action);
    }
    edited |= add_row(ui, state);
    if edited {
        state.mark_custom();
    }
    changed || edited
}

fn preset_picker(ui: &mut Ui, state: &mut CameraStudioState) -> bool {
    let mut names: Vec<&str> = FilterPreset::ALL
        .iter()
        .map(|p| tr(p.label_key()))
        .collect();
    names.push(tr("camera.preset_custom"));
    let mut index = state.preset;
    if form::dropdown(ui, "camera-preset", &mut index, &names, 280.0)
        && index < FilterPreset::ALL.len()
    {
        state.choose_preset(index);
        return true;
    }
    false
}

/// Something the user asked of one row of the list.
#[derive(Clone, Copy)]
enum Action {
    Remove(usize),
    Move(usize, usize),
    Select(usize),
    Edited,
}

/// Applies `action`; returns true when it changed the filters (selecting a row does not).
fn apply(state: &mut CameraStudioState, action: Action) -> bool {
    match action {
        Action::Remove(index) => {
            state.stack.remove(index);
            state.selected = None;
        }
        Action::Move(from, to) => {
            if state.stack.move_entry(from, to) {
                state.selected = Some(to);
            }
        }
        Action::Select(index) => {
            state.selected = if state.selected == Some(index) {
                None
            } else {
                Some(index)
            };
            return false;
        }
        Action::Edited => {}
    }
    true
}

fn row(ui: &mut Ui, state: &mut CameraStudioState, index: usize) -> Option<Action> {
    let action = Cell::new(None);
    let last = state.stack.len() - 1;
    let selected = state.selected == Some(index);
    let entry = &mut state.stack.entries[index];
    let label = tr(entry.filter.kind.label_key());
    ui.push_id(("camera-filter", index), |ui| {
        split_row(
            ui,
            28.0,
            |ui| {
                ui.spacing_mut().item_spacing.x = 6.0;
                if toggle::show_sized(ui, &mut entry.enabled, vec2(28.0, 16.0)).changed() {
                    action.set(Some(Action::Edited));
                }
                let ink = if entry.enabled {
                    colors::ON_SURFACE
                } else {
                    colors::OUTLINE
                };
                let name = text::label(ui, label, typography::body_sm(), ink);
                if ui
                    .interact(name.rect, ui.id().with("name"), Sense::click())
                    .clicked()
                {
                    action.set(Some(Action::Select(index)));
                }
            },
            |ui| {
                ui.spacing_mut().item_spacing.x = 0.0;
                let small = |glyph| IconButton::new(glyph).sizes(14.0, 22.0).fill(None);
                if small(icons::CLOSE)
                    .tooltip(tr("camera.remove"))
                    .show(ui)
                    .clicked()
                {
                    action.set(Some(Action::Remove(index)));
                }
                if index < last
                    && small(icons::EXPAND_MORE)
                        .tooltip(tr("camera.move_down"))
                        .show(ui)
                        .clicked()
                {
                    action.set(Some(Action::Move(index, index + 1)));
                }
                if index > 0
                    && small(icons::EXPAND_LESS)
                        .tooltip(tr("camera.move_up"))
                        .show(ui)
                        .clicked()
                {
                    action.set(Some(Action::Move(index, index - 1)));
                }
            },
        );
        if selected {
            let kind = entry.filter.kind;
            for (param, spec) in kind.specs().iter().enumerate() {
                let value = entry.filter.params[param];
                let mut t = (value - spec.min) / (spec.max - spec.min);
                controls::readout(ui, tr(spec.label_key), controls::trim(value, 2));
                ui.add_space(-6.0);
                if Slider::precision().show(ui, &mut t).changed() {
                    entry
                        .filter
                        .set_param(param, spec.min + (spec.max - spec.min) * t);
                    action.set(Some(Action::Edited));
                }
            }
        }
    });
    action.get()
}

fn add_row(ui: &mut Ui, state: &mut CameraStudioState) -> bool {
    let names: Vec<&str> = FilterKind::ALL.iter().map(|k| tr(k.label_key())).collect();
    let mut added = false;
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 8.0;
        form::dropdown(ui, "camera-add-filter", &mut state.add_kind, &names, 168.0);
        let add = Button::new(tr("camera.add_filter"), ButtonStyle::Container)
            .icon(icons::ADD, 16.0)
            .padding(10.0, 5.0)
            .show(ui);
        if add.clicked() {
            let index = state.stack.push(FilterKind::ALL[state.add_kind]);
            state.selected = Some(index);
            added = true;
        }
    });
    added
}
