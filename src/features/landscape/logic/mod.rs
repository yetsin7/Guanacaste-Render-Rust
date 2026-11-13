//! Landscape rules: noise, sampling, sculpting, generation, heightmaps, meshes, water and scattering.

pub mod actions;
pub mod generate;
pub mod heightmap;
pub mod mesh;
pub mod noise;
pub mod persist;
pub mod sample;
pub mod scatter;
pub mod sculpt;
pub mod water;

pub use mesh::{terrain_layer_meshes, terrain_mesh};
pub use sample::{height_at, slope_deg_at};
pub use water::{water_mesh, wave_height};
