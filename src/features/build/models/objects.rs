//! Placed scene objects: an asset reference plus a transform, the tool options of object placement, and the
//! serializable form stored in the `.gr` project document.

use crate::features::library::models::{AssetCategory, AssetRef, ImportedAssetDoc};
use serde::{Deserialize, Serialize};

fn unit_scale() -> [f32; 3] {
    [1.0; 3]
}

/// Position, rotation and scale of an instance. Meters in the project frame (the same frame as the model vertices),
/// Z up; rotation in degrees applied X, then Y, then Z.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Transform {
    /// Position in meters.
    pub position: [f64; 3],
    /// Euler rotation in degrees.
    #[serde(default)]
    pub rotation: [f32; 3],
    /// Scale per axis.
    #[serde(default = "unit_scale")]
    pub scale: [f32; 3],
}

impl Default for Transform {
    fn default() -> Self {
        Self {
            position: [0.0; 3],
            rotation: [0.0; 3],
            scale: unit_scale(),
        }
    }
}

impl Transform {
    /// A transform at `position` with no rotation and unit scale.
    pub fn at(position: [f64; 3]) -> Self {
        Self {
            position,
            ..Self::default()
        }
    }
}

/// A library asset placed in the scene.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PlacedObject {
    /// Unique id within the project.
    pub id: u64,
    /// The asset shown.
    pub asset: AssetRef,
    /// Where and how it is placed.
    #[serde(default)]
    pub transform: Transform,
}

/// Objects of a project as stored in the `.gr` document (`objects` field). Every field has a default so older and
/// newer files open.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct ObjectsDoc {
    /// Placed instances.
    pub instances: Vec<PlacedObject>,
    /// Models imported as library assets.
    pub imported_assets: Vec<ImportedAssetDoc>,
    /// Next free id for instances and imported assets.
    pub next_id: u64,
}

impl ObjectsDoc {
    /// Whether there is nothing to store (the field is then left out of the file).
    pub fn is_empty(&self) -> bool {
        self.instances.is_empty() && self.imported_assets.is_empty()
    }
}

/// What dragging in the view does to the selection.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum GizmoMode {
    /// Clicking places the active asset.
    #[default]
    Place,
    /// Dragging moves.
    Move,
    /// Dragging rotates around the vertical axis.
    Rotate,
    /// Dragging scales.
    Scale,
}

impl GizmoMode {
    /// Modes in display order.
    pub const ALL: [GizmoMode; 4] = [
        GizmoMode::Place,
        GizmoMode::Move,
        GizmoMode::Rotate,
        GizmoMode::Scale,
    ];

    /// Localization key of the mode name.
    pub fn key(self) -> &'static str {
        match self {
            GizmoMode::Place => "objects.mode_place",
            GizmoMode::Move => "objects.mode_move",
            GizmoMode::Rotate => "objects.mode_rotate",
            GizmoMode::Scale => "objects.mode_scale",
        }
    }
}

/// Axis restriction of a drag.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum AxisLock {
    /// Move over the horizontal plane; scale uniformly.
    #[default]
    Free,
    /// Only X.
    X,
    /// Only Y.
    Y,
    /// Only Z (vertical).
    Z,
}

/// How a coordinate is aligned across the selection.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AlignMode {
    /// To the smallest value.
    Min,
    /// To the average.
    Center,
    /// To the largest value.
    Max,
}

/// Options applied when instances are created.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PlacementOptions {
    /// Drop new and snapped instances onto the model surface (or the ground).
    pub snap_to_surface: bool,
    /// Random yaw for every new instance.
    pub random_rotation: bool,
    /// Random uniform scale variation, ± this fraction (0..=0.5).
    pub random_scale: f32,
    /// A new random seed per instance, so procedural vegetation never repeats.
    pub vary_seed: bool,
    /// Move snapping step in meters (0 = free).
    pub grid: f32,
    /// Rotation snapping step in degrees (0 = free).
    pub rotation_step: f32,
}

impl Default for PlacementOptions {
    fn default() -> Self {
        Self {
            snap_to_surface: true,
            random_rotation: true,
            random_scale: 0.1,
            vary_seed: true,
            grid: 0.0,
            rotation_step: 15.0,
        }
    }
}

/// Options of the mass tools: duplicate in an array and scatter around a point.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MassOptions {
    /// Copies made of each selected instance.
    pub copies: u32,
    /// Offset between consecutive copies, meters.
    pub offset: [f32; 3],
    /// Instances created by a scatter.
    pub scatter_count: u32,
    /// Scatter radius, meters.
    pub scatter_radius: f32,
}

impl Default for MassOptions {
    fn default() -> Self {
        Self {
            copies: 5,
            offset: [3.0, 0.0, 0.0],
            scatter_count: 20,
            scatter_radius: 15.0,
        }
    }
}

/// State of the asset browser of the objects panel.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct BrowserState {
    /// Search text.
    pub search: String,
    /// Category filter (`None` shows all).
    pub category: Option<AssetCategory>,
    /// Asset placed by the next click (with the seed below).
    pub active: Option<AssetRef>,
    /// Variation seed of the active procedural asset.
    pub seed: u64,
}
