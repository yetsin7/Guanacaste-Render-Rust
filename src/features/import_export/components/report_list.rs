//! The import report: warnings and notes with counts, and the skipped entity types.

use crate::components::{icons, text};
use crate::config::theme::{colors, typography};
use crate::features::import_export::logic::report_text::message;
use crate::features::import_export::models::{ImportReport, Severity};
use crate::services::localization::{tr, trf};
use eframe::egui::{self, ScrollArea, Ui};

/// Draws the report inside a scroll area of at most `height` points.
pub fn show(ui: &mut Ui, report: &ImportReport, height: f32) {
    let mut entries: Vec<_> = report.entries.iter().collect();
    entries.sort_by_key(|entry| std::cmp::Reverse(entry.severity));
    ScrollArea::vertical()
        .max_height(height)
        .auto_shrink([false, true])
        .show(ui, |ui| {
            ui.spacing_mut().item_spacing.y = 4.0;
            if entries.is_empty() {
                text::label(
                    ui,
                    tr("import.report_empty"),
                    typography::body_sm(),
                    colors::TERTIARY,
                );
            }
            for entry in entries {
                let (glyph, color) = match entry.severity {
                    Severity::Error => (icons::ERROR, colors::ERROR),
                    Severity::Warning => (icons::WARNING, colors::WARNING),
                    Severity::Info => (icons::INFO, colors::SECONDARY),
                };
                ui.horizontal_top(|ui| {
                    ui.spacing_mut().item_spacing.x = 8.0;
                    text::icon(ui, glyph, 16.0, color);
                    let times = if entry.count > 1 {
                        format!("  ×{}", entry.count)
                    } else {
                        String::new()
                    };
                    ui.add(
                        egui::Label::new(
                            egui::RichText::new(format!(
                                "{}{times}",
                                message(entry.key, &entry.args)
                            ))
                            .font(typography::body_sm())
                            .color(colors::ON_SURFACE),
                        )
                        .wrap(),
                    );
                });
            }
            if !report.skipped_types.is_empty() {
                let mut kinds: Vec<_> = report.skipped_types.iter().collect();
                kinds.sort_by_key(|(_, count)| std::cmp::Reverse(**count));
                let list: Vec<String> = kinds
                    .iter()
                    .take(8)
                    .map(|(kind, count)| format!("{kind} ×{count}"))
                    .collect();
                ui.add_space(4.0);
                text::paragraph(
                    ui,
                    trf("import.skipped_types", &[&list.join(", ")]),
                    typography::label_sm(),
                    colors::OUTLINE,
                    15.0,
                );
            }
        });
}
