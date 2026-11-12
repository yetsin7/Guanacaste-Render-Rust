//! A small caption over a value: the "ENGINE STATE / DXR Path Tracer" pairs of the technical panels.

use super::text;
use crate::config::theme::typography;
use eframe::egui::{Color32, Ui};

/// Draws `caption` (10 px, uppercase) above `value` (12 px medium) in a vertical stack.
pub fn show(ui: &mut Ui, caption: &str, caption_color: Color32, value: &str, value_color: Color32) {
    ui.vertical(|ui| {
        ui.spacing_mut().item_spacing.y = 0.0;
        text::label(
            ui,
            text::caps(caption),
            typography::mono(10.0),
            caption_color,
        );
        text::label(ui, value, typography::label_md(), value_color);
    });
}
