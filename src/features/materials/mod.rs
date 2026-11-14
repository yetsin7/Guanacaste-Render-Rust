//! Materials domain: the PBR material model, textures, presets and the material editor.
//!
//! Public API: [`Material`], [`MaterialLibrary`], [`TextureCache`] and [`sample`].

pub mod components;
pub mod logic;
pub mod models;
pub mod services;

pub use crate::services::texture_cache::TextureCache;
pub use logic::maps::MaterialMaps;
pub use logic::sample::{sample, surface, surface_with, Rgba, Surface};
pub use models::{Material, MaterialLibrary};

#[cfg(test)]
mod tests;
