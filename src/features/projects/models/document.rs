//! The editable part of a project, stored as JSON inside a `.gr` container (`GRRPROJ1`). Heavy geometry lives in binary
//! blocks (see `services/container.rs`); this document only keeps the metadata of each mesh.

use serde::{Deserialize, Serialize};

/// Schema version written by this application.
pub const SCHEMA_VERSION: u32 = 1;

/// Name, dates and writer of the document.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Meta {
    /// Project name.
    pub name: String,
    /// Creation time, seconds since the Unix epoch.
    pub created: u64,
    /// Last save time, seconds since the Unix epoch.
    pub modified: u64,
    /// Version of the application that wrote the file.
    pub app_version: String,
}

impl Default for Meta {
    fn default() -> Self {
        Self {
            name: "Sin título".into(),
            created: 0,
            modified: 0,
            app_version: env!("CARGO_PKG_VERSION").into(),
        }
    }
}

/// Where an imported project came from.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ImportOrigin {
    /// Source format name (`IFC`, `OBJ`, `glTF`, `FBX`, `Guanacaste 3D`).
    pub format: String,
    /// Schema or version text of the source file.
    pub version: String,
    /// Authoring application named by the file.
    pub application: String,
    /// Original file name (never the full path).
    pub file_name: String,
    /// Original file size in bytes.
    pub file_size: u64,
    /// Length unit declared by the source.
    pub length_unit: String,
    /// Meters per source unit.
    pub unit_scale: f64,
    /// Import time, seconds since the Unix epoch.
    pub imported: u64,
}

/// A hierarchy node.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NodeDoc {
    /// Display name.
    pub name: String,
    /// Source class (`IFCWALL`...).
    pub class: String,
    /// `project`, `site`, `building`, `level`, `element` or `group`.
    pub kind: String,
    /// Element type name.
    #[serde(default)]
    pub type_name: Option<String>,
    /// Parent node index.
    #[serde(default)]
    pub parent: Option<u32>,
    /// Indices into [`ProjectDocument::meshes`].
    #[serde(default)]
    pub meshes: Vec<u32>,
    /// Index into [`ProjectDocument::levels`].
    #[serde(default)]
    pub level: Option<u32>,
}

/// A building level.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LevelDoc {
    /// Name.
    pub name: String,
    /// Elevation in meters.
    pub elevation: f64,
    /// Node index of the level.
    pub node: u32,
}

/// A material.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MaterialDoc {
    /// Name.
    pub name: String,
    /// RGBA 0..=1.
    pub color: [f32; 4],
}

/// Metadata of a mesh whose vertices and indices are in a binary block.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MeshMeta {
    /// Material index.
    #[serde(default)]
    pub material: Option<u32>,
    /// Vertex count (checked against the block).
    pub vertices: u32,
    /// Index count (checked against the block).
    pub indices: u32,
}

/// A saved camera.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CameraDoc {
    /// Name.
    pub name: String,
    /// Eye position in meters.
    pub position: [f64; 3],
    /// Target position in meters.
    pub target: [f64; 3],
    /// Focal length in millimeters.
    pub focal_mm: f32,
    /// Aperture f-number.
    pub aperture: f32,
}

/// A timeline keyframe.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct KeyframeDoc {
    /// Time in seconds.
    pub time: f32,
    /// Camera index.
    pub camera: u32,
}

/// The animation timeline.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct TimelineDoc {
    /// Duration in seconds.
    pub duration: f32,
    /// Frames per second.
    pub fps: u32,
    /// Keyframes.
    pub keyframes: Vec<KeyframeDoc>,
}

impl Default for TimelineDoc {
    fn default() -> Self {
        Self {
            duration: 14.0,
            fps: 30,
            keyframes: Vec::new(),
        }
    }
}

/// Render settings.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct RenderSettingsDoc {
    /// Quality level name.
    pub quality: String,
    /// Output resolution.
    pub resolution: [u32; 2],
    /// Samples per pixel.
    pub samples: u32,
    /// Denoise switch.
    pub denoise: bool,
}

impl Default for RenderSettingsDoc {
    fn default() -> Self {
        Self {
            quality: "ultra".into(),
            resolution: [3840, 2160],
            samples: 1024,
            denoise: true,
        }
    }
}

/// Sun and sky.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct SunSkyDoc {
    /// Sun azimuth in degrees.
    pub azimuth: f32,
    /// Sun elevation in degrees.
    pub elevation: f32,
    /// Sky preset name.
    pub sky: String,
}

impl Default for SunSkyDoc {
    fn default() -> Self {
        Self {
            azimuth: 142.0,
            elevation: 35.0,
            sky: "cirrus".into(),
        }
    }
}

/// A placed vegetation instance.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct VegetationDoc {
    /// Library asset name.
    pub asset: String,
    /// Position in meters.
    pub position: [f64; 3],
    /// Yaw in degrees.
    pub rotation: f32,
    /// Uniform scale.
    pub scale: f32,
}

/// The whole JSON document of a project.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct ProjectDocument {
    /// Name and dates.
    pub meta: Meta,
    /// Import origin, when the project was imported.
    pub origin: Option<ImportOrigin>,
    /// World position subtracted from every vertex (meters).
    pub origin_shift: [f64; 3],
    /// Levels.
    pub levels: Vec<LevelDoc>,
    /// Materials.
    pub materials: Vec<MaterialDoc>,
    /// Hierarchy.
    pub nodes: Vec<NodeDoc>,
    /// Mesh metadata, one per binary block.
    pub meshes: Vec<MeshMeta>,
    /// Cameras.
    pub cameras: Vec<CameraDoc>,
    /// Timeline.
    pub timeline: TimelineDoc,
    /// Render settings.
    pub render: RenderSettingsDoc,
    /// Sun and sky.
    pub sun_sky: SunSkyDoc,
    /// Placed vegetation.
    pub vegetation: Vec<VegetationDoc>,
}
