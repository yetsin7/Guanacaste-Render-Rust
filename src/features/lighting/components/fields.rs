//! Small labeled numeric fields shared by the lighting panel.

use crate::features::lighting::logic::kelvin_to_rgb;
use crate::features::lighting::models::Vec3;
use crate::services::localization::tr;
use eframe::egui::{self, Color32, DragValue, Rgba, Sense, Ui, Vec2};
use std::ops::RangeInclusive;

/// A caption followed by a draggable number with a unit suffix.
pub fn number(
    ui: &mut Ui,
    label: &str,
    value: &mut f32,
    range: RangeInclusive<f32>,
    speed: f64,
    suffix: &str,
) -> bool {
    ui.horizontal(|ui| {
        ui.label(label);
        ui.add(
            DragValue::new(value)
                .range(range)
                .speed(speed)
                .suffix(suffix),
        )
        .changed()
    })
    .inner
}

/// A caption followed by three draggable numbers labeled X, Y and Z.
pub fn vector(ui: &mut Ui, label: &str, value: &mut Vec3, speed: f64, suffix: &str) {
    ui.horizontal(|ui| {
        ui.label(label);
        for (axis, component) in [
            tr("lighting.axis_x"),
            tr("lighting.axis_y"),
            tr("lighting.axis_z"),
        ]
        .into_iter()
        .zip(value.iter_mut())
        {
            ui.add(
                DragValue::new(component)
                    .speed(speed)
                    .prefix(axis)
                    .suffix(suffix),
            );
        }
    });
}

/// A slider from 0 to 1 shown as a percentage.
pub fn percent(ui: &mut Ui, label: &str, value: &mut f32) {
    ui.horizontal(|ui| {
        ui.label(label);
        let mut shown = *value * 100.0;
        if ui
            .add(egui::Slider::new(&mut shown, 0.0..=100.0).suffix(tr("lighting.unit_percent")))
            .changed()
        {
            *value = shown / 100.0;
        }
    });
}

/// A color temperature slider with a swatch of the resulting color.
pub fn kelvin(ui: &mut Ui, label: &str, value: &mut f32) {
    ui.horizontal(|ui| {
        ui.label(label);
        ui.add(
            egui::Slider::new(value, 1667.0..=25000.0)
                .logarithmic(true)
                .suffix(tr("lighting.unit_kelvin")),
        );
        let color = kelvin_to_rgb(*value);
        let (rect, _) = ui.allocate_exact_size(Vec2::new(18.0, 18.0), Sense::hover());
        ui.painter().rect_filled(
            rect,
            3.0,
            Color32::from(Rgba::from_rgb(color.r, color.g, color.b)),
        );
    });
}
