//! The editor header: title, project chips and the material filter.

use crate::components::{
    icon_button::IconButton, icons, layout::split_row, search_field::SearchField, tag::Tag, text,
};
use crate::config::theme::{colors, typography};
use crate::services::localization::{tr, trf};
use eframe::egui::{self, Margin, Ui};

/// Draws the header. `project` is the open project's name, `count` the number of materials and `filter` the list
/// filter text.
pub fn show(ui: &mut Ui, project: Option<&str>, count: usize, filter: &mut String) {
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
                    let scope = match project {
                        Some(name) => trf("material.scene", &[name]),
                        None => tr("material.session").to_owned(),
                    };
                    Tag::new(scope, colors::ON_SURFACE_VARIANT)
                        .fill(colors::SURFACE_CONTAINER)
                        .padding(4.0, 2.0)
                        .show(ui);
                    Tag::new(
                        trf("material.stats", &[&count.to_string()]),
                        colors::TERTIARY,
                    )
                    .fill(colors::SURFACE_CONTAINER)
                    .padding(4.0, 2.0)
                    .show(ui);
                },
                |ui| {
                    ui.spacing_mut().item_spacing.x = 4.0;
                    IconButton::bell()
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
