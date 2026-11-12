//! The diagnostics strip at the bottom: GPU status and the support actions.

use crate::components::{
    button::{Button, ButtonStyle},
    icons,
    layout::split_row,
    tag::Tag,
};
use crate::config::theme::{colors, typography};
use crate::services::localization::tr;
use eframe::egui::{self, CornerRadius, Margin, Ui};

/// Draws the strip.
pub fn show(ui: &mut Ui) {
    egui::Frame::new()
        .fill(colors::SURFACE_LOW)
        .corner_radius(CornerRadius::same(8))
        .inner_margin(Margin::same(12))
        .show(ui, |ui| {
            ui.set_width(ui.available_width());
            split_row(
                ui,
                32.0,
                |ui| {
                    ui.spacing_mut().item_spacing.x = 16.0;
                    Tag::new(tr("help.gpu"), colors::ON_SURFACE)
                        .lead_icon(icons::CHECK_CIRCLE, 18.0, colors::TERTIARY)
                        .gap(8.0)
                        .font(typography::body_md())
                        .padding(0.0, 0.0)
                        .show(ui);
                    Tag::new(tr("help.vram_status"), colors::ON_SURFACE)
                        .lead_icon(icons::MEMORY, 18.0, colors::PRIMARY)
                        .gap(8.0)
                        .font(typography::body_md())
                        .padding(0.0, 0.0)
                        .show(ui);
                    Tag::new(tr("help.buffer"), colors::OUTLINE)
                        .padding(0.0, 0.0)
                        .show(ui);
                },
                |ui| {
                    ui.spacing_mut().item_spacing.x = 8.0;
                    Button::new(tr("help.support"), ButtonStyle::Container)
                        .icon(icons::FORUM, 16.0)
                        .font(typography::label_sm())
                        .padding(12.0, 6.0)
                        .disabled(tr("help.unavailable"))
                        .show(ui);
                    Button::new(tr("help.pdf"), ButtonStyle::Container)
                        .icon(icons::DOWNLOAD, 16.0)
                        .font(typography::label_sm())
                        .padding(12.0, 6.0)
                        .disabled(tr("help.unavailable"))
                        .show(ui);
                },
            );
        });
}
