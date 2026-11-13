//! Render use cases: turns the open project into queue jobs. The hub owns the queue and remembers the scene it
//! converted so several photos of the same project share it (and its acceleration structure).

use crate::commands::workspace::Workspace;
use crate::features::import_export::models::ImportedScene;
use crate::features::projects::logic::convert;
use crate::features::rendering::engine::ViewPreset;
use crate::features::rendering::logic::plan::{build_spec, JobPlan};
use crate::features::rendering::services::RenderQueue;
use std::sync::Arc;

/// Why nothing was added to the queue; the text is a localization key.
pub const NO_SCENE: &str = "render.no_scene";

/// Identity of the project content a cached scene was built from.
type SceneKey = ([u8; 16], u64, usize);

/// The render queue plus the scene cache.
#[derive(Default)]
pub struct RenderHub {
    /// The background queue.
    pub queue: RenderQueue,
    cache: Option<(SceneKey, Arc<ImportedScene>)>,
}

impl RenderHub {
    /// The open project as a scene, converted once per project state; `None` without geometry.
    pub fn scene(&mut self, workspace: &Workspace) -> Option<Arc<ImportedScene>> {
        let project = workspace
            .project
            .as_ref()
            .filter(|p| !p.meshes.is_empty())?;
        let key = (
            project.id,
            project.document.meta.modified,
            project.meshes.len(),
        );
        if let Some((cached, scene)) = &self.cache {
            if *cached == key {
                return Some(Arc::clone(scene));
            }
        }
        let scene = Arc::new(convert::to_scene(&project.document, &project.meshes, 0));
        self.cache = Some((key, Arc::clone(&scene)));
        Some(scene)
    }

    /// Queues one photo per view of `views` with the settings of `plan`; returns how many were queued, or the
    /// localization key of the reason nothing was.
    pub fn enqueue(
        &mut self,
        workspace: &Workspace,
        plan: &JobPlan,
        views: &[ViewPreset],
    ) -> Result<usize, &'static str> {
        let scene = self.scene(workspace).ok_or(NO_SCENE)?;
        let mut queued = 0;
        for view in views {
            let plan = JobPlan {
                view: *view,
                ..plan.clone()
            };
            let spec = build_spec(&scene, &plan, views.len() > 1).ok_or(NO_SCENE)?;
            self.queue.enqueue(spec);
            queued += 1;
        }
        Ok(queued)
    }
}
