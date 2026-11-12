//! The preview of what an imported file contains.

use crate::components::{label_value, text};
use crate::config::theme::{colors, typography};
use crate::features::import_export::models::{ImportReport, ImportedScene};
use crate::services::localization::{tr, trf};
use eframe::egui::{self, vec2, CornerRadius, Margin, Ui};

fn size_text(bytes: u64) -> String {
    if bytes >= 1 << 30 {
        format!("{:.2} GB", bytes as f64 / (1u64 << 30) as f64)
    } else if bytes >= 1 << 20 {
        format!("{:.1} MB", bytes as f64 / (1u64 << 20) as f64)
    } else {
        format!("{:.1} KB", bytes as f64 / 1024.0)
    }
}

/// Draws the summary grid.
pub fn show(ui: &mut Ui, scene: &ImportedScene, report: &ImportReport) {
    let size = scene.bounds().map_or_else(
        || "—".to_owned(),
        |b| {
            format!(
                "{:.1} × {:.1} × {:.1} m",
                b.max[0] - b.min[0],
                b.max[1] - b.min[1],
                b.max[2] - b.min[2]
            )
        },
    );
    let application = if scene.source.application.is_empty() {
        "—".to_owned()
    } else {
        scene.source.application.clone()
    };
    let cells = [
        (
            "import.format",
            format!("{} · {}", scene.source.format.name(), scene.source.version),
            colors::PRIMARY,
        ),
        ("import.application", application, colors::ON_SURFACE),
        (
            "import.units",
            format!(
                "{} (1 = {} m)",
                scene.source.length_unit, scene.source.unit_scale
            ),
            colors::ON_SURFACE,
        ),
        (
            "import.file_size",
            size_text(scene.source.file_size),
            colors::ON_SURFACE,
        ),
        (
            "import.elements",
            format!(
                "{} / {}",
                report.elements_with_geometry, report.elements_total
            ),
            colors::TERTIARY,
        ),
        (
            "import.levels",
            scene.levels.len().to_string(),
            colors::ON_SURFACE,
        ),
        (
            "import.materials",
            scene.materials.len().to_string(),
            colors::ON_SURFACE,
        ),
        (
            "import.triangles",
            format!("{}", scene.triangle_count()),
            colors::ON_SURFACE,
        ),
        ("import.model_size", size, colors::ON_SURFACE),
        (
            "import.elapsed",
            format!("{} ms", report.duration_ms),
            colors::OUTLINE,
        ),
    ];
    egui::Frame::new()
        .fill(colors::SURFACE_CONTAINER)
        .corner_radius(CornerRadius::same(2))
        .inner_margin(Margin::same(10))
        .show(ui, |ui| {
            ui.set_width(ui.available_width());
            let columns = 5;
            let width = (ui.available_width() - 12.0 * (columns - 1) as f32) / columns as f32;
            for row in cells.chunks(columns) {
                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing.x = 12.0;
                    for (caption, value, color) in row {
                        ui.allocate_ui(vec2(width, 32.0), |ui| {
                            ui.set_width(width);
                            label_value::show(ui, tr(caption), colors::OUTLINE, value, *color);
                        });
                    }
                });
                ui.add_space(4.0);
            }
            if !scene.levels.is_empty() {
                let names: Vec<String> = scene
                    .levels
                    .iter()
                    .take(8)
                    .map(|level| format!("{} ({:.2} m)", level.name, level.elevation))
                    .collect();
                text::label(
                    ui,
                    trf("import.level_list", &[&names.join(" · ")]),
                    typography::label_sm(),
                    colors::ON_SURFACE_VARIANT,
                );
            }
        });
}
