//! Rendering rules: export size estimates and queue summaries.

use super::models::{ImageFormat, JobStatus, RenderJob, RESOLUTIONS};

/// Rough size in megabytes of an exported image.
pub fn estimate_megabytes(format: ImageFormat, resolution: usize, quality: f32) -> f32 {
    let (width, height) = RESOLUTIONS[resolution.min(RESOLUTIONS.len() - 1)];
    let pixels = width as f32 * height as f32;
    let bytes_per_pixel = match format {
        ImageFormat::Png => 2.2,
        ImageFormat::Jpeg => 0.1 + 0.9 * quality.clamp(0.0, 1.0),
        ImageFormat::Exr => 7.0,
    };
    pixels * bytes_per_pixel / 1_048_576.0
}

/// Counts of `(queued, rendering, finished)` jobs.
pub fn summary(jobs: &[RenderJob]) -> (usize, usize, usize) {
    let queued = jobs
        .iter()
        .filter(|job| job.status == JobStatus::Queued)
        .count();
    let rendering = jobs
        .iter()
        .filter(|job| job.status == JobStatus::Rendering)
        .count();
    (
        queued,
        rendering,
        jobs.iter().filter(|job| job.status.is_finished()).count(),
    )
}

/// Removes finished jobs from the queue and returns how many were removed.
pub fn clear_finished(jobs: &mut Vec<RenderJob>) -> usize {
    let before = jobs.len();
    jobs.retain(|job| !job.status.is_finished());
    before - jobs.len()
}

/// Display text of a resolution, e.g. `3840×2160`.
pub fn resolution_label(index: usize) -> String {
    let (width, height) = RESOLUTIONS[index.min(RESOLUTIONS.len() - 1)];
    format!("{width}×{height}")
}
