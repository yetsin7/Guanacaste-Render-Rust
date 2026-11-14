//! Realistic PBR materials for models that only carry flat colors (Revit IFC exports above all): the surface family
//! is guessed from the material name, the type name and the element class (Spanish and English words), and each
//! family gets roughness, texture maps and a tint that look like the real thing.
//!
//! The rules are plain functions of text: [`classify`] picks the family, [`derive`] builds the material and
//! [`is_hidden`] tells the surfaces that stay out of the picture by default (buried pipes, the soil embankment).

mod build;
mod classify;
mod enrich;
mod text;

pub use build::derive;
pub use classify::classify;
pub use enrich::{enrich_scene, needs_enrichment};

use crate::features::materials::models::Material;

/// The family of surface a material belongs to.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SurfaceKind {
    /// Rough exterior render in a light color.
    Stucco,
    /// Smooth interior plaster, gypsum board and ceilings.
    Plaster,
    /// Painted trim, frames and generic painted parts.
    Paint,
    /// Clay barrel roof tile.
    RoofTile,
    /// Timber with grain.
    Wood,
    /// Cut stone blocks.
    Stone,
    /// Brick with mortar.
    Brick,
    /// Cast concrete.
    Concrete,
    /// Dark brushed metal.
    Metal,
    /// Transparent glass.
    Glass,
    /// Glazed ceramic tile.
    Ceramic,
    /// Lawn.
    Grass,
    /// Woven fabric and carpet.
    Fabric,
    /// Plastic.
    Plastic,
    /// Soil and embankments: hidden by default so the lawn shows instead of a crater.
    Soil,
    /// Pipes, ducts and fully invisible styles: hidden by default.
    Hidden,
}

impl SurfaceKind {
    /// Localization key of the family's name.
    pub fn label_key(self) -> &'static str {
        match self {
            SurfaceKind::Stucco => "material.kind_stucco",
            SurfaceKind::Plaster => "material.kind_plaster",
            SurfaceKind::Paint => "material.kind_paint",
            SurfaceKind::RoofTile => "material.kind_roof_tile",
            SurfaceKind::Wood => "material.kind_wood",
            SurfaceKind::Stone => "material.kind_stone",
            SurfaceKind::Brick => "material.kind_brick",
            SurfaceKind::Concrete => "material.kind_concrete",
            SurfaceKind::Metal => "material.kind_metal",
            SurfaceKind::Glass => "material.kind_glass",
            SurfaceKind::Ceramic => "material.kind_ceramic",
            SurfaceKind::Grass => "material.kind_grass",
            SurfaceKind::Fabric => "material.kind_fabric",
            SurfaceKind::Plastic => "material.kind_plastic",
            SurfaceKind::Soil => "material.kind_soil",
            SurfaceKind::Hidden => "material.kind_hidden",
        }
    }

    /// Whether surfaces of this family are left out of the picture by default.
    pub fn is_hidden(self) -> bool {
        matches!(self, SurfaceKind::Soil | SurfaceKind::Hidden)
    }
}

/// What the import knows about a surface.
#[derive(Debug, Clone, Copy)]
pub struct SurfaceInfo<'a> {
    /// The material name.
    pub name: &'a str,
    /// The element's type name (a Revit family type), when known.
    pub type_name: Option<&'a str>,
    /// The element's name, when known.
    pub element: Option<&'a str>,
    /// The IFC class of the element (`IFCWALL`...), or empty.
    pub class: &'a str,
    /// The style color with alpha.
    pub color: [f32; 4],
}

/// Whether `material` was made invisible (the "hidden by default" families are fully transparent). The user can
/// show them again by lowering the transparency in the material editor.
pub fn is_hidden(material: &Material) -> bool {
    material.transparency >= 0.999 || material.base_color[3] <= 0.001
}

#[cfg(test)]
mod tests;
