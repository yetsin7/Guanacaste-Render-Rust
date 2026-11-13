//! Collada (`.dae`) reader: the visual scene hierarchy with node transforms, polygon geometry, texture
//! coordinates and materials (diffuse color, transparency, texture file). Units (`<unit meter>`) and the up axis
//! come from the file. SketchUp's "Collada" export is the main source. Controllers, skinning, animation, lights
//! and cameras are ignored.

mod geometry;
mod materials;
mod transform;

use super::common::{finish, node, read_limited, source_info, too_big, MeshBuilder, MAX_INDICES};
use crate::features::import_export::logic::geometry::Mat4;
use crate::features::import_export::models::{
    ImportError, ImportOutcome, ImportReport, Material, Mesh, NodeKind, SceneNode, Severity,
    SourceFormat,
};
use crate::features::import_export::services::progress::Progress;
use geometry::{children, first, Geometry};
use roxmltree::{Document, Node, ParsingOptions};
use std::collections::HashMap;
use std::path::Path;
use std::rc::Rc;
use transform::{local_matrix, mirrored, upright, Up};

const MAX_FILE: u64 = 256 << 20;
const MAX_DEPTH: usize = 64;
const MAX_NODES: usize = 500_000;

/// Elements of the document found by `tag` and `id`.
pub struct Index<'a, 'input> {
    map: HashMap<String, Node<'a, 'input>>,
}

impl<'a, 'input> Index<'a, 'input> {
    fn new(document: &'a Document<'input>) -> Self {
        let mut map = HashMap::new();
        for n in document.descendants().filter(Node::is_element) {
            let tag = n.tag_name().name();
            if matches!(tag, "image" | "material" | "effect" | "node" | "geometry") {
                if let Some(id) = n.attribute("id") {
                    map.entry(format!("{tag}#{id}")).or_insert(n);
                }
            }
        }
        Self { map }
    }

    /// The element `<tag id="id">`.
    pub fn get(&self, tag: &str, id: &str) -> Option<Node<'a, 'input>> {
        self.map.get(&format!("{tag}#{id}")).copied()
    }
}

struct Importer<'a, 'input> {
    index: Index<'a, 'input>,
    folder: Option<&'a Path>,
    up: Up,
    scale: f64,
    nodes: Vec<SceneNode>,
    meshes: Vec<Mesh>,
    materials: Vec<Material>,
    material_ids: HashMap<String, Option<usize>>,
    geometries: HashMap<String, Option<Rc<Geometry>>>,
    report: ImportReport,
    progress: &'a Progress,
    indices: usize,
}

impl Importer<'_, '_> {
    fn material(&mut self, id: &str) -> Option<usize> {
        if let Some(found) = self.material_ids.get(id) {
            return *found;
        }
        let built = materials::build(&self.index, id, self.folder).map(|m| {
            self.materials.push(m);
            self.materials.len() - 1
        });
        if built.is_none() {
            self.report
                .note(Severity::Warning, "dae.material_missing", &[id]);
        }
        self.material_ids.insert(id.to_owned(), built);
        built
    }

    fn add_node(&mut self, name: &str, parent: usize) -> usize {
        self.nodes
            .push(node(name, "Collada", NodeKind::Group, Some(parent)));
        let at = self.nodes.len() - 1;
        self.nodes[parent].children.push(at);
        at
    }

    /// Instances geometry `id` under scene node `at`, placed by `world`; `binds` maps symbols to material ids.
    fn instance(
        &mut self,
        id: &str,
        binds: &[(String, String)],
        at: usize,
        world: &Mat4,
    ) -> Result<(), ImportError> {
        self.progress.check()?;
        if !self.geometries.contains_key(id) {
            let parsed = self
                .index
                .get("geometry", id)
                .and_then(|g| geometry::parse(g, &mut self.report))
                .map(Rc::new);
            self.geometries.insert(id.to_owned(), parsed);
        }
        let Some(Some(geometry)) = self.geometries.get(id).cloned() else {
            return Ok(());
        };
        let flip = mirrored(world);
        for primitive in &geometry.primitives {
            let symbol = primitive.symbol.clone().unwrap_or_default();
            let target = binds
                .iter()
                .find(|(s, _)| *s == symbol)
                .map_or(symbol, |(_, t)| t.clone());
            let material = (!target.is_empty())
                .then(|| self.material(&target))
                .flatten();
            self.indices += primitive.corners.len();
            if self.indices > MAX_INDICES {
                return Err(too_big());
            }
            let mut builder = MeshBuilder::default();
            for triangle in primitive.corners.chunks_exact(3) {
                let ids: Vec<u32> = triangle
                    .iter()
                    .map(|(p, uv)| {
                        let point = world.point(geometry.positions[*p as usize]);
                        let uv = geometry.uvs.get(*uv as usize).copied();
                        builder.vertex(upright(point, self.up, self.scale), uv)
                    })
                    .collect();
                if flip {
                    builder.triangle(ids[0], ids[2], ids[1]);
                } else {
                    builder.triangle(ids[0], ids[1], ids[2]);
                }
            }
            self.meshes.push(builder.finish(material));
            self.nodes[at].meshes.push(self.meshes.len() - 1);
            self.nodes[at].kind = NodeKind::Element;
        }
        Ok(())
    }

