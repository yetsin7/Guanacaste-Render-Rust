//! Read-only importer of Guanacaste 3D projects (`G3DNATV1` container). It rebuilds the geometry the original
//! application draws: walls with their doors and windows (frames, leaves, sashes, glass), slabs with stair openings,
//! pitched roofs, columns, stairs, guards, curtain walls and cobogó screens, residential components, and the
//! detailed embedded families (trees, plants, cars) with their textures, all with PBR materials by surface.
//! The original file is never modified.
//!
//! Container layout (little endian, 81-byte header): `"G3DNATV1"`, `i32` container version (1), `u8` document
//! kind (1 = project), `i32` schema version, 16-byte GUID, `i64` uncompressed length, `i64` compressed length,
//! 32-byte SHA-256 of the uncompressed JSON, then the gzip-compressed UTF-8 JSON.

mod axis;
mod base64;
mod circulation;
mod components;
mod context;
mod elements;
mod families;
mod json;
mod leaves;
mod materials;
mod openings;
mod part;
mod placed;
mod roofs;
mod slabs;
mod surface;
mod textures;
mod walls;

use crate::features::import_export::models::{
    ImportError, ImportOutcome, ImportReport, ImportedScene, Severity, SourceFormat, SourceInfo,
};
use crate::features::import_export::services::legacy_container::{invalid, MAX_PAYLOAD};
use crate::features::import_export::services::progress::Progress;
use context::Context;
use json::list;
use serde_json::Value;
use std::path::Path;

pub use crate::features::import_export::services::legacy_container::{read_container, SIGNATURE};

/// Most elements a project may declare before it is refused as hostile or corrupt.
const MAX_ELEMENTS: usize = 1_000_000;

/// Lists of the Guanacaste 3D document that carry no 3D geometry in this importer.
const IGNORED_LISTS: [&str; 6] = [
    "Rooms",
    "ElectricalDevices",
    "PlumbingPoints",
    "Sheets",
    "Dimensions",
    "Callouts",
];

/// Builds the scene and its report from a parsed Guanacaste 3D document.
pub fn build(document: &Value, schema: i32, file_size: u64) -> (ImportedScene, ImportReport) {
    let mut ctx = Context::new(document);
    let levels = ctx.add_levels(document);
    let mut families = families::Families::new(document);
    walls::build(&mut ctx, document);
    slabs::build_floors(&mut ctx, document);
    slabs::build_columns(&mut ctx, document);
    elements::build(&mut ctx, document, &mut families);
    for key in IGNORED_LISTS {
        let count = list(document, key).len();
        if count > 0 {
            ctx.report.skipped_types.insert(key.to_owned(), count);
        }
    }
    if ctx.report.elements_total == 0 {
        ctx.report.note(Severity::Warning, "legacy.empty", &[]);
    }
    ctx.report.note(Severity::Info, "legacy.read_only", &[]);
    let scene = ImportedScene {
        source: SourceInfo {
            format: SourceFormat::Legacy,
            version: format!("G3DNATV1 · esquema {schema}"),
            application: "Guanacaste 3D".into(),
            length_unit: "m".into(),
            unit_scale: 1.0,
            file_size,
        },
        nodes: ctx.nodes,
        meshes: ctx.meshes,
        materials: ctx.book.into_materials(),
        levels,
        origin_shift: [0.0; 3],
    };
    (scene, ctx.report)
}

/// Imports the Guanacaste 3D project at `path` without modifying it.
pub fn import(path: &Path, progress: &Progress) -> Result<ImportOutcome, ImportError> {
    let started = std::time::Instant::now();
    let size = std::fs::metadata(path)?.len();
    if size > MAX_PAYLOAD {
        return Err(invalid(
            "El proyecto de Guanacaste 3D supera el límite de 256 MB.",
        ));
    }
    let bytes = std::fs::read(path)?;
    progress.check()?;
    let (schema, document) = read_container(&bytes)?;
    progress.set(0.5);
    drop(bytes);
    let declared: usize = [
        "Walls",
        "Floors",
        "Columns",
        "StructuralElements",
        "Levels",
        "Openings",
    ]
    .iter()
    .map(|key| list(&document, key).len())
    .sum();
    if declared > MAX_ELEMENTS {
        return Err(invalid(
            "El proyecto declara más de un millón de elementos: parece dañado.",
        ));
    }
    let (scene, mut report) = build(&document, schema, size);
    report.duration_ms = started.elapsed().as_millis();
    progress.set(1.0);
    Ok(ImportOutcome { scene, report })
}
