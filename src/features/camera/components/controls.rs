//! Small building blocks shared by the camera panels.

use crate::components::{layout::split_row, slider::Slider, text};
use crate::config::theme::{colors, typography};
use crate::services::localization::trf;
use eframe::egui::Ui;

/// A caption on the left and a value on the right.
pub fn readout(ui: &mut Ui, caption: &str, value: String) {
    split_row(
        ui,
        18.0,
        |ui| {
            text::label(ui, caption, typography::body_sm(), colors::ON_SURFACE);
        },
        |ui| {
            text::label(ui, value, typography::mono_medium(11.0), colors::PRIMARY);
        },
    );
}

/// A captioned slider over the normalized position `t`; returns true when the user moved it.
pub fn slider(ui: &mut Ui, caption: &str, value: String, t: &mut f32) -> bool {
    readout(ui, caption, value);
    ui.add_space(-6.0);
    Slider::precision().show(ui, t).changed()
}

/// Millimetres as text, e.g. `50 mm`.
pub fn mm(value: f32) -> String {
    trf("camera.value_mm", &[&trim(value, 1)])
}

/// Metres as text with a precision that suits the size, e.g. `0.25 m`, `3.3 m`, `250 m`.
pub fn metres(value: f32) -> String {
    let digits = if value < 1.0 {
        2
    } else if value < 100.0 {
        1
    } else {
        0
    };
    trf("camera.value_m", &[&format!("{value:.digits$}")])
}

/// Degrees as text, e.g. `39.6°`.
pub fn degrees(radians: f32) -> String {
    format!("{:.1}°", radians.to_degrees())
}

/// `value` with up to `digits` decimals and no trailing zeros.
pub fn trim(value: f32, digits: usize) -> String {
    let text = format!("{value:.digits$}");
    if text.contains('.') {
        text.trim_end_matches('0').trim_end_matches('.').to_owned()
    } else {
        text
    }
}

/// Index of the entry of `stops` closest to `value` on a logarithmic scale.
pub fn nearest_stop(stops: &[f32], value: f32) -> usize {
    let distance = |stop: f32| (stop.ln() - value.max(1e-6).ln()).abs();
    (0..stops.len())
        .min_by(|a, b| distance(stops[*a]).total_cmp(&distance(stops[*b])))
        .unwrap_or(0)
}
