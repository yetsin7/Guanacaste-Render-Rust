//! The inspector header: title, scene chip, telemetry and metadata filter.

use crate::components::{
    icon_button::IconButton, icons, layout::split_row, search_field::SearchField, tag::Tag, text,
};
use crate::config::theme::{colors, typography};
use crate::services::localization::tr;
use eframe::egui::{self, Margin, Ui};

/// Draws the header; `filter` is the text of the metadata filter box.
pub fn show(ui: &mut Ui, filter: &mut String) {
    egui::Frame::new()
        .fill(colors::SURFACE_LOW)
        .inner_margin(Margin::symmetric(16, 4))
        .show(ui, |ui| {
            split_row(
                ui,
                32.0,
                |ui| {
                    ui.spacing_mut().item_spacing.x = 8.0;
                    text::icon(ui, icons::VIEW_IN_AR, 18.0, colors::PRIMARY);
                    text::label(
                        ui,
                        tr("inspector.title"),
                        typography::headline_sm(),
                        colors::ON_SURFACE,
                    );
                    ui.add_space(8.0);
                    Tag::new(tr("inspector.scene"), colors::ON_SURFACE_VARIANT)
                        .fill(colors::SURFACE_HIGHEST)
                        .padding(8.0, 2.0)
                        .show(ui);
                    Tag::new("37 FPS", colors::TERTIARY)
                        .font(typography::mono_medium(11.0))
                        .part(format!(" {}", tr("inspector.rt")), colors::OUTLINE)
                        .dot(colors::TERTIARY)
                        .gap(6.0)
                        .fill(colors::SURFACE_LOWEST)
                        .padding(8.0, 2.0)
                        .show(ui);
                },
                |ui| {
                    ui.spacing_mut().item_spacing.x = 8.0;
                    IconButton::new(icons::DOWNLOAD)
                        .sizes(16.0, 28.0)
                        .tooltip(tr("inspector.download"))
                        .show(ui);
                    IconButton::bell()
                        .sizes(16.0, 28.0)
                        .tooltip(tr("library.notifications"))
                        .show(ui);
                    SearchField::new(tr("inspector.filter_hint"))
                        .width(256.0)
                        .height(28.0)
                        .fill(colors::SURFACE_LOWEST)
                        .show(ui, filter);
                },
            );
        });
}
