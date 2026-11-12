//! Modal dialog frame: a scrim over the whole window and a centered floating panel with title bar.

use super::{frames, icon_button::IconButton, icons, text};
use crate::config::theme::{colors, typography};
use crate::services::localization::tr;
use eframe::egui::{
    self, Align, Align2, Color32, Context, Id, Key, LayerId, Layout, Order, Sense, Ui, Vec2,
};

/// Shows a dialog titled `title` and `width` points wide; returns true when the user asked to close it (close
/// button or Escape). `body` draws the content.
pub fn show(
    ctx: &Context,
    id: &'static str,
    title: &str,
    glyph: char,
    width: f32,
    body: impl FnOnce(&mut Ui),
) -> bool {
    let screen = ctx.content_rect();
    egui::Area::new(Id::new((id, "scrim")))
        .order(Order::Foreground)
        .fixed_pos(screen.min)
        .show(ctx, |ui| {
            let (rect, _) = ui.allocate_exact_size(screen.size(), Sense::click_and_drag());
            ui.painter()
                .rect_filled(rect, 0.0, Color32::from_black_alpha(150));
        });
    let mut closed = false;
    let panel_id = Id::new((id, "panel"));
    egui::Area::new(panel_id)
        .order(Order::Foreground)
        .anchor(Align2::CENTER_CENTER, Vec2::ZERO)
        .show(ctx, |ui| {
            frames::floating(0).show(ui, |ui| {
                ui.set_width(width);
                ui.spacing_mut().item_spacing = Vec2::new(8.0, 0.0);
                ui.horizontal(|ui| {
                    ui.set_height(40.0);
                    ui.add_space(12.0);
                    text::icon(ui, glyph, 18.0, colors::PRIMARY);
                    text::label(ui, title, typography::headline_sm(), colors::ON_SURFACE);
                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        ui.add_space(8.0);
                        if IconButton::new(icons::CLOSE)
                            .sizes(16.0, 28.0)
                            .fill(None)
                            .tooltip(tr("dialog.close"))
                            .show(ui)
                            .clicked()
                        {
                            closed = true;
                        }
                    });
                });
                divider(ui);
                body(ui);
            });
        });
    ctx.move_to_top(LayerId::new(Order::Foreground, panel_id));
    closed || ctx.input(|input| input.key_pressed(Key::Escape))
}

/// A hairline across the dialog width.
pub fn divider(ui: &mut Ui) {
    let (rect, _) = ui.allocate_exact_size(Vec2::new(ui.available_width(), 1.0), Sense::hover());
    ui.painter().rect_filled(rect, 0.0, colors::SURFACE_HIGHEST);
}

/// The dialog content area: 16 px padding and 12 px between items.
pub fn content(ui: &mut Ui, add: impl FnOnce(&mut Ui)) {
    egui::Frame::new()
        .inner_margin(egui::Margin::same(16))
        .show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.spacing_mut().item_spacing.y = 12.0;
            add(ui);
        });
}

/// The footer strip: a divider and right-aligned action buttons (added last-to-first).
pub fn footer(ui: &mut Ui, add: impl FnOnce(&mut Ui)) {
    divider(ui);
    egui::Frame::new()
        .fill(colors::with_alpha(colors::SURFACE_HIGH, 120))
        .inner_margin(egui::Margin::symmetric(16, 10))
        .show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                ui.spacing_mut().item_spacing.x = 8.0;
                add(ui);
            });
        });
}
