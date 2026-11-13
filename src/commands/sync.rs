//! Keeps the domain states of `AppState` and the open project's document in step. When a project is created,
//! opened or imported its document is loaded into the live states; afterwards every change is written back into
//! the document (and the project marked as modified), so Save persists objects, landscape, lighting, animation,
//! saved cameras and environment.

use super::camera_link::LinkMarks;
use super::ground;
use crate::features::build::models::ObjectsState;
use crate::features::environment::EnvironmentSettings;
use crate::features::landscape::logic::persist;
use crate::features::landscape::models::{Landscape, LandscapeState};
use crate::features::lighting::models::{LightingSetup, LightingState};
use crate::features::materials::logic::sync as material_sync;
use crate::features::movie::models::{Animation, MovieState};
use crate::features::projects::models::CameraDoc;
use crate::models::AppState;

#[cfg(test)]
mod tests;

/// What each domain state looked like when it last matched the document.
#[derive(Default)]
pub struct Baselines {
    key: Option<([u8; 16], u64)>,
    lighting: LightingSetup,
    animation: Animation,
    cameras: Vec<CameraDoc>,
    environment: Option<EnvironmentSettings>,
    landscape_revision: u64,
    /// Meters the imported model was moved by hand since it was loaded (shown in the model section).
    pub model_shift: [f32; 3],
    /// Last values exchanged between the movie camera and the 3D view.
    pub link: LinkMarks,
}

/// Brings the states and the document in step. Call once per frame before drawing.
pub fn step(app: &mut AppState) {
    let key = app
        .workspace
        .project
        .as_ref()
        .map(|p| (p.id, app.workspace.generation));
    if key != app.baselines.key {
        app.baselines.key = key;
        load(app);
    } else {
        flush(app);
    }
    material_sync::adopt(&mut app.materials, app.workspace.project.as_ref());
    material_sync::flush(&mut app.materials, app.workspace.project.as_mut());
}

/// Replaces the live states with the content of the (new) project, or with empty ones without a project.
fn load(app: &mut AppState) {
    let b = &mut app.baselines;
    b.model_shift = [0.0; 3];
    b.link = LinkMarks::default();
    app.movie = MovieState::default();
    app.cameras = Vec::new();
    app.thumbs.items.clear();
    let Some(project) = app.workspace.project.as_mut() else {
        app.objects = ObjectsState::default();
        app.landscape = LandscapeState::default();
        app.lighting = LightingState::default();
        app.environment = EnvironmentSettings::default();
        app.animation = Animation::default();
        b.lighting = LightingSetup::default();
        b.animation = Animation::default();
        b.cameras = Vec::new();
        b.environment = None;
        b.landscape_revision = 0;
        app.composer.invalidate();
        return;
    };
    // A model without a stored ground is stood on the base ground once, and that is remembered in the file.
    let first_time = project.document.landscape.is_none() && !project.meshes.is_empty();
    let moved = first_time && ground::settle(project);
    if moved {
        project.dirty = true;
        project.document.landscape = Some(Landscape::default());
        app.composer.invalidate();
    }
    let doc = &mut project.document;
    app.objects = ObjectsState::from_doc(doc.objects.clone());
    app.landscape = LandscapeState::with_landscape(persist::load_from(doc).unwrap_or_default());
    app.lighting = LightingState::new(doc.lighting.clone());
    app.environment = doc.environment.unwrap_or_default();
    let mut animation = doc.animation.clone();
    animation.normalize();
    doc.animation = animation.clone();
    app.animation = animation;
    app.cameras = doc.cameras.clone();
    b.lighting = doc.lighting.clone();
    b.animation = app.animation.clone();
    b.cameras = app.cameras.clone();
    b.environment = doc.environment;
    b.landscape_revision = app.landscape.revision;
    app.composer.invalidate();
}

/// Writes the live states that changed into the document.
fn flush(app: &mut AppState) {
    let Some(project) = app.workspace.project.as_mut() else {
        return;
    };
    if project.read_only {
        return;
    }
    let b = &mut app.baselines;
    let mut changed = false;
    if app.objects.dirty {
        project.document.objects = app.objects.to_doc();
        app.objects.dirty = false;
        changed = true;
    }
    if app.landscape.revision != b.landscape_revision {
        persist::save_into(&mut project.document, &app.landscape.landscape);
        b.landscape_revision = app.landscape.revision;
        changed = true;
    }
    if app.lighting.setup != b.lighting {
        project.document.lighting = app.lighting.setup.clone();
        b.lighting = app.lighting.setup.clone();
        changed = true;
    }
    if app.animation != b.animation {
        app.animation.normalize();
        project.document.animation = app.animation.clone();
        b.animation = app.animation.clone();
        changed = true;
    }
    if app.cameras != b.cameras {
        project.document.cameras = app.cameras.clone();
        b.cameras = app.cameras.clone();
        changed = true;
    }
    let environment = Some(app.environment);
    if environment != b.environment && environment != Some(EnvironmentSettings::default()) {
        project.document.environment = environment;
        b.environment = environment;
        changed = true;
    }
    if changed {
        project.dirty = true;
    }
}

/// Moves the imported model by `delta` meters and records it in the project. Returns whether it moved.
pub fn move_model(app: &mut AppState, delta: [f32; 3]) -> bool {
    let Some(project) = app.workspace.project.as_mut() else {
        return false;
    };
    if !ground::shift(project, delta) {
        return false;
    }
    project.dirty = true;
    for axis in 0..3 {
        app.baselines.model_shift[axis] += delta[axis];
    }
    app.composer.invalidate();
    true
}

/// Rests the imported model on the ground (lowest point at Z = 0).
pub fn settle_model(app: &mut AppState) -> bool {
    let Some(project) = app.workspace.project.as_mut() else {
        return false;
    };
    let moved = ground::settle(project);
    if moved {
        project.dirty = true;
        app.composer.invalidate();
    }
    moved
}

/// Shows or hides the base ground; the choice is saved with the project.
pub fn set_ground_visible(app: &mut AppState, visible: bool) {
    if app.landscape.landscape.visible != visible {
        app.landscape.landscape.visible = visible;
        app.landscape.touch();
    }
}
