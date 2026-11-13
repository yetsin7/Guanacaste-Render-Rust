//! User-facing description of how an export ended.

use crate::features::rendering::services::job::{ExportReport, JobEnd, VideoResult};
use crate::services::localization::trf;

/// One sentence in Spanish that says what was generated and, when there is no MP4, why.
pub fn describe(end: &JobEnd) -> String {
    match end {
        JobEnd::Finished(report) => describe_report(report),
        JobEnd::Failed(detail) => trf("movie.result_failed", &[detail]),
        JobEnd::Cancelled => trf("movie.result_cancelled", &[]),
    }
}

fn describe_report(report: &ExportReport) -> String {
    let frames = report.frames.to_string();
    let folder = report.folder.display().to_string();
    match &report.video {
        VideoResult::NotRequested => trf("movie.result_sequence", &[&frames, &folder]),
        VideoResult::FfmpegMissing => trf("movie.result_no_ffmpeg", &[&frames, &folder]),
        VideoResult::Encoded(video) => trf(
            "movie.result_video",
            &[&video.display().to_string(), &frames, &folder],
        ),
        VideoResult::Failed(detail) => {
            trf("movie.result_video_failed", &[detail, &frames, &folder])
        }
    }
}
