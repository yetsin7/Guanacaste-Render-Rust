//! The batch render dispatcher bar at the bottom of the repository.

use crate::components::{
    button::{Button, ButtonStyle},
    icons,
    layout::split_row,
    tag::Tag,
    text,
};
use crate::config::theme::{colors, typography};
use crate::services::localization::tr;
use eframe::egui::{self, CornerRadius, Ui};

/// Draws the bar; returns true when "Export all" was clicked.
pub fn show(ui: &mut Ui) -> bool {
    let mut exported = false;
    egui::Frame::new()
        .fill(colors::SURFACE_LOW)
        .corner_radius(CornerRadius::same(2))
        .inner_margin(egui::Margin::same(12))
        .show(ui, |ui| {
            split_row(
                ui,
                36.0,
                |ui| {
                    ui.spacing_mut().item_spacing.x = 4.0;
                    text::icon(ui, icons::HUB, 22.0, colors::PRIMARY);
                    ui.vertical(|ui| {
                        ui.spacing_mut().item_spacing.y = 0.0;
                        text::label(
                            ui,
                            tr("projects.batch_title"),
                            typography::headline_sm(),
                            colors::ON_SURFACE,
                        );
                        text::label(
                            ui,
                            tr("projects.batch_status"),
                            typography::label_sm(),
                            colors::OUTLINE,
                        );
                    });
                    ui.add_space(12.0);
                    Tag::new(tr("projects.batch_node"), colors::ON_SURFACE_VARIANT)
                        .dot(colors::TERTIARY)
                        .gap(8.0)
                        .padding(8.0, 4.0)
                        .fill(colors::SURFACE_CONTAINER)
                        .show(ui);
                },
                |ui| {
                    ui.spacing_mut().item_spacing.x = 8.0;
                    exported = Button::new(tr("projects.batch_export"), ButtonStyle::Pale)
                        .icon(icons::LAUNCH, 16.0)
                        .show(ui)
                        .clicked();
                    Button::new(tr("projects.view_queue"), ButtonStyle::Raised)
                        .icon(icons::QUEUE, 16.0)
                        .show(ui);
                    ui.add_space(4.0);
                    text::label(
                        ui,
                        tr("projects.batch_preset_value"),
                        typography::mono_medium(11.0),
                        colors::PRIMARY,
                    );
                    text::label(
                        ui,
                        tr("projects.batch_preset"),
                        typography::label_sm(),
                        colors::OUTLINE,
                    );
                },
            );
        });
    exported
}
