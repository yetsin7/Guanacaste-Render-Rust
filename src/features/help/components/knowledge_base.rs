//! The knowledge base card: two documentation links.

use crate::components::{icons, layout::split_row, text};
use crate::config::theme::{colors, typography};
use crate::services::localization::tr;
use eframe::egui::{self, pos2, vec2, CornerRadius, Margin, Rect, Sense, Ui, UiBuilder};

/// Draws the card.
pub fn show(ui: &mut Ui) {
    egui::Frame::new()
        .fill(colors::SURFACE_LOW)
        .corner_radius(CornerRadius::same(8))
        .inner_margin(Margin::same(12))
        .show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.spacing_mut().item_spacing.y = 8.0;
            split_row(
                ui,
                18.0,
                |ui| {
                    text::label(
                        ui,
                        text::caps(tr("help.knowledge")),
                        typography::mono_medium(11.0),
                        colors::OUTLINE,
                    );
                },
                |ui| {
                    text::label(
                        ui,
                        tr("help.docs_version"),
                        typography::label_sm(),
                        colors::PRIMARY,
                    );
                },
            );
            for (glyph, color, title, note) in [
                (
                    icons::EFFECTS,
                    colors::PRIMARY,
                    "help.kb_rt",
                    "help.kb_rt_note",
                ),
                (
                    icons::NATURE,
                    colors::TERTIARY,
                    "help.kb_veg",
                    "help.kb_veg_note",
                ),
            ] {
                let (rect, response) =
                    ui.allocate_exact_size(vec2(ui.available_width(), 52.0), Sense::click());
                let fill = if response.hovered() {
                    colors::SURFACE_HIGH
                } else {
                    colors::SURFACE_CONTAINER
                };
                ui.painter().rect_filled(rect, CornerRadius::same(4), fill);
                let tile = Rect::from_min_size(
                    pos2(rect.left() + 8.0, rect.center().y - 16.0),
                    vec2(32.0, 32.0),
                );
                ui.painter()
                    .rect_filled(tile, CornerRadius::same(2), colors::SURFACE_HIGHEST);
                text::paint_icon(ui.painter(), tile.center(), glyph, 18.0, color);
                text::paint_icon(
                    ui.painter(),
                    pos2(rect.right() - 18.0, rect.center().y),
                    icons::OPEN_IN_NEW,
                    18.0,
                    colors::OUTLINE,
                );
                let mut info = ui.new_child(UiBuilder::new().max_rect(Rect::from_min_max(
                    pos2(tile.right() + 12.0, rect.top() + 8.0),
                    pos2(rect.right() - 36.0, rect.bottom()),
                )));
                info.spacing_mut().item_spacing.y = 0.0;
                crate::components::text_layout::clamped(
                    &mut info,
                    tr(title),
                    typography::body_md(),
                    colors::ON_SURFACE,
                    rect.width() - 100.0,
                    1,
                    18.0,
                );
                crate::components::text_layout::clamped(
                    &mut info,
                    tr(note),
                    typography::body_sm(),
                    colors::ON_SURFACE_VARIANT,
                    rect.width() - 100.0,
                    1,
                    16.0,
                );
            }
        });
}
