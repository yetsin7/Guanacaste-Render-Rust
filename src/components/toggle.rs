//! Toggle switch: a pill track with a round thumb that slides with a short animation.

use crate::config::theme::colors;
use eframe::egui::{pos2, CornerRadius, CursorIcon, Response, Sense, Ui, Vec2};

/// Draws a 32x16 switch bound to `on`; returns the response (changed when clicked).
pub fn show(ui: &mut Ui, on: &mut bool) -> Response {
    show_sized(ui, on, Vec2::new(32.0, 16.0))
}

/// Draws a switch of the given size.
pub fn show_sized(ui: &mut Ui, on: &mut bool, size: Vec2) -> Response {
    let (rect, mut response) = ui.allocate_exact_size(size, Sense::click());
    let response_cursor = response.clone().on_hover_cursor(CursorIcon::PointingHand);
    if response_cursor.clicked() {
        *on = !*on;
        response.mark_changed();
    }
    let t = ui.ctx().animate_bool_with_time(response.id, *on, 0.12);
    let painter = ui.painter();
    let track = if *on {
        colors::SECONDARY_CONTAINER
    } else {
        colors::SURFACE_HIGHEST
    };
    painter.rect_filled(rect, CornerRadius::same((size.y / 2.0) as u8), track);
    let thumb_radius = size.y / 2.0 - 2.0;
    let travel = size.x - size.y;
    let center = pos2(rect.left() + size.y / 2.0 + travel * t, rect.center().y);
    let thumb = if *on {
        colors::ON_SECONDARY_CONTAINER
    } else {
        colors::OUTLINE
    };
    painter.circle_filled(center, thumb_radius, thumb);
    response
}
