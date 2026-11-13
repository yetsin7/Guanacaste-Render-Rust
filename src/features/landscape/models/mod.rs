//! Landscape entities: terrain grid, soil layers, water bodies, vegetation and tool settings.

pub mod landscape;
pub mod terrain;
pub mod tools;
pub mod water;

pub use landscape::{Landscape, LandscapeState, Status, VegetationInstance, MAX_INSTANCES};
pub use terrain::{LayerWeights, SoilLayer, Terrain, MAX_RESOLUTION, MIN_RESOLUTION};
pub use tools::{Brush, FbmSettings, ScatterSettings, Tool};
pub use water::{WaterBody, WaterKind};
