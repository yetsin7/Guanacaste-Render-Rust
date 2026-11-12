//! The common imported scene: hierarchy, meshes and materials in meters, Z up.

use super::options::SourceFormat;

/// Where the scene came from.
#[derive(Debug, Clone, PartialEq)]
pub struct SourceInfo {
    /// File format.
    pub format: SourceFormat,
    /// Schema or version text, e.g. `IFC2X3` or `FBX 7.4`.
    pub version: String,
    /// Authoring application when the file says so.
    pub application: String,
    /// Length unit declared by the file, as text (`mm`, `ft`, `m`...).
    pub length_unit: String,
    /// Meters per file length unit.
    pub unit_scale: f64,
    /// File size in bytes.
    pub file_size: u64,
}

/// Role of a node in the hierarchy.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NodeKind {
    /// Project root.
    Project,
    /// Site.
    Site,
    /// Building.
    Building,
    /// Building level (storey).
    Level,
    /// A building element or object with geometry.
    Element,
    /// A plain grouping node.
    Group,
}

/// A node of the scene hierarchy.
#[derive(Debug, Clone, PartialEq)]
pub struct SceneNode {
    /// Display name.
    pub name: String,
    /// Source entity type (`IFCWALL`...) or empty.
    pub class: String,
    /// Role.
    pub kind: NodeKind,
    /// Element type name (e.g. a Revit family type) when known.
    pub type_name: Option<String>,
    /// Parent node index.
    pub parent: Option<usize>,
    /// Child node indices.
    pub children: Vec<usize>,
    /// Indices into [`ImportedScene::meshes`].
    pub meshes: Vec<usize>,
    /// Index into [`ImportedScene::levels`] of the level containing the node.
    pub level: Option<usize>,
}

/// An indexed triangle mesh in meters, relative to the scene origin.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Mesh {
    /// Vertex positions.
    pub positions: Vec<[f32; 3]>,
    /// Triangle indices, three per triangle.
    pub indices: Vec<u32>,
    /// Material index.
    pub material: Option<usize>,
}

/// A surface material.
#[derive(Debug, Clone, PartialEq)]
pub struct Material {
    /// Name.
    pub name: String,
    /// RGBA, each 0..=1.
    pub color: [f32; 4],
}

/// A building level.
#[derive(Debug, Clone, PartialEq)]
pub struct Level {
    /// Name.
    pub name: String,
    /// Elevation in meters.
    pub elevation: f64,
    /// Node index.
    pub node: usize,
}

/// Axis-aligned bounds in meters.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Bounds {
    /// Minimum corner.
    pub min: [f32; 3],
    /// Maximum corner.
    pub max: [f32; 3],
}

/// A model imported from any supported format.
#[derive(Debug, Clone, PartialEq)]
pub struct ImportedScene {
    /// Origin of the data.
    pub source: SourceInfo,
    /// Hierarchy; node 0 is the root.
    pub nodes: Vec<SceneNode>,
    /// All meshes.
    pub meshes: Vec<Mesh>,
    /// All materials.
    pub materials: Vec<Material>,
    /// Levels sorted by elevation.
    pub levels: Vec<Level>,
    /// World position (meters) subtracted from every vertex to keep `f32` precise.
    pub origin_shift: [f64; 3],
}

impl ImportedScene {
    /// Number of nodes that are elements.
    pub fn element_count(&self) -> usize {
        self.nodes
            .iter()
            .filter(|node| node.kind == NodeKind::Element)
            .count()
    }

    /// Total triangles.
    pub fn triangle_count(&self) -> usize {
        self.meshes.iter().map(|mesh| mesh.indices.len() / 3).sum()
    }

    /// Total vertices.
    pub fn vertex_count(&self) -> usize {
        self.meshes.iter().map(|mesh| mesh.positions.len()).sum()
    }

    /// Bounds of every vertex, or `None` for an empty scene.
    pub fn bounds(&self) -> Option<Bounds> {
        let mut points = self.meshes.iter().flat_map(|mesh| mesh.positions.iter());
        let first = *points.next()?;
        let mut bounds = Bounds {
            min: first,
            max: first,
        };
        for point in points {
            for axis in 0..3 {
                bounds.min[axis] = bounds.min[axis].min(point[axis]);
                bounds.max[axis] = bounds.max[axis].max(point[axis]);
            }
        }
        Some(bounds)
    }
}
