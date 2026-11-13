//! The photo renderer as the source of movie frames: each frame places the camera at the animation pose and sets
//! the time of day and the cloud cover from the animated channels.

use super::frame_source::{FrameRequest, FrameSource};
use crate::features::environment::models::EnvironmentSettings;
use crate::features::movie::models::{CHANNEL_CLOUDS, CHANNEL_TIME_OF_DAY};
use crate::features::rendering::engine::{
    render_prepared, RenderCamera, RenderControl, RenderError, RenderScene, World,
};
use crate::features::rendering::logic::inputs::RenderInputs;
use crate::features::rendering::models::RayTracingQuality;
use image::RgbaImage;
use std::sync::atomic::AtomicBool;
use std::sync::Arc;

/// Renders frames of an animation with the CPU path tracer. The scene is prepared once, on the first frame.
pub struct PhotoFrameSource {
    world: Arc<World>,
    scene: Option<RenderScene>,
    inputs: RenderInputs,
    quality: RayTracingQuality,
    cancel: Option<Arc<AtomicBool>>,
}

impl PhotoFrameSource {
    /// A source that renders `world` with the project's `inputs` at `quality`.
    pub fn new(world: Arc<World>, inputs: RenderInputs, quality: RayTracingQuality) -> Self {
        Self {
            world,
            scene: None,
            inputs,
            quality,
            cancel: None,
        }
    }

    /// The environment of a frame: the project's, with the animated channels applied.
    pub fn environment_at(&self, request: &FrameRequest) -> EnvironmentSettings {
        let mut env = self.inputs.environment;
        for (name, value) in &request.channels {
            let value = *value as f32;
            if name.starts_with(CHANNEL_TIME_OF_DAY) {
                env.hour = value.rem_euclid(24.0);
            } else if name.starts_with(CHANNEL_CLOUDS) {
                env.sky.clouds.coverage = value.clamp(0.0, 1.0);
            }
        }
        env.sky.time_seconds = request.time as f32;
        env
    }
}

impl FrameSource for PhotoFrameSource {
    fn render(&mut self, request: &FrameRequest) -> Result<RgbaImage, String> {
        if request.width == 0 || request.height == 0 {
            return Err("empty frame size".into());
        }
        let camera = RenderCamera::from_pose(&request.pose, request.width, request.height);
        let mut render = self.inputs.request(camera, None, false, self.quality);
        render.environment = self.environment_at(request);
        render.seed = request.index.wrapping_add(1);
        let control = match &self.cancel {
            Some(flag) => RenderControl::linked(Arc::clone(flag)),
            None => RenderControl::new(),
        };
        let world = &self.world;
        let scene = self.scene.get_or_insert_with(|| RenderScene::new(world));
        render_prepared(scene, &render, &control, &|_| {}).map_err(|error| match error {
            RenderError::Cancelled => "cancelled".to_owned(),
            RenderError::EmptyImage => "empty frame size".to_owned(),
        })
    }

    fn set_cancel(&mut self, cancel: Arc<AtomicBool>) {
        self.cancel = Some(cancel);
    }
}
