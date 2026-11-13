//! A short message floating at the bottom-left corner (saved, opened, exported, or what failed).

use super::{frames, notice::Level, text};
use crate::config::theme::{colors, typography};
use eframe::egui::{self, Align2, Context, Id, Order};

/// Draws `message` with the look of `level`, `margin` pixels from the bottom-left corner of the window.
pub fn show(ctx: &Context, level: Level, message: &str, margin: f32) {
    let (accent, glyph) = level.look();
    let corner = ctx.content_rect().left_bottom() + egui::vec2(margin, -margin);
    egui::Area::new(Id::new("toast"))
        .order(Order::Tooltip)
        .pivot(Align2::LEFT_BOTTOM)
        .fixed_pos(corner)
        .interactable(false)
        .show(ctx, |ui| {
            frames::floating(10).show(ui, |ui| {
                ui.set_max_width(560.0);
                ui.horizontal_top(|ui| {
                    ui.spacing_mut().item_spacing.x = 8.0;
                    text::icon(ui, glyph, 16.0, accent);
                    text::paragraph(ui, message, typography::body_sm(), colors::ON_SURFACE, 18.0);
                });
            });
        });
}
