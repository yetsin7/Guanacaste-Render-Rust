//! Plain-text rendering of an import report, for the console command and the integration test.

use crate::features::import_export::models::{ImportReport, ImportedScene, Severity};
use crate::services::localization::{tr, trf};

/// The localized message of a report entry with its arguments substituted.
pub fn message(key: &'static str, args: &[String]) -> String {
    let refs: Vec<&str> = args.iter().map(String::as_str).collect();
    trf(key, &refs)
}

/// One line per report entry, most severe first.
pub fn entry_lines(report: &ImportReport) -> Vec<String> {
    let mut entries: Vec<_> = report.entries.iter().collect();
    entries.sort_by_key(|entry| std::cmp::Reverse(entry.severity));
    entries
        .into_iter()
        .map(|entry| {
            let label = match entry.severity {
                Severity::Error => "ERROR",
                Severity::Warning => "AVISO",
                Severity::Info => "INFO",
            };
            let times = if entry.count > 1 {
                format!(" (x{})", entry.count)
            } else {
                String::new()
            };
            format!("[{label}] {}{times}", message(entry.key, &entry.args))
        })
        .collect()
}

/// Element counts per source class, most numerous first.
pub fn class_counts(scene: &ImportedScene) -> Vec<(String, usize)> {
    let mut counts: std::collections::BTreeMap<String, usize> = std::collections::BTreeMap::new();
    for node in scene
        .nodes
        .iter()
        .filter(|n| n.kind == crate::features::import_export::models::NodeKind::Element)
    {
        *counts.entry(node.class.clone()).or_insert(0) += 1;
    }
    let mut list: Vec<_> = counts.into_iter().collect();
    list.sort_by_key(|(_, count)| std::cmp::Reverse(*count));
    list
}

/// A complete plain-text report: summary of the scene followed by every entry.
pub fn full_text(scene: &ImportedScene, report: &ImportReport) -> String {
    let mut lines = vec![
        format!(
            "{}: {} ({})",
            tr("import.format"),
            scene.source.format.name(),
            scene.source.version
        ),
        format!("{}: {}", tr("import.application"), scene.source.application),
        format!(
            "{}: {} (1 = {} m)",
            tr("import.units"),
            scene.source.length_unit,
            scene.source.unit_scale
        ),
        format!(
            "{}: {} ({} con geometría, {} fallidos)",
            tr("import.elements"),
            report.elements_total,
            report.elements_with_geometry,
            report.elements_failed
        ),
        format!("{}: {}", tr("import.levels"), scene.levels.len()),
        format!("{}: {}", tr("import.materials"), scene.materials.len()),
        format!(
            "{}: {} / {}",
            tr("import.triangles"),
            scene.triangle_count(),
            scene.vertex_count()
        ),
        format!("Tiempo: {} ms", report.duration_ms),
    ];
    for (class, count) in class_counts(scene) {
        lines.push(format!("  {class}: {count}"));
    }
    lines.extend(entry_lines(report));
    for (kind, count) in &report.skipped_types {
        lines.push(format!("[OMITIDO] {kind} x{count}"));
    }
    lines.join("\n")
}
