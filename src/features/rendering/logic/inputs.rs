//! What a project contributes to a render besides geometry: environment, lights, filters and saved cameras.

use crate::features::camera::models::camera::PhotoCamera;
use crate::features::camera::models::stack::FilterStack;
use crate::features::environment::models::EnvironmentSettings;
use crate::features::lighting::logic::active_lights;
use crate::features::lighting::models::Light;
use crate::features::projects::models::{CameraDoc, ProjectDocument};
use crate::features::rendering::engine::{
    frame_facing, saved_photo, PostSettings, RenderCamera, RenderRequest, ViewPreset, World,
};
use crate::features::rendering::models::RayTracingQuality;
use crate::features::viewport::models::Camera;
use std::sync::Arc;

/// Where the camera of a render comes from.
#[derive(Debug, Clone, PartialEq, Default)]
pub enum CameraChoice {
    /// A ready-made view around the model (the plan's `view`).
    #[default]
    Preset,
    /// A saved camera of the project, by index.
    Saved(usize),
    /// The camera of the 3D view.
    Viewport(Camera),
    /// A 360 x 180 degree panorama from the 3D view's eye, with this many pixels across.
    Panorama(Camera, u32),
    /// A photographic camera standing at `position`, looking at `target`.
    Photo(PhotoCamera, [f32; 3], [f32; 3]),
}

/// The environment, lights, filters and cameras of a project, ready to be copied into render requests.
#[derive(Debug, Clone, PartialEq)]
pub struct RenderInputs {
    /// Date, time, place, sky and weather.
    pub environment: EnvironmentSettings,
    /// The lights that are switched on, with groups and dimmers applied.
    pub lights: Arc<Vec<Light>>,
    /// Filters of the project.
    pub filters: FilterStack,
    /// Remove noise from the pictures.
    pub denoise: bool,
    /// Saved cameras.
    pub cameras: Vec<CameraDoc>,
}

impl Default for RenderInputs {
    fn default() -> Self {
        Self {
            environment: EnvironmentSettings::default(),
            lights: Arc::new(Vec::new()),
            filters: FilterStack::new(),
            denoise: true,
            cameras: Vec::new(),
        }
    }
}

impl RenderInputs {
    /// Reads the render inputs of a project document; documents without environment or filters (older files)
    /// get the defaults.
    pub fn from_document(document: &ProjectDocument) -> Self {
        Self {
            environment: document.environment.unwrap_or_default(),
            lights: Arc::new(active_lights(&document.lighting)),
            filters: document.filters.clone(),
            denoise: document.render.denoise,
            cameras: document.cameras.clone(),
        }
    }

    /// The camera for `choice` at `width` x `height` pixels, with the photographic camera behind it when there is
    /// one. A preset frames the model of `world` from `preset`.
    pub fn camera(
        &self,
        world: &World,
        choice: &CameraChoice,
        preset: ViewPreset,
        size: (u32, u32),
    ) -> Option<(RenderCamera, Option<PhotoCamera>)> {
        let (width, height) = size;
        match choice {
            CameraChoice::Preset => {
                let bounds = world.focus.or_else(|| world.scene.bounds())?;
                if preset == ViewPreset::Interior {
                    if let Some(camera) = world.interior_camera(width, height) {
                        return Some((camera, None));
                    }
                }
                Some((
                    frame_facing(&bounds, world.facade, preset, width, height),
                    None,
                ))
            }
            CameraChoice::Saved(index) => {
                let saved = self.cameras.get(*index)?;
                Some((
                    RenderCamera::from_saved(saved, width, height),
                    Some(saved_photo(saved)),
                ))
            }
            CameraChoice::Viewport(camera) => {
                Some((RenderCamera::from_viewport(camera, width, height), None))
            }
            CameraChoice::Panorama(camera, width) => {
                Some((RenderCamera::panorama(camera, *width), None))
            }
            CameraChoice::Photo(photo, position, target) => {
                let mut camera = RenderCamera::from_photo(photo, *position, *target, width);
                camera.height = height;
                Some((camera, Some(*photo)))
            }
        }
    }

    /// A request for `camera` with these inputs. `photo` and `depth_of_field` come from the camera choice.
    pub fn request(
        &self,
        camera: RenderCamera,
        photo: Option<PhotoCamera>,
        depth_of_field: bool,
        quality: RayTracingQuality,
    ) -> RenderRequest {
        let mut request = RenderRequest::new(camera, quality);
        request.environment = self.environment;
        request.lights = Arc::clone(&self.lights);
        request.post = PostSettings {
            photo,
            depth_of_field: depth_of_field && photo.is_some(),
            filters: self.filters.clone(),
            denoise: self.denoise,
            auto_exposure: false,
        };
        request
    }
}
