//! The persisted landscape (terrain, water, vegetation) and the editing state around it.

use super::terrain::{SoilLayer, Terrain};
use super::tools::{Brush, FbmSettings, ScatterSettings, Tool};
use super::water::WaterBody;
use serde::{Deserialize, Serialize};

/// Most vegetation instances a landscape may hold.
pub const MAX_INSTANCES: usize = 50_000;

/// A placed plant.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct VegetationInstance {
    /// Library asset name.
    pub asset: String,
    /// Position in world meters (Z up), resting on the terrain.
    pub position: [f32; 3],
    /// Rotation about Z, in degrees.
    pub rotation_deg: f32,
    /// Uniform scale.
    pub scale: f32,
}

impl Default for VegetationInstance {
    fn default() -> Self {
        Self {
            asset: String::new(),
            position: [0.0; 3],
            rotation_deg: 0.0,
            scale: 1.0,
        }
    }
}

/// Everything of the landscape saved in a project.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Landscape {
    /// Height grid and soil layers.
    pub terrain: Terrain,
    /// Oceans and water planes.
    pub waters: Vec<WaterBody>,
    /// Placed plants.
    pub vegetation: Vec<VegetationInstance>,
    /// Whether the ground is drawn (the base ground can be hidden; its data is kept).
    pub visible: bool,
}

impl Default for Landscape {
    /// The base ground every project starts with: flat, grass-covered and 200 m wide at Z = 0.
    fn default() -> Self {
        Self {
            terrain: Terrain::flat(BASE_COLUMNS, BASE_COLUMNS, BASE_SPACING),
            waters: Vec::new(),
            vegetation: Vec::new(),
            visible: true,
        }
    }
}

/// Vertices per side of the base ground.
pub const BASE_COLUMNS: usize = 101;
/// Distance between the vertices of the base ground, in meters (so it is 200 m wide).
pub const BASE_SPACING: f32 = 2.0;

impl Landscape {
    /// Makes data read from a file safe to use; returns `true` when something had to be changed.
    pub fn repair(&mut self) -> bool {
        let mut changed = self.terrain.repair();
        for water in &mut self.waters {
            let before = water.clone();
            water.repair();
            changed |= *water != before;
        }
        let before = self.vegetation.len();
        self.vegetation.truncate(MAX_INSTANCES);
        self.vegetation.retain(|plant| {
            plant.position.iter().all(|v| v.is_finite())
                && plant.rotation_deg.is_finite()
                && plant.scale.is_finite()
                && plant.scale > 0.0
        });
        changed | (self.vegetation.len() != before)
    }
}

/// A message for the user about the last landscape action.
#[derive(Debug, Clone, PartialEq)]
pub enum Status {
    /// Something worked; the text is already localized.
    Info(String),
    /// Something failed; the text is already localized.
    Error(String),
}

/// The landscape plus the tool settings of the panel. Only [`LandscapeState::landscape`] is saved.
#[derive(Debug, Clone)]
pub struct LandscapeState {
    /// The edited landscape.
    pub landscape: Landscape,
    /// Active tool.
    pub tool: Tool,
    /// Brush settings.
    pub brush: Brush,
    /// Layer painted by [`Tool::Paint`].
    pub paint_layer: SoilLayer,
    /// Procedural generation settings.
    pub generator: FbmSettings,
    /// Vegetation scattering settings.
    pub scatter: ScatterSettings,
    /// Heights (meters) mapped to black and white when importing a heightmap.
    pub heightmap_range: [f32; 2],
    /// Water body edited in the panel.
    pub selected_water: Option<usize>,
    /// Increases on every change so views know when to rebuild meshes.
    pub revision: u64,
    /// Number of strokes applied, mixed into seeds so each dab differs reproducibly.
    pub strokes: u64,
    /// Result of the last action.
    pub status: Option<Status>,
    /// Vegetation placement carried between dabs (fractional plants).
    pub scatter_carry: f32,
}

impl Default for LandscapeState {
    fn default() -> Self {
        Self::with_landscape(Landscape::default())
    }
}

impl LandscapeState {
    /// A state editing `landscape` with default tools.
    pub fn with_landscape(landscape: Landscape) -> Self {
        Self {
            landscape,
            tool: Tool::default(),
            brush: Brush::default(),
            paint_layer: SoilLayer::default(),
            generator: FbmSettings::default(),
            scatter: ScatterSettings::default(),
            heightmap_range: [0.0, 40.0],
            selected_water: None,
            revision: 0,
            strokes: 0,
            status: None,
            scatter_carry: 0.0,
        }
    }

    /// Records a change.
    pub fn touch(&mut self) {
        self.revision = self.revision.wrapping_add(1);
    }
}
