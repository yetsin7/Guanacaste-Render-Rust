//! Render use cases: turns the open project into queue jobs and movie frame sources. The hub owns the queue and
//! remembers the world it built so several photos of the same project share it (and its acceleration structure).

use crate::commands::workspace::Workspace;
use crate::features::projects::models::Project;
use crate::features::rendering::engine::{ViewPreset, World};
use crate::features::rendering::logic::inputs::{CameraChoice, RenderInputs};
use crate::features::rendering::logic::plan::{build_spec, JobPlan};
use crate::features::rendering::logic::world::build_world;
use crate::features::rendering::models::{RayTracingQuality, QUALITIES};
use crate::features::rendering::services::frame_source::FrameSource;
use crate::features::rendering::services::movie_source::PhotoFrameSource;
use crate::features::rendering::services::RenderQueue;
use crate::features::viewport::models::Camera;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::sync::Arc;

/// Why nothing was added to the queue; the text is a localization key.
pub const NO_SCENE: &str = "render.no_scene";

/// Identity of the project content a cached world was built from.
type SceneKey = ([u8; 16], u64, usize, u64);

/// A hash of the parts of the document that change the world without changing its meshes.
fn content_hash(project: &Project) -> u64 {
    let document = &project.document;
    let mut hasher = DefaultHasher::new();
    for part in [
        serde_json::to_vec(&document.materials),
        serde_json::to_vec(&document.objects),
        serde_json::to_vec(&document.landscape),
        serde_json::to_vec(&document.lighting.emissive),
    ] {
        part.unwrap_or_default().hash(&mut hasher);
    }
    hasher.finish()
}

/// The render queue plus the world cache.
#[derive(Default)]
pub struct RenderHub {
    /// The background queue.
    pub queue: RenderQueue,
    cache: Option<(SceneKey, Arc<World>)>,
}

impl RenderHub {
    /// The open project as a world (model, ground, landscape and objects), built once per project state; `None`
    /// without geometry.
    pub fn world(&mut self, workspace: &Workspace) -> Option<Arc<World>> {
        let project = workspace
            .project
            .as_ref()
            .filter(|p| !p.meshes.is_empty())?;
        let key = (
            project.id,
            project.document.meta.modified,
            project.meshes.len(),
            content_hash(project),
        );
        if let Some((cached, world)) = &self.cache {
            if *cached == key {
                return Some(Arc::clone(world));
            }
        }
        let world = Arc::new(build_world(project));
        self.cache = Some((key, Arc::clone(&world)));
        Some(world)
    }

    fn queue_plans(
        &mut self,
        workspace: &Workspace,
        plans: &[JobPlan],
    ) -> Result<usize, &'static str> {
        let world = self.world(workspace).ok_or(NO_SCENE)?;
        let project = workspace.project.as_ref().ok_or(NO_SCENE)?;
        let inputs = RenderInputs::from_document(&project.document);
        let mut queued = 0;
        for plan in plans {
            let spec = build_spec(&world, &inputs, plan, plans.len() > 1).ok_or(NO_SCENE)?;
            self.queue.enqueue(spec);
            queued += 1;
        }
        Ok(queued)
    }

    /// Queues one photo per view of `views` with the settings of `plan`; returns how many were queued, or the
    /// localization key of the reason nothing was. A single view uses the plan's own camera (so a saved camera
    /// chosen in the studio is honored); several views add the project's saved cameras to the presets.
    pub fn enqueue(
        &mut self,
        workspace: &Workspace,
        plan: &JobPlan,
        views: &[ViewPreset],
    ) -> Result<usize, &'static str> {
        let mut plans: Vec<JobPlan> = Vec::new();
        if views.len() == 1 {
            plans.push(JobPlan {
                view: views[0],
                ..plan.clone()
            });
        } else {
            for view in views {
                plans.push(JobPlan {
                    view: *view,
                    camera: CameraChoice::Preset,
                    ..plan.clone()
                });
            }
            let saved = workspace.project.as_ref().map_or(0, |p| p.document.cameras.len());
            for index in 0..saved {
                plans.push(JobPlan {
                    camera: CameraChoice::Saved(index),
                    ..plan.clone()
                });
            }
        }
        self.queue_plans(workspace, &plans)
    }

    /// Queues one photo per saved camera of the project.
    pub fn enqueue_saved_cameras(
        &mut self,
        workspace: &Workspace,
        plan: &JobPlan,
    ) -> Result<usize, &'static str> {
        let count = workspace.project.as_ref().map_or(0, |p| p.document.cameras.len());
        let plans: Vec<JobPlan> = (0..count)
            .map(|index| JobPlan {
                camera: CameraChoice::Saved(index),
                ..plan.clone()
            })
            .collect();
        self.queue_plans(workspace, &plans)
    }

    /// Queues a photo of the 3D view's camera.
    pub fn enqueue_viewport(
        &mut self,
        workspace: &Workspace,
        plan: &JobPlan,
        camera: &Camera,
    ) -> Result<usize, &'static str> {
        let plan = JobPlan {
            camera: CameraChoice::Viewport(*camera),
            ..plan.clone()
        };
        self.queue_plans(workspace, &[plan])
    }

    /// The frame source that renders the animation of the open project with the photo renderer; `None` without
    /// geometry. `quality` is an index into the render qualities.
    pub fn movie_source(
        &mut self,
        workspace: &Workspace,
        quality: usize,
    ) -> Option<Box<dyn FrameSource>> {
        let world = self.world(workspace)?;
        let inputs = RenderInputs::from_document(&workspace.project.as_ref()?.document);
        let quality: RayTracingQuality = QUALITIES[quality.min(QUALITIES.len() - 1)];
        Some(Box::new(PhotoFrameSource::new(world, inputs, quality)))
    }
}
