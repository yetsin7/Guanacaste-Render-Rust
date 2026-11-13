//! The CPU photo renderer: a BVH path tracer with the sun, sky, fog, artificial lights, PBR textured materials,
//! antialiasing and tiles, followed by denoising, exposure, depth of field and the camera filters.
//!
//! Entry points: [`render_scene`] (simple) and [`render_prepared`] (progress, cancellation, tile preview).

mod atmosphere;
mod buffers;
mod bvh;
mod camera;
mod denoise;
mod framing;
mod geometry;
mod lights;
mod parallel;
mod post;
mod renderer;
mod sampling;
mod shading;
mod surface;
mod types;
mod vec3;
mod world;

pub use camera::saved_photo;
pub use framing::{frame_bounds, ViewPreset};
pub use geometry::RenderScene;
pub use renderer::{render_prepared, render_scene, TILE};
pub use types::*;
pub use world::World;
