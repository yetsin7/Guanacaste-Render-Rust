//! The scene being assembled while the document is read: hierarchy, meshes, materials, levels and the report.

use super::json::{list, number, text};
use super::materials::MaterialBook;
use super::part::{Key, Part};
use crate::features::import_export::models::{
    ImportReport, Level, Mesh, NodeKind, SceneNode, Severity,
};
use serde_json::Value;
use std::collections::HashMap;

/// Coordinates beyond this (meters) are corrupt data, not architecture.
pub const MAX_COORDINATE: f32 = 1.0e7;

/// Hierarchy node, level index and elevation of a level, by the level's id in the document.
type LevelEntry = (usize, usize, f64);

/// Everything the element builders append to.
pub struct Context {
    /// Hierarchy; node 0 is the project.
    pub nodes: Vec<SceneNode>,
    /// All meshes, one per material per element.
    pub meshes: Vec<Mesh>,
    /// The materials in use.
    pub book: MaterialBook,
    /// Levels by document id.
    pub levels: HashMap<String, LevelEntry>,
    /// What was read, skipped and approximated.
    pub report: ImportReport,
}

fn sane(mesh: &Mesh) -> bool {
    mesh.positions
        .iter()
        .all(|p| p.iter().all(|v| v.is_finite() && v.abs() < MAX_COORDINATE))
}

impl Context {
    /// An empty scene whose root is the project named in `document`.
    pub fn new(document: &Value, source_folder: Option<&std::path::Path>) -> Self {
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
        let mut book = MaterialBook::default();
        book.read_overrides(document, source_folder);
        Self {
            nodes: vec![root],
            meshes: Vec::new(),
            book,
            levels: HashMap::new(),
            report: ImportReport::default(),
        }
    }

    /// Adds a node per level (sorted by elevation) and returns the levels.
    pub fn add_levels(&mut self, document: &Value) -> Vec<Level> {
        let mut sorted: Vec<&Value> = list(document, "Levels").iter().collect();
        sorted.sort_by(|a, b| number(a, "Elevation").total_cmp(&number(b, "Elevation")));
        let mut levels = Vec::new();
        for item in sorted {
            let node = self.nodes.len();
            self.nodes.push(SceneNode {
                name: text(item, "Name").to_owned(),
                class: "Level".into(),
                kind: NodeKind::Level,
                type_name: None,
                parent: Some(0),
                children: Vec::new(),
                meshes: Vec::new(),
                level: Some(levels.len()),
            });
            self.nodes[0].children.push(node);
            self.levels.insert(
                text(item, "Id").to_owned(),
                (node, levels.len(), number(item, "Elevation")),
            );
            levels.push(Level {
                name: text(item, "Name").to_owned(),
                elevation: number(item, "Elevation"),
                node,
            });
        }
        levels
    }

    /// Elevation of the level the item stands on (0 when it names none).
    pub fn elevation(&self, item: &Value) -> f64 {
        self.levels
            .get(text(item, "LevelId"))
            .map_or(0.0, |(_, _, z)| *z)
    }

    /// Elevation of the level with document id `id`.
    pub fn level_elevation(&self, id: &str) -> Option<f64> {
        self.levels.get(id).map(|(_, _, z)| *z)
    }

    /// Adds one element node for `item` with the triangles of `part`. An empty or corrupt part counts as a failed
    /// element and is reported. `name` defaults to the item's name, then its class.
    pub fn element(&mut self, item: &Value, class: &str, name: Option<&str>, part: Part) {
        self.element_on(item, text(item, "LevelId"), class, name, part);
    }

    /// Like [`element`](Self::element) for an item that does not carry its own level (a door or window takes the
    /// level of its host wall).
    pub fn element_on(
        &mut self,
        item: &Value,
        level_id: &str,
        class: &str,
        name: Option<&str>,
        mut part: Part,
    ) {
        self.report.elements_total += 1;
        let (parent, level) = self
            .levels
            .get(level_id)
            .map_or((0, None), |(node, level, _)| (*node, Some(*level)));
        let label = name
            .into_iter()
            .chain([text(item, "Name"), class])
            .find(|n| !n.is_empty())
            .unwrap_or(class)
            .to_owned();
        for (surface, material) in self.book.overrides_of(text(item, "Id")) {
            part.retarget(surface, Key::Custom(material));
        }
        let index = self.nodes.len();
        let mut node = SceneNode {
            name: label,
            class: class.to_owned(),
            kind: NodeKind::Element,
            type_name: None,
            parent: Some(parent),
            children: Vec::new(),
            meshes: Vec::new(),
            level,
        };
        for (key, mut mesh) in part.into_groups() {
            if !sane(&mesh) {
                continue;
            }
            mesh.material = Some(self.book.resolve(key));
            self.meshes.push(mesh);
            node.meshes.push(self.meshes.len() - 1);
        }
        if node.meshes.is_empty() {
            self.report.elements_failed += 1;
            self.report
                .note(Severity::Warning, "legacy.degenerate", &[class]);
        } else {
            self.report.elements_with_geometry += 1;
        }
        self.nodes.push(node);
        self.nodes[parent].children.push(index);
    }
}
