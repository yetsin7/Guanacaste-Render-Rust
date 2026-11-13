//! State of the 3D view that outlives frames: the camera controller, saved cameras, the scene sent to the GPU and
//! the helper toggles. One instance serves every screen that shows the 3D view, so the camera is shared.

use crate::features::import_export::models::{Bounds, Mesh};
use crate::features::viewport::logic::scene_buffers;
use crate::features::viewport::logic::{CameraSlots, NavController};
use crate::features::viewport::models::{Camera, ViewOptions, ViewPreset};
use crate::features::viewport::services::upload::SceneUpload;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::sync::Arc;
use std::time::Instant;

/// The geometry to show, borrowed from whoever owns the open project.
pub struct SceneSource<'a> {
    /// Identifier of the project; a different id means a different scene (the camera is re-framed).
    pub id: [u8; 16],
    /// Meshes in meters, Z up.
    pub meshes: &'a [Mesh],
    /// RGBA (0..=1) per material index.
    pub material_colors: Vec<[f32; 4]>,
}

/// Cheap fingerprint of a scene, compared every frame to know when to rebuild the GPU buffers.
#[derive(Debug, Clone, PartialEq, Eq)]
struct SceneKey {
    id: [u8; 16],
    meshes: usize,
    vertices: usize,
    indices: usize,
    colors: u64,
}

impl SceneKey {
    fn of(source: &SceneSource) -> Self {
        let mut hasher = DefaultHasher::new();
        for color in &source.material_colors {
            color.map(f32::to_bits).hash(&mut hasher);
        }
        for mesh in source.meshes {
            mesh.material.hash(&mut hasher);
        }
        Self {
            id: source.id,
            meshes: source.meshes.len(),
            vertices: source.meshes.iter().map(|m| m.positions.len()).sum(),
            indices: source.meshes.iter().map(|m| m.indices.len()).sum(),
            colors: hasher.finish(),
        }
    }
}

/// What [`ViewportState::sync`] did.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SceneChange {
    /// The scene is the one already shown.
    Unchanged,
    /// The GPU buffers were rebuilt but the camera stays (same project, e.g. a material changed).
    Updated,
    /// A different scene with geometry is now shown and the camera will frame it.
    Loaded,
}

/// Numbers about the scene being shown.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct SceneStats {
    /// Triangles drawn.
    pub triangles: usize,
    /// Meshes that could not be drawn.
    pub skipped_meshes: usize,
}

/// A short message shown over the view (saved camera, empty slot...).
#[derive(Debug, Clone, PartialEq)]
pub struct Notice {
    /// Text in Spanish.
    pub text: String,
    /// When it was raised.
    pub at: Instant,
}

/// Everything the 3D view remembers.
pub struct ViewportState {
    /// Camera and navigation preferences.
    pub controller: NavController,
    /// Numbered saved cameras.
    pub slots: CameraSlots,
    /// Which helpers are drawn.
    pub options: ViewOptions,
    /// Whether the GPU resources were installed (false means no 3D can be drawn).
    pub gpu_available: bool,
    /// Message to show briefly.
    pub notice: Option<Notice>,
    scene: Arc<SceneUpload>,
    generation: u64,
    key: Option<SceneKey>,
    scene_id: Option<[u8; 16]>,
    bounds: Option<Bounds>,
    stats: SceneStats,
    needs_frame: bool,
    aspect: f32,
}

impl Default for ViewportState {
    fn default() -> Self {
        Self::new()
    }
}

impl ViewportState {
    /// An empty scene (just the grid) with the default camera.
    pub fn new() -> Self {
        let buffers = scene_buffers::build(&[], &[]);
        Self {
            controller: NavController::default(),
            slots: CameraSlots::default(),
            options: ViewOptions::default(),
            gpu_available: false,
            notice: None,
            scene: Arc::new(SceneUpload::new(1, buffers)),
            generation: 1,
            key: None,
            scene_id: None,
            bounds: None,
            stats: SceneStats::default(),
            needs_frame: false,
            aspect: 16.0 / 9.0,
        }
    }

    /// The camera now in use.
    pub fn camera(&self) -> &Camera {
        &self.controller.camera
    }

    /// Bounds of the geometry shown, `None` for an empty scene.
    pub fn bounds(&self) -> Option<&Bounds> {
        self.bounds.as_ref()
    }

    /// Width/height ratio of the view as last drawn.
    pub fn aspect(&self) -> f32 {
        self.aspect
    }

    /// Records the ratio of the view just drawn.
    pub(super) fn set_aspect(&mut self, aspect: f32) {
        self.aspect = aspect;
    }

    /// Numbers about the scene shown.
    pub fn stats(&self) -> SceneStats {
        self.stats
    }

    /// Whether there is geometry to look at.
    pub fn has_geometry(&self) -> bool {
        self.bounds.is_some()
    }

    /// The scene handed to paint callbacks.
    pub fn upload(&self) -> Arc<SceneUpload> {
        self.scene.clone()
    }

    /// Radius of the scene, for scaling the grid fade.
    pub fn scene_radius(&self) -> f32 {
        self.bounds
            .as_ref()
            .map_or(10.0, crate::features::viewport::logic::framing::radius)
    }

    /// Makes the view show `source` (`None` = no project). Rebuilds the GPU buffers only when the scene changed;
    /// a new scene with geometry re-frames the camera on the next frame drawn.
    pub fn sync(&mut self, source: Option<SceneSource>) -> SceneChange {
        let key = source.as_ref().map(SceneKey::of);
        if key == self.key {
            return SceneChange::Unchanged;
        }
        let buffers = match &source {
            Some(source) => scene_buffers::build(source.meshes, &source.material_colors),
            None => scene_buffers::build(&[], &[]),
        };
        let new_id = source.as_ref().map(|s| s.id);
        let reframe =
            new_id != self.scene_id || (self.bounds.is_none() && buffers.bounds.is_some());
        self.bounds = buffers.bounds;
        self.stats = SceneStats {
            triangles: buffers.triangles,
            skipped_meshes: buffers.skipped_meshes,
        };
        self.generation += 1;
        self.scene = Arc::new(SceneUpload::new(self.generation, buffers));
        self.key = key;
        if new_id != self.scene_id {
            self.slots.clear();
        }
        self.scene_id = new_id;
        if reframe {
            self.needs_frame = true;
            self.controller.adapt_to_scene(self.scene_radius());
        }
        if reframe && self.has_geometry() {
            SceneChange::Loaded
        } else {
            SceneChange::Updated
        }
    }

    /// Frames the camera on the whole scene as soon as the view size is known (used right after loading).
    pub(super) fn take_pending_frame(&mut self) -> bool {
        std::mem::take(&mut self.needs_frame)
    }

    /// Frames the whole scene, gliding there.
    pub fn frame_scene(&mut self, aspect: f32) {
        let bounds = self.bounds;
        self.controller.frame(bounds.as_ref(), aspect, true);
    }

    /// Looks at the whole scene from a predefined direction, gliding there.
    pub fn show_preset(&mut self, preset: ViewPreset, aspect: f32) {
        let bounds = self.bounds;
        self.controller
            .frame_preset(bounds.as_ref(), preset, aspect, true);
    }

    /// Frames the scene at once, without animation (first view of a loaded model).
    pub(super) fn frame_now(&mut self, aspect: f32) {
        let bounds = self.bounds;
        self.controller.frame(bounds.as_ref(), aspect, false);
    }
}
