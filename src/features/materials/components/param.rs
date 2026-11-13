//! Parameter rows bound to material values: a labelled slider and a color swatch.

use crate::components::{
    layout::split_row,
    slider::{Slider, Thumb},
    text,
};
use crate::config::theme::{colors, typography};
use crate::services::localization::tr;
use eframe::egui::{Color32, Ui};

/// A label with its value and a slider over `min..=max`; returns true when the value changed.
pub fn slider(
    ui: &mut Ui,
    label: &'static str,
    value: &mut f32,
    min: f32,
    max: f32,
    decimals: usize,
) -> bool {
    ui.spacing_mut().item_spacing.y = 2.0;
    split_row(
        ui,
        18.0,
        |ui| {
            text::label(
                ui,
                tr(label),
                typography::mono_medium(11.0),
                colors::ON_SURFACE,
            );
        },
        |ui| {
            text::label(
                ui,
                format!("{:.*}", decimals, *value),
                typography::mono(11.0),
                colors::PRIMARY,
            );
        },
    );
    let span = (max - min).max(f32::EPSILON);
    let mut t = ((*value - min) / span).clamp(0.0, 1.0);
    let response = Slider::precision()
        .track(6.0, true)
        .thumb(Thumb::Circle(8.0))
        .colors(colors::PRIMARY, colors::SURFACE_LOWEST)
        .show(ui, &mut t);
    if response.changed() {
        *value = min + t * span;
        true
    } else {
        false
    }
}

/// A label with a color button editing an sRGB triple; returns true when the color changed. The value is only
/// rewritten when the user picks a new color, so untouched colors keep their exact floats.
pub fn color(ui: &mut Ui, label: &'static str, rgb: &mut [f32; 3]) -> bool {
    let mut changed = false;
    split_row(
        ui,
        24.0,
        |ui| {
            text::label(
                ui,
                tr(label),
                typography::mono_medium(11.0),
                colors::ON_SURFACE,
            );
        },
        |ui| {
            let byte = |c: f32| (c.clamp(0.0, 1.0) * 255.0 + 0.5) as u8;
            let before = Color32::from_rgb(byte(rgb[0]), byte(rgb[1]), byte(rgb[2]));
            let mut picked = before;
            if ui.color_edit_button_srgba(&mut picked).changed() && picked != before {
                *rgb = [
                    picked.r() as f32 / 255.0,
                    picked.g() as f32 / 255.0,
                    picked.b() as f32 / 255.0,
                ];
                changed = true;
            }
        },
    );
    changed
}
