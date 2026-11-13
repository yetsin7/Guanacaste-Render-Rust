//! Read-only importer of Guanacaste 3D projects (`G3DNATV1` container): levels, walls, slabs, columns and
//! structural elements become simple solids. The original file is never modified.
//!
//! Container layout (little endian, 81-byte header): `"G3DNATV1"`, `i32` container version (1), `u8` document
//! kind (1 = project), `i32` schema version, 16-byte GUID, `i64` uncompressed length, `i64` compressed length,
//! 32-byte SHA-256 of the uncompressed JSON, then the gzip-compressed UTF-8 JSON.

use crate::features::import_export::logic::prism::{band, prism, rectangle};
use crate::features::import_export::models::{
    ImportError, ImportOutcome, ImportReport, ImportedScene, Level, Material, Mesh, NodeKind,
    SceneNode, Severity, SourceFormat, SourceInfo,
};
use crate::features::import_export::services::legacy_container::{invalid, MAX_PAYLOAD};
use crate::features::import_export::services::progress::Progress;

pub use crate::features::import_export::services::legacy_container::{read_container, SIGNATURE};
use serde_json::Value;
use std::collections::HashMap;
use std::path::Path;

/// Lists of the Guanacaste 3D document that carry no 3D geometry in this importer.
/// Most elements a project may declare before it is refused as hostile or corrupt.
const MAX_ELEMENTS: usize = 1_000_000;
/// Most points of one slab or roof outline (the ear-clipping triangulation is quadratic).
const MAX_OUTLINE_POINTS: usize = 5_000;
/// Coordinates beyond this (meters) are corrupt data, not architecture.
const MAX_COORDINATE: f32 = 1.0e7;

const IGNORED_LISTS: [&str; 6] = [
    "Rooms",
    "ElectricalDevices",
    "PlumbingPoints",
    "Sheets",
    "Dimensions",
    "Callouts",
];

fn number(value: &Value, key: &str) -> f64 {
    value.get(key).and_then(Value::as_f64).unwrap_or(0.0)
}

fn point(value: &Value, key: &str) -> [f64; 2] {
    value
        .get(key)
        .map_or([0.0, 0.0], |p| [number(p, "X"), number(p, "Y")])
}

fn text<'a>(value: &'a Value, key: &str) -> &'a str {
    value.get(key).and_then(Value::as_str).unwrap_or("")
}

fn list<'a>(document: &'a Value, key: &str) -> &'a [Value] {
    document
        .get(key)
        .and_then(Value::as_array)
        .map_or(&[], Vec::as_slice)
}

/// Parses `#RRGGBB` into RGBA.
fn hex_color(text: &str) -> Option<[f32; 4]> {
    let digits = text.strip_prefix('#').filter(|d| d.len() == 6)?;
    let channel = |at: usize| {
        u8::from_str_radix(&digits[at..at + 2], 16)
            .ok()
            .map(|v| v as f32 / 255.0)
    };
    Some([channel(0)?, channel(2)?, channel(4)?, 1.0])
}

struct Builder {
    nodes: Vec<SceneNode>,
    meshes: Vec<Mesh>,
    materials: Vec<Material>,
    level_nodes: HashMap<String, (usize, usize, f64)>,
    report: ImportReport,
}

impl Builder {
    fn material(&mut self, name: &str, color: [f32; 4]) -> usize {
        if let Some(index) = self.materials.iter().position(|m| m.name == name) {
            return index;
        }
        self.materials.push(Material {
            name: name.to_owned(),
            color,
            texture: None,
        });
        self.materials.len() - 1
    }

    fn element(&mut self, item: &Value, class: &str, mesh: Option<Mesh>, material: usize) {
        self.report.elements_total += 1;
        let level_id = text(item, "LevelId");
        let (parent, level) = self
            .level_nodes
            .get(level_id)
            .map_or((0, None), |(node, level, _)| (*node, Some(*level)));
        let index = self.nodes.len();
        let name = [text(item, "Name"), class]
            .into_iter()
            .find(|n| !n.is_empty())
            .unwrap_or(class)
            .to_owned();
        let mut node = SceneNode {
            name,
            class: class.to_owned(),
            kind: NodeKind::Element,
            type_name: None,
            parent: Some(parent),
            children: Vec::new(),
            meshes: Vec::new(),
            level,
        };
        let sane = |m: &Mesh| {
            m.positions
                .iter()
                .all(|p| p.iter().all(|v| v.is_finite() && v.abs() < MAX_COORDINATE))
        };
        match mesh.filter(sane) {
            Some(mut mesh) => {
                mesh.material = Some(material);
                self.meshes.push(mesh);
                node.meshes.push(self.meshes.len() - 1);
                self.report.elements_with_geometry += 1;
            }
            None => {
                self.report.elements_failed += 1;
                self.report
                    .note(Severity::Warning, "legacy.degenerate", &[class]);
            }
        }
        self.nodes.push(node);
        self.nodes[parent].children.push(index);
    }

    fn elevation(&self, item: &Value) -> f64 {
        self.level_nodes
            .get(text(item, "LevelId"))
            .map_or(0.0, |(_, _, z)| *z)
    }
}

