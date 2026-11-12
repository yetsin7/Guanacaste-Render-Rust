//! The strip under the catalog: scatter tools, pagination and engine status.

use crate::components::{icons, layout::split_row_centered, pagination, tag::Tag};
use crate::config::theme::colors;
use crate::services::localization::tr;
use crate::services::sample_data::library as sample;
use eframe::egui::{self, CornerRadius, Margin, Ui};

/// Draws the footer; `page` is the current 0-based page.
pub fn show(ui: &mut Ui, page: &mut usize) {
    egui::Frame::new()
        .fill(colors::SURFACE_LOW)
        .corner_radius(CornerRadius::same(2))
        .inner_margin(Margin::symmetric(8, 4))
        .show(ui, |ui| {
            split_row_centered(
                ui,
                28.0,
                pagination::width_for(sample::PAGES),
                |ui| {
                    ui.spacing_mut().item_spacing.x = 8.0;
                    tool_tag(ui, "library.cluster", icons::EFFECTS, colors::PRIMARY);
                    tool_tag(ui, "library.scatter", icons::SCATTER, colors::SECONDARY);
                },
                |ui| {
                    pagination::show(ui, page, sample::PAGES);
                },
                |ui| {
                    ui.spacing_mut().item_spacing.x = 12.0;
                    Tag::new(tr("library.instances"), colors::OUTLINE)
                        .part(format!(" {}", sample::PLACED_INSTANCES), colors::ON_SURFACE)
                        .padding(0.0, 0.0)
                        .show(ui);
                    Tag::new(tr("library.engine"), colors::OUTLINE)
                        .part(format!(" {}", tr("library.engine_value")), colors::TERTIARY)
                        .dot(colors::TERTIARY)
                        .gap(4.0)
                        .padding(0.0, 0.0)
                        .show(ui);
                },
            );
        });
}

fn tool_tag(ui: &mut Ui, key: &'static str, glyph: char, color: eframe::egui::Color32) {
    Tag::new(tr(key), colors::ON_SURFACE_VARIANT)
        .lead_icon(glyph, 15.0, color)
        .gap(4.0)
        .fill(colors::SURFACE_CONTAINER)
        .padding(8.0, 4.0)
        .show(ui);
}
