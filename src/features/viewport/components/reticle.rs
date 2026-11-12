//! The placement reticle: a ringed crosshair with the snapped object's label underneath.

use crate::components::icons;
use crate::components::{tag::Tag, text};
use crate::config::theme::colors;
use eframe::egui::{pos2, vec2, Align, Layout, Pos2, Stroke, Ui, UiBuilder, Vec2};

/// Paints the reticle centered on `center` with a label made of `name` and `snap`.
pub fn show(ui: &mut Ui, center: Pos2, name: &str, snap: &str) {
    let painter = ui.painter();
    let ring = colors::with_alpha(colors::PRIMARY, 204);
    painter.circle_filled(
        center,
        20.0,
        colors::with_alpha(colors::SURFACE_LOWEST, 102),
    );
    painter.circle_stroke(center, 20.0, Stroke::new(1.0, ring));
    painter.circle_stroke(
        center,
        32.0,
        Stroke::new(1.0, colors::with_alpha(colors::PRIMARY, 64)),
    );
    text::paint_icon(painter, center, icons::ADD, 18.0, colors::PRIMARY);
    for (dx, dy) in [(0.0, -1.0), (0.0, 1.0), (-1.0, 0.0), (1.0, 0.0)] {
        let direction = vec2(dx, dy);
        let start = center + direction * 35.0;
        let end = center + direction * 43.0;
        painter.line_segment([start, end], Stroke::new(2.0, colors::PRIMARY));
    }
    let area = eframe::egui::Rect::from_min_size(
        pos2(center.x - 160.0, center.y + 48.0),
        Vec2::new(320.0, 24.0),
    );
    let mut label = ui.new_child(
        UiBuilder::new()
            .max_rect(area)
            .layout(Layout::top_down(Align::Center)),
    );
    Tag::new(name, colors::ON_SURFACE)
        .part(format!(" | {snap}"), colors::OUTLINE)
        .dot(colors::TERTIARY)
        .gap(6.0)
        .padding(8.0, 2.0)
        .fill(colors::with_alpha(colors::SURFACE_LOW, 230))
        .show(&mut label);
}
