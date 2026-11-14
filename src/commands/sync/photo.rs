//! Photo-mode state in the project document: the filters (effects), the photographic camera and the quality and
//! output choices of the left rail. Only values that differ from what was loaded are written, so an untouched
//! project stays clean.

use crate::features::camera::models::camera::PhotoCamera;
use crate::features::camera::models::stack::FilterStack;
use crate::features::projects::models::{ProjectDocument, StudioDoc};
use crate::features::rendering::models::RenderStudioState;
use crate::models::AppState;

/// What the photo state looked like when it last matched the document.
#[derive(Default)]
pub struct PhotoBaseline {
    filters: FilterStack,
    camera: PhotoCamera,
    studio: StudioDoc,
}

/// The studio choices as stored in the document.
pub fn studio_doc(state: &RenderStudioState) -> StudioDoc {
    StudioDoc {
        resolution: state.resolution,
        aspect: state.aspect,
        quality: state.quality,
        depth_of_field: state.depth_of_field,
        transparent: state.transparent,
        format: state.format,
        output: state.output.clone(),
        file_name: state.file_name.clone(),
    }
}

fn apply_studio(state: &mut RenderStudioState, doc: &StudioDoc) {
    state.resolution = doc.resolution;
    state.aspect = doc.aspect;
    state.quality = doc.quality;
    state.depth_of_field = doc.depth_of_field;
    state.transparent = doc.transparent;
    state.format = doc.format;
    if !doc.output.is_empty() {
        state.output = doc.output.clone();
    }
    state.file_name = doc.file_name.clone();
    state.saved_view = None;
}

/// Loads the photo state of `doc` (or the defaults when it has none) into the live states.
pub fn load(app: &mut AppState, doc: Option<&ProjectDocument>) {
    let defaults = crate::features::camera::models::state::CameraStudioState::default();
    app.camera_studio.stack = defaults.stack;
    app.camera_studio.preset = defaults.preset;
    app.camera_studio.camera = PhotoCamera::default();
    app.camera_studio.selected = None;
    app.studio.state = RenderStudioState {
        output: app.studio.state.output.clone(),
        ..RenderStudioState::default()
    };
    if let Some(doc) = doc {
        if doc.studio.is_some() || !doc.filters.is_empty() {
            app.camera_studio.stack = doc.filters.clone();
            app.camera_studio.mark_custom();
        }
        if let Some(camera) = doc.photo_camera {
            app.camera_studio.camera = camera;
        }
        if let Some(studio) = &doc.studio {
            apply_studio(&mut app.studio.state, studio);
        }
    }
    app.baselines.photo = PhotoBaseline {
        filters: app.camera_studio.stack.clone(),
        camera: app.camera_studio.camera,
        studio: studio_doc(&app.studio.state),
    };
}

/// Writes the photo state into `doc` when it changed; returns whether it did.
pub fn flush(
    base: &mut PhotoBaseline,
    stack: &FilterStack,
    camera: &PhotoCamera,
    studio: &RenderStudioState,
    doc: &mut ProjectDocument,
) -> bool {
    let live = studio_doc(studio);
    if base.filters == *stack && base.camera == *camera && base.studio == live {
        return false;
    }
    doc.filters = stack.clone();
    doc.photo_camera = Some(*camera);
    doc.studio = Some(live.clone());
    *base = PhotoBaseline {
        filters: stack.clone(),
        camera: *camera,
        studio: live,
    };
    true
}
