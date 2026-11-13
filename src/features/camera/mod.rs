//! Camera domain: the photographic camera model (lens, sensor, field of view, clip planes, depth of field,
//! exposure), the image filter chain and the camera studio panel.
//!
//! Public API for other domains:
//!
//! * [`models::camera::PhotoCamera`] with [`models::camera::Projection`] for a 3D view.
//! * [`models::image_buffer::ImageBuffer`] (linear-light RGBA `f32`, documented there) and
//!   [`models::stack::FilterStack`] to post-process rendered pictures: `stack.apply(&buffer)`.
//! * [`models::preset::FilterPreset`] for ready-made looks.
//! * [`components::studio::show`] with [`models::state::CameraStudioState`] for the interface.

// Pixel data is interleaved RGBA; chunking by 4 reads better than `as_chunks` and keeps older toolchains working.
#![allow(unknown_lints, clippy::chunks_exact_to_as_chunks)]

pub mod components;
pub mod logic;
pub mod models;

#[cfg(test)]
mod tests;
