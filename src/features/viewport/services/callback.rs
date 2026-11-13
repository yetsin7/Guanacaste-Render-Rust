//! The egui paint callback of the 3D view: renders offscreen in `prepare` and copies the result in `paint`.

use super::globals::Globals;
use super::gpu::ViewportGpu;
use super::upload::SceneUpload;
use crate::features::viewport::models::ViewOptions;
use eframe::egui_wgpu::{self, CallbackResources, CallbackTrait, ScreenDescriptor};
use eframe::{egui, wgpu};
use std::sync::Arc;

/// One 3D view to draw this frame.
pub struct ViewportCallback {
    /// Per-frame shader values.
    pub globals: Globals,
    /// Size of the view in physical pixels.
    pub size_px: [u32; 2],
    /// The scene to show.
    pub scene: Arc<SceneUpload>,
    /// Helpers to draw.
    pub options: ViewOptions,
}

impl ViewportCallback {
    /// Wraps the callback as an egui shape covering `rect`.
    pub fn shape(self, rect: egui::Rect) -> egui::Shape {
        egui::Shape::Callback(egui_wgpu::Callback::new_paint_callback(rect, self))
    }
}

impl CallbackTrait for ViewportCallback {
    fn prepare(
        &self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        _screen: &ScreenDescriptor,
        encoder: &mut wgpu::CommandEncoder,
        resources: &mut CallbackResources,
    ) -> Vec<wgpu::CommandBuffer> {
        if let Some(gpu) = resources.get_mut::<ViewportGpu>() {
            gpu.prepare(device, queue, self.size_px, &self.globals, &self.scene);
            gpu.render(encoder, &self.globals.view_proj, self.options);
        }
        Vec::new()
    }

    fn paint(
        &self,
        _info: egui::PaintCallbackInfo,
        pass: &mut wgpu::RenderPass<'static>,
        resources: &CallbackResources,
    ) {
        if let Some(gpu) = resources.get::<ViewportGpu>() {
            gpu.blit(pass);
        }
    }
}
