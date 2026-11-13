//! Assets generated in code (no downloads): vegetation, people, vehicles, street furniture and primitives. Every
//! generator returns an [`ImportedScene`] in meters, Z up, resting on the ground at the origin.

mod builder;
mod primitives;
mod rng;
mod urban;
mod vegetation;

pub use builder::ShapeBuilder;
pub use rng::Rng;

use crate::features::import_export::models::ImportedScene;
use crate::features::library::models::ProceduralKind;

/// Generates the variation `seed` of `kind`.
pub fn generate(kind: ProceduralKind, seed: u64) -> ImportedScene {
    match kind {
        ProceduralKind::Tree => vegetation::tree(seed),
        ProceduralKind::Conifer => vegetation::conifer(seed),
        ProceduralKind::Bush => vegetation::bush(seed),
        ProceduralKind::Palm => vegetation::palm(seed),
        ProceduralKind::Person => urban::person(seed),
        ProceduralKind::Car => urban::car(seed),
        ProceduralKind::Bench => urban::bench(seed),
        ProceduralKind::StreetLight => urban::street_light(seed),
        ProceduralKind::Planter => urban::planter(seed),
        ProceduralKind::Fence => urban::fence(seed),
        ProceduralKind::Cube => primitives::cube(),
        ProceduralKind::Cylinder => primitives::cylinder(),
        ProceduralKind::Sphere => primitives::sphere(),
        ProceduralKind::Plane => primitives::plane(),
    }
}
