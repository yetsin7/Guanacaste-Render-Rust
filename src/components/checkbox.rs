//! A 14 px checkbox with a primary checkmark.

use super::text;
use crate::config::theme::colors;
use eframe::egui::{CornerRadius, CursorIcon, Response, Sense, Ui, Vec2};

/// Draws a checkbox bound to `checked`; the response is marked changed when toggled.
pub fn show(ui: &mut Ui, checked: &mut bool) -> Response {
    let (rect, mut response) = ui.allocate_exact_size(Vec2::splat(16.0), Sense::click());
    if response
        .clone()
        .on_hover_cursor(CursorIcon::PointingHand)
        .clicked()
    {
        *checked = !*checked;
        response.mark_changed();
    }
    let painter = ui.painter();
    if *checked {
        painter.rect_filled(rect, CornerRadius::same(2), colors::PRIMARY);
        text::paint_icon(
            painter,
            rect.center(),
            super::icons::CHECK,
            14.0,
            colors::ON_PRIMARY,
        );
    } else {
        super::paint::framed(
            painter,
            rect,
            2,
            colors::SURFACE_LOWEST,
            colors::OUTLINE_VARIANT,
        );
    }
    response
}
