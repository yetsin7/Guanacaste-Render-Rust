//! Material editor rules: sampling, procedural textures, presets, previews and project storage.

pub mod color;
pub mod convert;
pub mod derive;
pub mod editor;
pub mod legacy_catalog;
pub mod maps;
pub mod presets;
pub mod preview;
pub mod procedural;
pub mod project_io;
pub mod sample;
pub mod sampling;
pub mod sync;

use super::models::Material;

/// Restores every value of `material` to the defaults, keeping its name.
pub fn reset(material: &mut Material) {
    *material = Material::new(std::mem::take(&mut material.name));
}

/// Whether a material named `name` is shown under the filter text (case-insensitive substring).
pub fn matches_filter(name: &str, filter: &str) -> bool {
    let needle = filter.trim().to_lowercase();
    needle.is_empty() || name.to_lowercase().contains(&needle)
}

/// Built-in image maps shared with Guanacaste 3D.
pub mod bundled_textures;
