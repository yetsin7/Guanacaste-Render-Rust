//! Final assembly of the scene: report, origin shift, meshes per material and sorted levels.

use super::build::Builder;
use super::units;
use crate::features::import_export::logic::geometry::V3;
use crate::features::import_export::models::{
    ImportError, ImportOutcome, ImportedScene, Mesh, Severity, SourceFormat, SourceInfo,
};
use crate::features::import_export::services::step::StepFile;
use std::collections::HashMap;

pub(super) fn finish(
    mut builder: Builder,
    file: &StepFile,
    file_size: u64,
) -> Result<ImportOutcome, ImportError> {
    let mut report = std::mem::take(&mut builder.ctx.report);
    report.skipped_types = file.skipped.clone();
    if file.malformed > 0 {
        report.note(
            Severity::Warning,
            "ifc.malformed",
            &[&file.malformed.to_string()],
        );
    }
    let dropped: usize = file.skipped.values().sum();
    if dropped > 0 {
        report.note(
            Severity::Info,
            "ifc.properties_skipped",
            &[&dropped.to_string()],
        );
    }
    if builder.levels.is_empty() {
        report.note(Severity::Info, "ifc.no_levels", &[]);
    }
    // Shift to the model center so `f32` vertices stay precise even with large shared coordinates.
    let (mut min, mut max) = ([f64::INFINITY; 3], [f64::NEG_INFINITY; 3]);
    for (_, _, piece) in &builder.world {
        for p in &piece.positions {
            for axis in 0..3 {
                min[axis] = min[axis].min(p[axis]);
                max[axis] = max[axis].max(p[axis]);
            }
        }
    }
    let shift: V3 = if min[0].is_finite() {
        [(min[0] + max[0]) / 2.0, (min[1] + max[1]) / 2.0, min[2]]
    } else {
        [0.0; 3]
    };
    let mut meshes: Vec<Mesh> = Vec::new();
    let mut by_key: HashMap<(usize, usize), usize> = HashMap::new();
    for (node, material, piece) in &builder.world {
        let slot = *by_key.entry((*node, *material)).or_insert_with(|| {
            meshes.push(Mesh {
                positions: Vec::new(),
                indices: Vec::new(),
                material: Some(*material),
            });
            builder.nodes[*node].meshes.push(meshes.len() - 1);
            meshes.len() - 1
        });
        let mesh = &mut meshes[slot];
        let base = mesh.positions.len() as u32;
        mesh.positions.extend(piece.positions.iter().map(|p| {
            [
                (p[0] - shift[0]) as f32,
                (p[1] - shift[1]) as f32,
                (p[2] - shift[2]) as f32,
            ]
        }));
        mesh.indices.extend(piece.indices.iter().map(|i| i + base));
    }
    let mut order: Vec<usize> = (0..builder.levels.len()).collect();
    order.sort_by(|a, b| {
        builder.levels[*a]
            .elevation
            .total_cmp(&builder.levels[*b].elevation)
    });
    let mut remap = vec![0usize; order.len()];
    for (new_index, old_index) in order.iter().enumerate() {
        remap[*old_index] = new_index;
    }
    builder.levels = order
        .iter()
        .map(|old| builder.levels[*old].clone())
        .collect();
    for node in &mut builder.nodes {
        node.level = node.level.map(|old| remap[old]);
    }
    let application = [&file.header.originating_system, &file.header.preprocessor]
        .iter()
        .find(|s| !s.is_empty())
        .map(|s| (*s).clone())
        .unwrap_or_default();
    let source = SourceInfo {
        format: SourceFormat::Ifc,
        version: file.header.schemas.join(", "),
        application,
        length_unit: units::length_label(builder.ctx.length),
        unit_scale: builder.ctx.length,
        file_size,
    };
    let scene = ImportedScene {
        source,
        nodes: builder.nodes,
        meshes,
        materials: builder.palette,
        levels: builder.levels,
        origin_shift: shift,
    };
    Ok(ImportOutcome { scene, report })
}
