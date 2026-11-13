//! Material entities and the material editor's UI state.

mod library;
mod material;
mod state;
mod texture;

pub use library::MaterialLibrary;
pub use material::{MapKind, Material, TextureMaps, UvTransform, Wrap};
pub use state::MaterialState;
pub use texture::{Procedural, ProceduralKind, TextureRef, TextureSource};
