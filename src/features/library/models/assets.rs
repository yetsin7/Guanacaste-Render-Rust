//! Placeable assets: the procedural kinds generated in code, their categories, and the reference an instance keeps.

use serde::{Deserialize, Serialize};

/// Groups of the object library.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AssetCategory {
    /// Trees, bushes and palms.
    Vegetation,
    /// People.
    People,
    /// Vehicles.
    Vehicles,
    /// Street furniture: benches, lights, planters, fences.
    Furniture,
    /// Cube, cylinder, sphere and plane.
    Primitives,
    /// Models imported by the user.
    Imported,
}

impl AssetCategory {
    /// Categories in display order.
    pub const ALL: [AssetCategory; 6] = [
        AssetCategory::Vegetation,
        AssetCategory::People,
        AssetCategory::Vehicles,
        AssetCategory::Furniture,
        AssetCategory::Primitives,
        AssetCategory::Imported,
    ];

    /// Localization key of the category name.
    pub fn key(self) -> &'static str {
        match self {
            AssetCategory::Vegetation => "objects.cat_vegetation",
            AssetCategory::People => "objects.cat_people",
            AssetCategory::Vehicles => "objects.cat_vehicles",
            AssetCategory::Furniture => "objects.cat_furniture",
            AssetCategory::Primitives => "objects.cat_primitives",
            AssetCategory::Imported => "objects.cat_imported",
        }
    }
}

/// Assets generated in code; the seed of an [`AssetRef`] picks the variation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProceduralKind {
    /// Broadleaf tree.
    Tree,
    /// Conifer.
    Conifer,
    /// Bush.
    Bush,
    /// Palm.
    Palm,
    /// Standing person.
    Person,
    /// Car.
    Car,
    /// Park bench.
    Bench,
    /// Street light.
    StreetLight,
    /// Planter with a bush.
    Planter,
    /// Fence section.
    Fence,
    /// Cube.
    Cube,
    /// Cylinder.
    Cylinder,
    /// Sphere.
    Sphere,
    /// Plane.
    Plane,
}

impl ProceduralKind {
    /// Every kind in display order.
    pub const ALL: [ProceduralKind; 14] = [
        ProceduralKind::Tree,
        ProceduralKind::Conifer,
        ProceduralKind::Bush,
        ProceduralKind::Palm,
        ProceduralKind::Person,
        ProceduralKind::Car,
        ProceduralKind::Bench,
        ProceduralKind::StreetLight,
        ProceduralKind::Planter,
        ProceduralKind::Fence,
        ProceduralKind::Cube,
        ProceduralKind::Cylinder,
        ProceduralKind::Sphere,
        ProceduralKind::Plane,
    ];

    /// Category of the kind.
    pub fn category(self) -> AssetCategory {
        match self {
            ProceduralKind::Tree
            | ProceduralKind::Conifer
            | ProceduralKind::Bush
            | ProceduralKind::Palm => AssetCategory::Vegetation,
            ProceduralKind::Person => AssetCategory::People,
            ProceduralKind::Car => AssetCategory::Vehicles,
            ProceduralKind::Bench
            | ProceduralKind::StreetLight
            | ProceduralKind::Planter
            | ProceduralKind::Fence => AssetCategory::Furniture,
            ProceduralKind::Cube
            | ProceduralKind::Cylinder
            | ProceduralKind::Sphere
            | ProceduralKind::Plane => AssetCategory::Primitives,
        }
    }

    /// Localization key of the display name.
    pub fn key(self) -> &'static str {
        match self {
            ProceduralKind::Tree => "objects.asset_tree",
            ProceduralKind::Conifer => "objects.asset_conifer",
            ProceduralKind::Bush => "objects.asset_bush",
            ProceduralKind::Palm => "objects.asset_palm",
            ProceduralKind::Person => "objects.asset_person",
            ProceduralKind::Car => "objects.asset_car",
            ProceduralKind::Bench => "objects.asset_bench",
            ProceduralKind::StreetLight => "objects.asset_street_light",
            ProceduralKind::Planter => "objects.asset_planter",
            ProceduralKind::Fence => "objects.asset_fence",
            ProceduralKind::Cube => "objects.asset_cube",
            ProceduralKind::Cylinder => "objects.asset_cylinder",
            ProceduralKind::Sphere => "objects.asset_sphere",
            ProceduralKind::Plane => "objects.asset_plane",
        }
    }

    /// Whether the seed changes the shape (primitives ignore it).
    pub fn varies(self) -> bool {
        self.category() != AssetCategory::Primitives
    }
}

/// What a placed instance shows: a procedural kind with its variation seed, or a model imported by the user.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum AssetRef {
    /// A procedural asset.
    Procedural {
        /// Kind of asset.
        kind: ProceduralKind,
        /// Variation seed.
        seed: u64,
    },
    /// An imported model, by its id in the imported-asset list.
    Imported {
        /// Id of the imported asset.
        id: u64,
    },
}

impl AssetRef {
    /// Category of the referenced asset.
    pub fn category(self) -> AssetCategory {
        match self {
            AssetRef::Procedural { kind, .. } => kind.category(),
            AssetRef::Imported { .. } => AssetCategory::Imported,
        }
    }

    /// The same asset with another seed (imported assets do not vary).
    pub fn with_seed(self, seed: u64) -> Self {
        match self {
            AssetRef::Procedural { kind, .. } if kind.varies() => {
                AssetRef::Procedural { kind, seed }
            }
            AssetRef::Procedural { kind, .. } => AssetRef::Procedural { kind, seed: 0 },
            other => other,
        }
    }
}

/// A model the user imported to use as a library asset.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ImportedAssetDoc {
    /// Stable id referenced by [`AssetRef::Imported`].
    pub id: u64,
    /// Display name (the file stem).
    pub name: String,
    /// Path of the source file; the model is imported again from it when the project opens.
    pub source: String,
}
