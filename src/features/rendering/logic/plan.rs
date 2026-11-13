//! From what the user chose in the studio or the export dialog to a job the queue can run.

use super::inputs::{CameraChoice, RenderInputs};
use crate::features::rendering::engine::{ViewPreset, World};
use crate::features::rendering::models::{
    output_size, ExportState, ImageFormat, OutputTarget, RenderStudioState, QUALITIES,
};
use crate::features::rendering::services::JobSpec;
use crate::services::localization::{tr, trf};
use std::path::PathBuf;
use std::sync::Arc;

/// The choices behind one render.
#[derive(Debug, Clone, PartialEq)]
pub struct JobPlan {
    /// Index into the resolutions.
    pub resolution: usize,
    /// Index into the aspect ratios.
    pub aspect: usize,
    /// Index into [`QUALITIES`].
    pub quality: usize,
    /// Where the camera stands when `camera` is a preset.
    pub view: ViewPreset,
    /// Where the camera comes from: a preset view, a saved camera, the 3D view or a photographic camera.
    pub camera: CameraChoice,
    /// Blur by distance from the focus plane (only for cameras that have a lens).
    pub depth_of_field: bool,
    /// Sky pixels become transparent.
    pub transparent: bool,
    /// Where the picture is written.
    pub output: OutputTarget,
}

fn target(folder: &str, file_name: &str, format: usize, jpeg_quality: f32) -> OutputTarget {
    OutputTarget {
        folder: PathBuf::from(folder.trim()),
        file_name: file_name.to_owned(),
        format: ImageFormat::ALL[format.min(ImageFormat::ALL.len() - 1)],
        jpeg_quality,
    }
}

/// The plan of the render studio's current settings. A saved camera chosen in the studio (`saved_view`) replaces
/// the preset view.
pub fn plan_from_studio(state: &RenderStudioState) -> JobPlan {
    let output = target(&state.output, &state.file_name, state.format, 0.9);
    JobPlan {
        resolution: state.resolution,
        aspect: state.aspect,
        quality: state.quality,
        view: state.view,
        camera: state
            .saved_view
            .map_or(CameraChoice::Preset, CameraChoice::Saved),
        depth_of_field: state.depth_of_field,
        transparent: state.transparent && output.format.supports_alpha(),
        output,
    }
}

/// The plan of the export dialog's current settings (corner view).
pub fn plan_from_export(state: &ExportState) -> JobPlan {
    let output = target(
        &state.folder,
        &state.file_name,
        state.format,
        state.jpeg_quality,
    );
    JobPlan {
        resolution: state.resolution,
        aspect: state.aspect,
        quality: state.quality,
        view: ViewPreset::Corner,
        camera: CameraChoice::Preset,
        depth_of_field: false,
        transparent: state.transparent && output.format.supports_alpha(),
        output,
    }
}

/// The name of the camera of `plan`, for the queue and the file name.
fn camera_name(plan: &JobPlan, inputs: &RenderInputs) -> String {
    match &plan.camera {
        CameraChoice::Preset => tr(plan.view.label_key()).to_owned(),
        CameraChoice::Saved(index) => inputs
            .cameras
            .get(*index)
            .map_or_else(|| tr("render.view_saved").to_owned(), |c| c.name.clone()),
        CameraChoice::Viewport(_) => tr("render.view_viewport").to_owned(),
        CameraChoice::Photo(..) => tr("render.view_photo").to_owned(),
    }
}

/// The queue job for `plan` on `world`, or `None` when the camera cannot be built (an empty scene, or a saved
/// camera that no longer exists). A batch of views appends the camera name to the file name so the pictures do
/// not share one file.
pub fn build_spec(
    world: &Arc<World>,
    inputs: &RenderInputs,
    plan: &JobPlan,
    suffix: bool,
) -> Option<JobSpec> {
    let size = output_size(plan.resolution, plan.aspect);
    let quality = QUALITIES[plan.quality.min(QUALITIES.len() - 1)];
    let (camera, photo) = inputs.camera(world, &plan.camera, plan.view, size)?;
    let mut request = inputs.request(camera, photo, plan.depth_of_field, quality);
    request.transparent_background = plan.transparent;
    let view = camera_name(plan, inputs);
    let mut output = plan.output.clone();
    if suffix {
        output.file_name = format!("{}_{}", output.file_name, view.to_lowercase());
    }
    Some(JobSpec {
        name: trf(
            "queue.job_name",
            &[&view, &format!("{}×{}", size.0, size.1)],
        ),
        preset: tr(quality.label_key()).to_owned(),
        scene: Arc::clone(world),
        request,
        output,
    })
}
