//! Selection section of the objects panel: numeric transform of one instance, or a summary of several.

use crate::components::{section_title, text};
use crate::config::theme::{colors, typography};
use crate::features::build::models::{ObjectsState, Transform};
use crate::services::localization::{tr, trf};
use eframe::egui::{DragValue, Ui};

fn row3<T: eframe::emath::Numeric>(
    ui: &mut Ui,
    label: &str,
    values: &mut [T; 3],
    speed: f64,
) -> bool {
    let mut changed = false;
    ui.horizontal(|ui| {
        text::label(
            ui,
            label,
            typography::label_sm(),
            colors::ON_SURFACE_VARIANT,
        );
    });
    ui.horizontal(|ui| {
        for (axis, value) in values.iter_mut().enumerate() {
            let name = ["X ", "Y ", "Z "][axis];
            changed |= ui
                .add(DragValue::new(value).speed(speed).prefix(name))
                .changed();
        }
    });
    changed
}

/// Draws the selection section.
pub fn show(ui: &mut Ui, state: &mut ObjectsState) {
    section_title::caption(ui, tr("objects.selection"), colors::OUTLINE);
    let count = state.selection.len();
    if count == 0 {
        text::label(
            ui,
            tr("objects.none_selected"),
            typography::body_sm(),
            colors::OUTLINE,
        );
        return;
    }
    if count > 1 {
        let n = count.to_string();
        text::label(
            ui,
            trf("objects.n_selected", &[&n]),
            typography::body_sm(),
            colors::ON_SURFACE,
        );
        return;
    }
    let Some(object) = state.single() else { return };
    let (id, before) = (object.id, object.transform);
    let mut edited: Transform = before;
    let mut changed = row3(ui, tr("objects.position"), &mut edited.position, 0.05);
    changed |= row3(ui, tr("objects.rotation"), &mut edited.rotation, 1.0);
    changed |= row3(ui, tr("objects.scale"), &mut edited.scale, 0.01);
    for s in &mut edited.scale {
        *s = s.max(0.01);
    }
    if changed && edited != before {
        state.edit_selection(|object, t| {
            if object.id == id {
                *t = edited;
            }
        });
    }
}
