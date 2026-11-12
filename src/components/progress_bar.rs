//! A flat progress bar.

use crate::config::theme::colors;
use eframe::egui::{vec2, Color32, CornerRadius, Rect, Sense, Ui};

/// Draws a bar `height` points tall filled to `fraction` (0..=1) with `fill` over a recessed track.
pub fn show(ui: &mut Ui, fraction: f32, fill: Color32, height: f32) {
    let (rect, _) = ui.allocate_exact_size(vec2(ui.available_width(), height), Sense::hover());
    ui.painter()
        .rect_filled(rect, CornerRadius::same(255), colors::SURFACE_LOWEST);
    let filled = Rect::from_min_size(
        rect.min,
        vec2(rect.width() * fraction.clamp(0.0, 1.0), height),
    );
    ui.painter()
        .rect_filled(filled, CornerRadius::same(255), fill);
}
