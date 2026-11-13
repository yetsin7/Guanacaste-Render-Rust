//! The 3D view: a real-time viewport (wgpu paint callback), camera navigation in the style of Lumion (fly with
//! `W`/`A`/`S`/`D`/`Q`/`E`, orbit, pan, zoom, frame the model), the backdrop and the overlay controls.
//!
//! Layers: `models` holds the public [`models::Camera`] and the input vocabulary; `logic` is pure and tested (math,
//! navigation, framing, scene buffers); `services` talks to the GPU; `ui` reads egui input and paints the view.

#![allow(clippy::chunks_exact_to_as_chunks)]

pub mod components;
pub mod logic;
pub mod models;
pub mod services;
pub mod ui;

#[cfg(test)]
mod tests;
