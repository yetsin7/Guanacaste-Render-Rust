//! The library header: title, scene context, presets and the VRAM cache readout.

use crate::components::{icon_button::IconButton, icons, layout::split_row, tag::Tag, text};
use crate::config::theme::{colors, typography};
use crate::services::localization::tr;
use eframe::egui::{self, Margin, Sense, Ui, Vec2};

fn divider(ui: &mut Ui, height: f32) {
    let (rect, _) = ui.allocate_exact_size(Vec2::new(1.0, height), Sense::hover());
    ui.painter().rect_filled(rect, 0.0, colors::SURFACE_HIGHEST);
}

/// Draws the header bar.
pub fn show(ui: &mut Ui) {
    egui::Frame::new()
        .fill(colors::SURFACE_LOW)
        .inner_margin(Margin::symmetric(8, 4))
        .show(ui, |ui| {
            split_row(
                ui,
                28.0,
                |ui| {
                    ui.spacing_mut().item_spacing.x = 8.0;
                    text::icon(ui, icons::NATURE, 18.0, colors::PRIMARY);
                    text::label(
                        ui,
                        text::caps(tr("library.title")),
                        typography::headline_sm(),
                        colors::ON_SURFACE,
                    );
                    divider(ui, 12.0);
                    text::label(
                        ui,
                        tr("library.context"),
                        typography::label_sm(),
                        colors::OUTLINE,
                    );
                    Tag::new(tr("library.scene"), colors::SECONDARY)
                        .fill(colors::SURFACE_CONTAINER)
                        .padding(8.0, 2.0)
                        .show(ui);
                    text::label(
                        ui,
                        tr("library.stable"),
                        typography::mono_medium(11.0),
                        colors::TERTIARY,
                    );
                },
                |ui| {
                    ui.spacing_mut().item_spacing.x = 4.0;
                    Tag::new(tr("library.vram"), colors::OUTLINE)
                        .part(format!(" {}", tr("library.vram_value")), colors::PRIMARY)
                        .fill(colors::SURFACE_LOWEST)
                        .padding(8.0, 4.0)
                        .show(ui);
                    ui.add_space(4.0);
                    divider(ui, 14.0);
                    ui.add_space(4.0);
                    Tag::new(tr("library.presets"), colors::ON_SURFACE)
                        .lead_icon(icons::FILTER_LIST, 15.0, colors::OUTLINE)
                        .gap(4.0)
                        .fill(colors::SURFACE_CONTAINER)
                        .padding(8.0, 4.0)
                        .show(ui);
                    IconButton::new(icons::NOTIFICATIONS)
                        .sizes(16.0, 28.0)
                        .badge(true)
                        .tooltip(tr("library.notifications"))
                        .show(ui);
                },
            );
        });
}
