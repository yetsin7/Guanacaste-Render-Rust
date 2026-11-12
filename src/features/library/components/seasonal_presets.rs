//! The four seasonal foliage presets.

use crate::components::{icons, layout::split_row, text};
use crate::config::theme::{colors, typography};
use crate::features::library::models::InspectorState;
use crate::services::localization::tr;
use eframe::egui::{
    self, pos2, vec2, Align2, CornerRadius, CursorIcon, Margin, Sense, Stroke, StrokeKind, Ui,
};

const SEASONS: [(char, &str); 4] = [
    (icons::NATURE, "inspector.summer"),
    (icons::LEAF, "inspector.autumn"),
    (icons::SNOW, "inspector.winter"),
    (icons::SPA, "inspector.spring"),
];

/// Draws the panel and updates the selected season.
pub fn show(ui: &mut Ui, state: &mut InspectorState) {
    egui::Frame::new()
        .fill(colors::SURFACE_LOW)
        .corner_radius(CornerRadius::same(8))
        .inner_margin(Margin::same(12))
        .show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.spacing_mut().item_spacing.y = 8.0;
            split_row(
                ui,
                14.0,
                |ui| {
                    text::label(
                        ui,
                        text::caps(tr("inspector.seasons")),
                        typography::label_sm(),
                        colors::OUTLINE,
                    );
                },
                |ui| {
                    text::label(
                        ui,
                        tr("inspector.variations"),
                        typography::label_sm(),
                        colors::SECONDARY,
                    );
                },
            );
            let gap = 4.0;
            let width = (ui.available_width() - gap * 3.0) / 4.0;
            let (row, _) = ui.allocate_exact_size(vec2(ui.available_width(), 56.0), Sense::hover());
            for (index, (glyph, key)) in SEASONS.into_iter().enumerate() {
                let rect = egui::Rect::from_min_size(
                    pos2(row.left() + index as f32 * (width + gap), row.top()),
                    vec2(width, 56.0),
                );
                let response = ui
                    .interact(rect, ui.id().with(("season", index)), Sense::click())
                    .on_hover_cursor(CursorIcon::PointingHand);
                let selected = state.season == index;
                let fill = if selected || response.hovered() {
                    colors::SURFACE_HIGH
                } else {
                    colors::SURFACE_CONTAINER
                };
                let ink = if selected {
                    colors::PRIMARY
                } else {
                    colors::ON_SURFACE_VARIANT
                };
                let painter = ui.painter();
                painter.rect_filled(rect, CornerRadius::same(2), fill);
                if selected {
                    painter.rect_stroke(
                        rect,
                        CornerRadius::same(2),
                        Stroke::new(2.0, colors::PRIMARY),
                        StrokeKind::Inside,
                    );
                }
                text::paint_icon(
                    painter,
                    pos2(rect.center().x, rect.top() + 18.0),
                    glyph,
                    18.0,
                    ink,
                );
                let font = if selected {
                    typography::mono_medium(11.0)
                } else {
                    typography::label_sm()
                };
                text::paint_text(
                    painter,
                    pos2(rect.center().x, rect.top() + 40.0),
                    Align2::CENTER_CENTER,
                    tr(key),
                    font,
                    ink,
                );
                if response.clicked() {
                    state.season = index;
                }
            }
        });
}
