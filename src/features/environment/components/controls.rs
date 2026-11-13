//! Labeled controls of the environment panel.

use crate::components::{form, layout::split_row, slider::Slider, text};
use crate::config::theme::{colors, typography};
use crate::services::localization::tr;
use eframe::egui::Ui;

/// A caption with its current value on the right and a bar slider below. `value` is normalized 0..=1.
/// Returns true when the user changed it.
pub fn slider_row(ui: &mut Ui, key: &'static str, value_text: String, value: &mut f32) -> bool {
    let mut changed = false;
    ui.vertical(|ui| {
        ui.spacing_mut().item_spacing.y = 6.0;
        split_row(
            ui,
            14.0,
            |ui| {
                text::label(
                    ui,
                    text::caps(tr(key)),
                    typography::label_sm(),
                    colors::ON_SURFACE_VARIANT,
                );
            },
            |ui| {
                text::label(
                    ui,
                    value_text,
                    typography::mono_medium(11.0),
                    colors::ON_SURFACE,
                );
            },
        );
        changed = Slider::bar(8.0).show(ui, value).changed();
    });
    changed
}

/// Like [`slider_row`] for a value in `min..=max`; `format` renders the value text.
pub fn ranged_row(
    ui: &mut Ui,
    key: &'static str,
    value: &mut f32,
    (min, max): (f32, f32),
    format: impl Fn(f32) -> String,
) -> bool {
    let mut normalized = (*value - min) / (max - min);
    let changed = slider_row(ui, key, format(*value), &mut normalized);
    if changed {
        *value = min + normalized.clamp(0.0, 1.0) * (max - min);
    }
    changed
}

/// A caption above a dropdown; returns true when the selection changed.
pub fn dropdown_row(
    ui: &mut Ui,
    key: &'static str,
    id: &str,
    selected: &mut usize,
    options: &[&str],
) -> bool {
    form::caption(ui, tr(key));
    let width = ui.available_width();
    form::dropdown(ui, id, selected, options, width)
}
