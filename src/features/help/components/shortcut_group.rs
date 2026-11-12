//! A shortcut reference card: a titled list of action and key pairs.

use crate::components::{key_cap::KeyCap, layout::split_row, text};
use crate::config::theme::{colors, typography};
use crate::features::help::logic::filter_shortcuts;
use crate::features::help::models::Shortcut;
use crate::services::localization::tr;
use eframe::egui::{self, Color32, CornerRadius, Margin, Ui};

/// Draws the card; `search` filters the rows and `glyph` is the corner icon.
pub fn show(
    ui: &mut Ui,
    title: &'static str,
    title_color: Color32,
    glyph: char,
    shortcuts: &[Shortcut],
    search: &str,
) {
    egui::Frame::new()
        .fill(colors::SURFACE_LOW)
        .corner_radius(CornerRadius::same(8))
        .inner_margin(Margin::same(12))
        .show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.spacing_mut().item_spacing.y = 6.0;
            split_row(
                ui,
                20.0,
                |ui| {
                    text::label(
                        ui,
                        text::caps(tr(title)),
                        typography::mono_medium(11.0),
                        title_color,
                    );
                },
                |ui| {
                    text::icon(ui, glyph, 16.0, colors::OUTLINE);
                },
            );
            let rows = filter_shortcuts(shortcuts, search);
            if rows.is_empty() {
                text::label(
                    ui,
                    tr("help.no_shortcuts"),
                    typography::body_sm(),
                    colors::OUTLINE,
                );
            }
            for shortcut in rows {
                egui::Frame::new()
                    .fill(colors::SURFACE_CONTAINER)
                    .corner_radius(CornerRadius::same(2))
                    .inner_margin(Margin::same(6))
                    .show(ui, |ui| {
                        ui.set_width(ui.available_width());
                        split_row(
                            ui,
                            22.0,
                            |ui| {
                                text::label(
                                    ui,
                                    shortcut.action,
                                    typography::body_sm(),
                                    colors::ON_SURFACE,
                                );
                            },
                            |ui| {
                                KeyCap::new(shortcut.keys)
                                    .colors(colors::SURFACE_HIGHEST, colors::PRIMARY)
                                    .font(typography::mono_medium(11.0))
                                    .show(ui);
                            },
                        );
                    });
            }
        });
}
