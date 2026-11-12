//! Underlined tabs.

use super::text;
use crate::config::theme::{colors, typography};
use eframe::egui::{pos2, vec2, Align2, CursorIcon, Rect, Sense, Ui};

/// Draws the tabs left to right; returns true when the selection changed.
pub fn show(ui: &mut Ui, labels: &[&str], selected: &mut usize) -> bool {
    let mut changed = false;
    ui.spacing_mut().item_spacing.x = 4.0;
    for (index, label) in labels.iter().enumerate() {
        let width = text::text_width(ui, label, &typography::label_md()) + 24.0;
        let (rect, response) = ui.allocate_exact_size(vec2(width, 32.0), Sense::click());
        let response = response.on_hover_cursor(CursorIcon::PointingHand);
        let active = index == *selected;
        let ink = if active {
            colors::PRIMARY
        } else if response.hovered() {
            colors::ON_SURFACE
        } else {
            colors::ON_SURFACE_VARIANT
        };
        text::paint_text(
            ui.painter(),
            rect.center(),
            Align2::CENTER_CENTER,
            label,
            typography::label_md(),
            ink,
        );
        if active {
            ui.painter().rect_filled(
                Rect::from_min_max(pos2(rect.left(), rect.bottom() - 2.0), rect.max),
                0.0,
                colors::PRIMARY_CONTAINER,
            );
        }
        if response.clicked() && !active {
            *selected = index;
            changed = true;
        }
    }
    changed
}
