//! From what the user chose in the studio or the export dialog to a job the queue can run.

use crate::features::import_export::models::ImportedScene;
use crate::features::rendering::engine::{frame_bounds, RenderRequest, ViewPreset};
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
    /// Where the camera stands.
    pub view: ViewPreset,
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

/// The plan of the render studio's current settings.
pub fn plan_from_studio(state: &RenderStudioState) -> JobPlan {
    let output = target(&state.output, &state.file_name, state.format, 0.9);
    JobPlan {
        resolution: state.resolution,
        aspect: state.aspect,
        quality: state.quality,
        view: state.view,
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
        transparent: state.transparent && output.format.supports_alpha(),
        output,
    }
}

/// The queue job for `plan` on `scene`, or `None` when the scene has no geometry to frame. A view other than
/// the first appends its name to the file name so a batch of views does not share one file.
pub fn build_spec(scene: &Arc<ImportedScene>, plan: &JobPlan, suffix: bool) -> Option<JobSpec> {
    let bounds = scene.bounds()?;
    let (width, height) = output_size(plan.resolution, plan.aspect);
    let quality = QUALITIES[plan.quality.min(QUALITIES.len() - 1)];
    let camera = frame_bounds(&bounds, plan.view, width, height);
    let mut request = RenderRequest::new(camera, quality);
    request.transparent_background = plan.transparent;
    let view = tr(plan.view.label_key());
    let mut output = plan.output.clone();
    if suffix {
        output.file_name = format!("{}_{}", output.file_name, view.to_lowercase());
    }
    Some(JobSpec {
        name: trf("queue.job_name", &[view, &format!("{width}×{height}")]),
        preset: tr(quality.label_key()).to_owned(),
        scene: Arc::clone(scene),
        request,
        output,
    })
}
