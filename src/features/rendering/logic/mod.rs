//! Rendering rules: export size estimates, queue summaries and the text of job times.

pub mod inputs;
pub mod panorama_view;
pub mod plan;
pub mod world;

use super::models::{ImageFormat, JobInfo, JobStatus, RESOLUTIONS};
use crate::services::localization::{tr, trf};
use std::time::Duration;

/// Rough size in megabytes of an exported image of `size` pixels.
pub fn estimate_megabytes(format: ImageFormat, size: (u32, u32), quality: f32) -> f32 {
    let pixels = size.0 as f32 * size.1 as f32;
    let bytes_per_pixel = match format {
        ImageFormat::Png => 2.2,
        ImageFormat::Jpeg => 0.1 + 0.9 * quality.clamp(0.0, 1.0),
    };
    pixels * bytes_per_pixel / 1_048_576.0
}

/// Counts of `(queued, rendering, finished)` jobs.
pub fn summary(jobs: &[JobInfo]) -> (usize, usize, usize) {
    let count = |status: JobStatus| jobs.iter().filter(|job| job.status == status).count();
    (
        count(JobStatus::Queued),
        count(JobStatus::Rendering),
        jobs.iter().filter(|job| job.status.is_finished()).count(),
    )
}

/// Display text of a resolution, e.g. `3840×2160`.
pub fn resolution_label(index: usize) -> String {
    let (width, height) = RESOLUTIONS[index.min(RESOLUTIONS.len() - 1)];
    format!("{width}×{height}")
}

/// A duration as `hh:mm:ss`.
pub fn clock(duration: Duration) -> String {
    let seconds = duration.as_secs();
    format!(
        "{:02}:{:02}:{:02}",
        seconds / 3600,
        seconds / 60 % 60,
        seconds % 60
    )
}

/// The time column of a job: remaining time, total time, or the reason it stopped.
pub fn time_text(job: &JobInfo) -> String {
    match job.status {
        JobStatus::Queued => tr("queue.eta_waiting").to_owned(),
        JobStatus::Rendering => match job.remaining {
            Some(left) => trf("queue.eta_remaining", &[&clock(left)]),
            None => tr("queue.eta_calculating").to_owned(),
        },
        JobStatus::Done => trf("queue.eta_done", &[&clock(job.elapsed)]),
        JobStatus::Cancelled => tr("queue.eta_cancelled").to_owned(),
        JobStatus::Failed => trf("queue.eta_failed", &[job.error.as_deref().unwrap_or("")]),
    }
}
