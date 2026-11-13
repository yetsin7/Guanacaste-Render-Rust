//! Helpers shared by the simple mesh readers (STL, PLY, Collada, 3DS): bounded reads, vertex welding, texture
//! paths and scene assembly.

use crate::features::import_export::logic::geometry::V3;
use crate::features::import_export::models::{
    ImportError, ImportOptions, ImportOutcome, ImportReport, ImportedScene, Material, Mesh,
    NodeKind, SceneNode, SourceFormat, SourceInfo,
};
use crate::features::import_export::services::ifc::label_for;
use std::collections::HashMap;
use std::path::Path;

/// Largest file the in-memory readers accept.
pub const MAX_FILE_BYTES: u64 = 512 << 20;
/// Largest triangle index list a single mesh may grow to (guards memory on hostile or huge files).
pub const MAX_INDICES: usize = 90_000_000;

/// Error for a model that is too large for the importer.
pub fn too_big() -> ImportError {
    ImportError::Invalid(
        "El modelo es demasiado grande para importarlo (más de 30 millones de triángulos).".into(),
    )
}

/// Reads `path` completely unless it is larger than `max` bytes.
pub fn read_limited(path: &Path, max: u64) -> Result<Vec<u8>, ImportError> {
    let size = std::fs::metadata(path)?.len();
    if size > max {
        return Err(ImportError::Invalid(format!(
            "El archivo pesa {} MB y supera el límite de {} MB de este formato.",
            size >> 20,
            max >> 20
        )));
    }
    Ok(std::fs::read(path)?)
}

/// Applies the unit scale and, when `options.y_up`, the Y-up to Z-up conversion.
pub fn orient(point: V3, options: &ImportOptions) -> [f32; 3] {
    let s = options.unit_scale;
    let p = [point[0] * s, point[1] * s, point[2] * s];
    if options.y_up {
        [p[0] as f32, -p[2] as f32, p[1] as f32]
    } else {
        [p[0] as f32, p[1] as f32, p[2] as f32]
    }
}

/// Builds an indexed mesh from unindexed corners by welding equal position + UV pairs.
#[derive(Default)]
pub struct MeshBuilder {
    /// The mesh being built.
    pub mesh: Mesh,
    map: HashMap<[u32; 5], u32>,
}

impl MeshBuilder {
    /// Index of the vertex at `position` with `uv`, adding it when new.
    pub fn vertex(&mut self, position: [f32; 3], uv: Option<[f32; 2]>) -> u32 {
        let uv_bits = uv.map_or([0, 0], |t| [t[0].to_bits(), t[1].to_bits()]);
        let key = [
            position[0].to_bits(),
            position[1].to_bits(),
            position[2].to_bits(),
            uv_bits[0],
            uv_bits[1],
        ];
        if let Some(index) = self.map.get(&key) {
            return *index;
        }
        let index = self.mesh.positions.len() as u32;
        self.mesh.positions.push(position);
        if let Some(uv) = uv {
            // Keep `uvs` aligned with `positions` even if earlier vertices had none.
            self.mesh.uvs.resize(index as usize, [0.0, 0.0]);
            self.mesh.uvs.push(uv);
        }
        self.map.insert(key, index);
        index
    }

    /// Adds one triangle.
    pub fn triangle(&mut self, a: u32, b: u32, c: u32) {
        self.mesh.indices.extend_from_slice(&[a, b, c]);
    }

    /// Whether the mesh reached [`MAX_INDICES`].
    pub fn is_full(&self) -> bool {
        self.mesh.indices.len() >= MAX_INDICES
    }

    /// The finished mesh with its material.
    pub fn finish(mut self, material: Option<usize>) -> Mesh {
        if !self.mesh.uvs.is_empty() {
            self.mesh.uvs.resize(self.mesh.positions.len(), [0.0, 0.0]);
        }
        self.mesh.material = material;
        self.mesh
    }
}

/// The texture file name as the model stores it, made absolute when it exists next to the model.
pub fn texture_path(folder: Option<&Path>, name: &str) -> String {
    let clean = name
        .trim()
        .trim_start_matches("file:///")
        .replace('\\', "/");
    match folder {
        Some(folder) if !clean.is_empty() => {
            let joined = folder.join(&clean);
            if joined.is_file() {
                joined.display().to_string()
            } else {
                clean
            }
        }
        _ => clean,
    }
}

/// A scene node with no children or meshes yet.
pub fn node(name: &str, class: &str, kind: NodeKind, parent: Option<usize>) -> SceneNode {
    SceneNode {
        name: name.to_owned(),
        class: class.to_owned(),
        kind,
        type_name: None,
        parent,
        children: Vec::new(),
        meshes: Vec::new(),
        level: None,
    }
}

/// Source description for a mesh format.
pub fn source_info(
    format: SourceFormat,
    version: &str,
    application: &str,
    scale: f64,
    size: u64,
) -> SourceInfo {
    SourceInfo {
        format,
        version: version.to_owned(),
        application: application.to_owned(),
        length_unit: label_for(scale),
        unit_scale: scale,
        file_size: size,
    }
}

/// A flat scene: one project root with one element per `(name, mesh)` part.
pub fn flat_scene(
    source: SourceInfo,
    class: &str,
    root_name: &str,
    parts: Vec<(String, Mesh)>,
    materials: Vec<Material>,
    report: ImportReport,
    started: std::time::Instant,
) -> Result<ImportOutcome, ImportError> {
    let mut nodes = vec![node(root_name, "", NodeKind::Project, None)];
    let mut meshes = Vec::new();
    for (name, mesh) in parts.into_iter().filter(|(_, m)| !m.indices.is_empty()) {
        meshes.push(mesh);
        let mut element = node(&name, class, NodeKind::Element, Some(0));
        element.meshes.push(meshes.len() - 1);
        nodes.push(element);
        let index = nodes.len() - 1;
        nodes[0].children.push(index);
    }
    finish(source, nodes, meshes, materials, report, started)
}

/// Completes the report counters and rejects scenes without a single triangle.
pub fn finish(
    source: SourceInfo,
    nodes: Vec<SceneNode>,
    meshes: Vec<Mesh>,
    materials: Vec<Material>,
    mut report: ImportReport,
    started: std::time::Instant,
) -> Result<ImportOutcome, ImportError> {
    if meshes.iter().all(|mesh| mesh.indices.len() < 3) {
        return Err(ImportError::Invalid(
            "El archivo no contiene triángulos que mostrar (¿está vacío o solo tiene líneas y puntos?)."
                .into(),
        ));
    }
    let elements = nodes.iter().filter(|n| n.kind == NodeKind::Element);
    report.elements_total = elements.clone().count();
    report.elements_with_geometry = elements.filter(|n| !n.meshes.is_empty()).count();
    report.duration_ms = started.elapsed().as_millis();
    Ok(ImportOutcome {
        scene: ImportedScene {
            source,
            nodes,
            meshes,
            materials,
            levels: Vec::new(),
            origin_shift: [0.0; 3],
        },
        report,
    })
}