    fn visit(
        &mut self,
        source: Node,
        parent: usize,
        matrix: &Mat4,
        depth: usize,
    ) -> Result<(), ImportError> {
        if depth > MAX_DEPTH || self.nodes.len() > MAX_NODES {
            self.report.note(Severity::Warning, "dae.too_deep", &[]);
            return Ok(());
        }
        let world = matrix.mul(&local_matrix(source));
        let name = source
            .attribute("name")
            .or_else(|| source.attribute("id"))
            .unwrap_or("nodo");
        let at = self.add_node(name, parent);
        for instance in children(source, "instance_geometry") {
            let id = instance
                .attribute("url")
                .unwrap_or("")
                .trim_start_matches('#');
            let binds = materials::bindings(instance);
            self.instance(id, &binds, at, &world)?;
        }
        for instance in children(source, "instance_node") {
            let id = instance
                .attribute("url")
                .unwrap_or("")
                .trim_start_matches('#');
            if let Some(target) = self.index.get("node", id) {
                self.visit(target, at, &world, depth + 1)?;
            }
        }
        for child in children(source, "node") {
            self.visit(child, at, &world, depth + 1)?;
        }
        Ok(())
    }
}

fn text_of<'a>(root: Node<'a, '_>, path: &[&'static str]) -> Option<&'a str> {
    let mut current = root;
    for step in path {
        current = first(current, step)?;
    }
    current.text().map(str::trim)
}

/// Imports Collada `text`; `folder` is where textures are looked for.
pub fn import_text(
    text: &str,
    name: &str,
    folder: Option<&Path>,
    progress: &Progress,
) -> Result<ImportOutcome, ImportError> {
    let started = std::time::Instant::now();
    let options = ParsingOptions {
        allow_dtd: false,
        nodes_limit: 8_000_000,
    };
    let document = Document::parse_with_options(text, options)
        .map_err(|e| ImportError::Invalid(format!("El XML del archivo no es válido: {e}.")))?;
    let root = document.root_element();
    if root.tag_name().name() != "COLLADA" {
        return Err(ImportError::Invalid(
            "No es un archivo Collada: el elemento raíz no es COLLADA.".into(),
        ));
    }
    let scale = first(root, "asset")
        .and_then(|a| first(a, "unit"))
        .and_then(|u| u.attribute("meter"))
        .and_then(|m| m.parse::<f64>().ok())
        .filter(|m| m.is_finite() && *m > 0.0)
        .unwrap_or(1.0);
    let up = Up::parse(text_of(root, &["asset", "up_axis"]).unwrap_or(""));
    let mut importer = Importer {
        index: Index::new(&document),
        folder,
        up,
        scale,
        nodes: vec![node(name, "", NodeKind::Project, None)],
        meshes: Vec::new(),
        materials: Vec::new(),
        material_ids: HashMap::new(),
        geometries: HashMap::new(),
        report: ImportReport::default(),
        progress,
        indices: 0,
    };
    let scene = first(root, "scene")
        .and_then(|s| first(s, "instance_visual_scene"))
        .and_then(|i| i.attribute("url"))
        .map(|u| u.trim_start_matches('#'))
        .and_then(|id| {
            document.descendants().find(|n| {
                n.is_element()
                    && n.tag_name().name() == "visual_scene"
                    && n.attribute("id") == Some(id)
            })
        })
        .or_else(|| {
            document
                .descendants()
                .find(|n| n.is_element() && n.tag_name().name() == "visual_scene")
        });
    match scene {
        Some(scene) => {
            for top in children(scene, "node") {
                importer.visit(top, 0, &Mat4::IDENTITY, 0)?;
            }
        }
        None => {
            importer.report.note(Severity::Warning, "dae.no_scene", &[]);
            let ids: Vec<String> = document
                .descendants()
                .filter(|n| n.is_element() && n.tag_name().name() == "geometry")
                .filter_map(|n| n.attribute("id").map(str::to_owned))
                .collect();
            for id in ids {
                let at = importer.add_node(&id, 0);
                importer.instance(&id, &[], at, &Mat4::IDENTITY)?;
            }
        }
    }
    progress.set(1.0);
    let source = source_info(
        SourceFormat::Collada,
        &format!("Collada {}", root.attribute("version").unwrap_or("?")),
        text_of(root, &["asset", "contributor", "authoring_tool"]).unwrap_or(""),
        scale,
        text.len() as u64,
    );
    finish(
        source,
        importer.nodes,
        importer.meshes,
        importer.materials,
        importer.report,
        started,
    )
}

/// Imports the Collada file at `path`.
pub fn import(path: &Path, progress: &Progress) -> Result<ImportOutcome, ImportError> {
    let bytes = read_limited(path, MAX_FILE)?;
    progress.check()?;
    let text = String::from_utf8_lossy(&bytes);
    let name = path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("Collada")
        .to_owned();
    let mut outcome = import_text(&text, &name, path.parent(), progress)?;
    outcome.scene.source.file_size = bytes.len() as u64;
    Ok(outcome)
}
