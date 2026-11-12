//! A small popup menu anchored to a point, closed by clicking outside.

use super::{frames, text};
use crate::config::theme::{colors, typography};
use eframe::egui::{self, Align2, Context, Id, Order, Pos2, Sense, Vec2};

/// One menu entry.
pub struct MenuItem<'a> {
    /// Leading icon.
    pub glyph: char,
    /// Label.
    pub label: &'a str,
    /// Shortcut hint shown on the right.
    pub shortcut: &'a str,
}

/// Shows the menu with its top-right corner at `anchor`; returns `Some(Some(index))` when an entry was chosen,
/// `Some(None)` when the menu should close, and `None` while it stays open.
pub fn show(
    ctx: &Context,
    id: &'static str,
    anchor: Pos2,
    items: &[MenuItem],
) -> Option<Option<usize>> {
    let mut chosen = None;
    let response = egui::Area::new(Id::new(id))
        .order(Order::Foreground)
        .pivot(Align2::RIGHT_TOP)
        .fixed_pos(anchor)
        .show(ctx, |ui| {
            frames::floating(4).show(ui, |ui| {
                ui.set_width(240.0);
                ui.spacing_mut().item_spacing.y = 2.0;
                for (index, item) in items.iter().enumerate() {
                    let (rect, response) = ui
                        .allocate_exact_size(Vec2::new(ui.available_width(), 28.0), Sense::click());
                    if response.hovered() {
                        ui.painter().rect_filled(rect, 2.0, colors::SURFACE_HIGH);
                    }
                    text::paint_icon(
                        ui.painter(),
                        rect.left_center() + Vec2::new(14.0, 0.0),
                        item.glyph,
                        16.0,
                        colors::ON_SURFACE_VARIANT,
                    );
                    text::paint_text(
                        ui.painter(),
                        rect.left_center() + Vec2::new(32.0, 0.0),
                        Align2::LEFT_CENTER,
                        item.label,
                        typography::body_md(),
                        colors::ON_SURFACE,
                    );
                    text::paint_text(
                        ui.painter(),
                        rect.right_center() - Vec2::new(8.0, 0.0),
                        Align2::RIGHT_CENTER,
                        item.shortcut,
                        typography::mono(10.0),
                        colors::OUTLINE,
                    );
                    if response.clicked() {
                        chosen = Some(index);
                    }
                }
            });
        })
        .response;
    if chosen.is_some() {
        return Some(chosen);
    }
    let clicked_outside =
        ctx.input(|input| input.pointer.any_click()) && !response.contains_pointer();
    (clicked_outside || ctx.input(|input| input.key_pressed(egui::Key::Escape))).then_some(None)
}
