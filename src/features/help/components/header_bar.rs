//! The help header: close, title, chips, search and notifications.

use crate::components::{
    icon_button::IconButton, icons, layout::split_row, search_field::SearchField, tag::Tag, text,
};
use crate::config::theme::{colors, typography};
use crate::services::localization::tr;
use eframe::egui::{self, CornerRadius, Margin, Ui};

/// Draws the header; returns true when the close button was clicked.
pub fn show(ui: &mut Ui, search: &mut String) -> bool {
    let mut closed = false;
    egui::Frame::new()
        .fill(colors::SURFACE_LOW)
        .corner_radius(CornerRadius::same(4))
        .inner_margin(Margin::symmetric(8, 4))
        .show(ui, |ui| {
            split_row(
                ui,
                32.0,
                |ui| {
                    ui.spacing_mut().item_spacing.x = 8.0;
                    closed = IconButton::new(icons::CLOSE)
                        .sizes(16.0, 28.0)
                        .tooltip(tr("help.close"))
                        .show(ui)
                        .clicked();
                    ui.add_space(4.0);
                    text::label(
                        ui,
                        tr("help.title"),
                        typography::headline_sm(),
                        colors::ON_SURFACE,
                    );
                    Tag::new(tr("help.scene"), colors::PRIMARY)
                        .fill(colors::SURFACE_HIGHEST)
                        .padding(6.0, 2.0)
                        .show(ui);
                    Tag::new(tr("help.status"), colors::TERTIARY)
                        .dot(colors::TERTIARY)
                        .gap(4.0)
                        .fill(colors::SURFACE_HIGHEST)
                        .padding(6.0, 2.0)
                        .show(ui);
                },
                |ui| {
                    ui.spacing_mut().item_spacing.x = 8.0;
                    IconButton::new(icons::NOTIFICATIONS)
                        .sizes(16.0, 28.0)
                        .badge(true)
                        .tooltip(tr("library.notifications"))
                        .show(ui);
                    SearchField::new(tr("help.search_hint"))
                        .width(256.0)
                        .height(28.0)
                        .font(typography::body_sm(), 14.0)
                        .show(ui, search);
                },
            );
        });
    closed
}