/// Builds the scene and its report from a parsed Guanacaste 3D document.
pub fn build(document: &Value, schema: i32, file_size: u64) -> (ImportedScene, ImportReport) {
    let root = SceneNode {
        name: text(document, "Name").to_owned(),
        class: "G3D".into(),
        kind: NodeKind::Project,
        type_name: None,
        parent: None,
        children: Vec::new(),
        meshes: Vec::new(),
        level: None,
    };
    let mut b = Builder {
        nodes: vec![root],
        meshes: Vec::new(),
        materials: Vec::new(),
        level_nodes: HashMap::new(),
        report: ImportReport::default(),
    };
    let mut levels: Vec<Level> = Vec::new();
    let mut sorted: Vec<&Value> = list(document, "Levels").iter().collect();
    sorted.sort_by(|a, b| number(a, "Elevation").total_cmp(&number(b, "Elevation")));
    for item in sorted {
        let node = b.nodes.len();
        b.nodes.push(SceneNode {
            name: text(item, "Name").to_owned(),
            class: "Level".into(),
            kind: NodeKind::Level,
            type_name: None,
            parent: Some(0),
            children: Vec::new(),
            meshes: Vec::new(),
            level: Some(levels.len()),
        });
        b.nodes[0].children.push(node);
        b.level_nodes.insert(
            text(item, "Id").to_owned(),
            (node, levels.len(), number(item, "Elevation")),
        );
        levels.push(Level {
            name: text(item, "Name").to_owned(),
            elevation: number(item, "Elevation"),
            node,
        });
    }
    let wall_types: HashMap<&str, &Value> = list(document, "WallTypes")
        .iter()
        .map(|t| (text(t, "Id"), t))
        .collect();
    for wall in list(document, "Walls") {
        let base = b.elevation(wall);
        let top = wall
            .get("TopLevelId")
            .and_then(Value::as_str)
            .and_then(|id| b.level_nodes.get(id))
            .map(|(_, _, z)| z + number(wall, "TopOffset"));
        let height = top.map_or(number(wall, "Height"), |top| top - base);
        let kind = wall_types.get(text(wall, "WallTypeId"));
        let material_name = kind.map_or("Muro", |t| text(t, "MaterialName"));
        let color = kind
            .and_then(|t| hex_color(text(t, "SectionFillColor")))
            .unwrap_or([0.78, 0.76, 0.72, 1.0]);
        let material = b.material(material_name, color);
        let mesh = prism(
            &band(
                point(wall, "Start"),
                point(wall, "End"),
                number(wall, "Thickness"),
            ),
            base,
            base + height,
        );
        b.element(wall, "Wall", mesh, material);
    }
    for floor in list(document, "Floors") {
        let top = b.elevation(floor) + number(floor, "ElevationOffset");
        let boundary = list(floor, "Boundary");
        let outline: Vec<[f64; 2]> = if boundary.len() > MAX_OUTLINE_POINTS {
            Vec::new()
        } else {
            boundary
                .iter()
                .map(|p| [number(p, "X"), number(p, "Y")])
                .collect()
        };
        let roof = floor.get("Kind").and_then(Value::as_i64) == Some(2);
        let material = if roof {
            b.material("Cubierta", [0.55, 0.36, 0.30, 1.0])
        } else {
            b.material("Losa", [0.70, 0.70, 0.68, 1.0])
        };
        if roof && number(floor, "Pitch") > 0.0 {
            b.report.note(Severity::Warning, "legacy.roof_flat", &[]);
        }
        let mesh = prism(&outline, top - number(floor, "Thickness").max(0.01), top);
        b.element(floor, if roof { "Roof" } else { "Floor" }, mesh, material);
    }
    for column in list(document, "Columns") {
        let base = b.elevation(column);
        let material = b.material("Columna", [0.66, 0.66, 0.66, 1.0]);
        if column.get("Shape").and_then(Value::as_i64).unwrap_or(0) != 0 {
            b.report.note(Severity::Info, "legacy.column_box", &[]);
        }
        let mesh = prism(
            &rectangle(
                point(column, "Position"),
                number(column, "Width"),
                number(column, "Depth"),
            ),
            base,
            base + number(column, "Height"),
        );
        b.element(column, "Column", mesh, material);
    }
    for element in list(document, "StructuralElements") {
        if number(element, "Height") <= 0.0 {
            // Voids such as stair openings carry no solid.
            *b.report
                .skipped_types
                .entry("StructuralElement (sin altura)".to_owned())
                .or_insert(0) += 1;
            continue;
        }
        let base = b.elevation(element) + number(element, "Offset");
        let (start, end) = (point(element, "Start"), point(element, "End"));
        let width = number(element, "Width");
        let mut footprint = band(start, end, width);
        if footprint.is_empty() {
            let depth = number(element, "Depth");
            footprint = rectangle(start, width, if depth > 0.0 { depth } else { width });
        }
        let material = b.material("Estructura", [0.62, 0.64, 0.68, 1.0]);
        let mesh = prism(&footprint, base, base + number(element, "Height"));
        b.element(element, "StructuralElement", mesh, material);
    }
    let openings = list(document, "Openings").len();
    if openings > 0 {
        b.report.note(
            Severity::Warning,
            "legacy.openings_not_cut",
            &[&openings.to_string()],
        );
    }
    for key in IGNORED_LISTS {
        let count = list(document, key).len();
        if count > 0 {
            b.report.skipped_types.insert(key.to_owned(), count);
        }
    }
    if b.report.elements_total == 0 {
        b.report.note(Severity::Warning, "legacy.empty", &[]);
    }
    b.report.note(Severity::Info, "legacy.read_only", &[]);
    let scene = ImportedScene {
        source: SourceInfo {
            format: SourceFormat::Legacy,
            version: format!("G3DNATV1 · esquema {schema}"),
            application: "Guanacaste 3D".into(),
            length_unit: "m".into(),
            unit_scale: 1.0,
            file_size,
        },
        nodes: b.nodes,
        meshes: b.meshes,
        materials: b.materials,
        levels,
        origin_shift: [0.0; 3],
    };
    (scene, b.report)
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
