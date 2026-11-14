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
mod interior;
mod lights;
mod parallel;
mod portals;
mod post;
mod renderer;
mod sampling;
mod shading;
mod site;
#[cfg(test)]
mod site_tests;
mod split;
mod surface;
mod types;
mod uvmap;
mod vec3;
mod world;

pub use camera::saved_photo;
pub use framing::{frame_bounds, frame_facing, ViewPreset, SOUTH};
pub use geometry::RenderScene;
pub use interior::interior_camera;
pub use renderer::{render_prepared, render_scene, TILE};
pub use site::{measure as measure_site, Site};
pub use types::*;
pub use world::World;
