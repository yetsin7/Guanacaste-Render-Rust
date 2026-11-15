//! GPU side of the 3D view (wgpu through egui's paint callbacks). Everything that needs a device lives here.

pub mod bench;
pub mod callback;
pub mod globals;
pub mod gpu;
mod pipelines;
pub mod upload;
