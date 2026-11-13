//! Pure rules of the 3D view: vector math, camera motions and navigation, framing, saved cameras and the
//! conversion of imported meshes into GPU buffers. Nothing here touches egui or wgpu.

pub mod controller;
pub mod framing;
pub mod frustum;
pub mod grid;
pub mod math;
pub mod motion;
pub mod scene_buffers;
pub mod slots;

pub use controller::NavController;
pub use slots::CameraSlots;
