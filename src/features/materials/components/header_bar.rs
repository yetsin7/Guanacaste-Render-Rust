//! The editor header: title, scene chips, parameter filter and tools.

use crate::components::{
    icon_button::IconButton, icons, layout::split_row, search_field::SearchField, tag::Tag, text,
};
use crate::config::theme::{colors, typography};
use crate::services::localization::tr;
use eframe::egui::{self, Margin, Ui};

/// Draws the header; `filter` is the parameter filter text.
pub fn show(ui: &mut Ui, filter: &mut String) {
    egui::Frame::new()
        .fill(colors::SURFACE_LOW)
        .inner_margin(Margin::symmetric(8, 2))
        .show(ui, |ui| {
            split_row(
                ui,
                32.0,
                |ui| {
                    ui.spacing_mut().item_spacing.x = 8.0;
                    text::icon(ui, icons::TEXTURE, 16.0, colors::PRIMARY);
                    text::label(
                        ui,
                        text::caps(tr("material.title")),
                        typography::label_md(),
                        colors::PRIMARY,
                    );
                    Tag::new(tr("material.scene"), colors::ON_SURFACE_VARIANT)
                        .fill(colors::SURFACE_CONTAINER)
                        .padding(4.0, 2.0)
                        .show(ui);
                    Tag::new(tr("material.stats"), colors::TERTIARY)
                        .fill(colors::SURFACE_CONTAINER)
                        .padding(4.0, 2.0)
                        .show(ui);
                },
                |ui| {
                    ui.spacing_mut().item_spacing.x = 4.0;
                    IconButton::new(icons::SYNC)
                        .sizes(16.0, 28.0)
                        .tooltip(tr("material.sync"))
                        .show(ui);
                    IconButton::new(icons::NOTIFICATIONS)
                        .sizes(16.0, 28.0)
                        .tooltip(tr("library.notifications"))
                        .show(ui);
                    SearchField::new(tr("material.filter_hint"))
                        .width(192.0)
                        .height(28.0)
                        .font(typography::label_sm(), 14.0)
                        .show(ui, filter);
                },
            );
        });
}
