//! The CPU photo renderer: a BVH path tracer with hard sun shadows, sky light, antialiasing and tiles.
//!
//! Entry points: [`render_scene`] (simple) and [`render_prepared`] (progress, cancellation, tile preview).

mod bvh;
mod framing;
mod geometry;
mod renderer;
mod sampling;
mod shading;
mod types;
mod vec3;

pub use framing::{frame_bounds, ViewPreset};
pub use geometry::{material_from_color, RenderScene};
pub use renderer::{render_prepared, render_scene, TILE};
pub use types::*;
