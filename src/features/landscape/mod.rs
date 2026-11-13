//! Landscape domain: heightmap terrain with brushes, soil layers, ocean and water planes, and vegetation
//! scattering. Everything is produced as triangle meshes in meters, Z up, compatible with `ImportedScene`.

pub mod components;
pub mod logic;
pub mod models;

pub use components::panel;
pub use logic::{height_at, terrain_mesh, water_mesh, wave_height};

#[cfg(test)]
mod tests;
