//! Render queue, output and export entities.

use super::quality::RayTracingQuality;
use crate::features::rendering::engine::ViewPreset;
use std::path::PathBuf;
use std::time::Duration;

/// State of a render job.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum JobStatus {
    /// Waiting its turn.
    Queued,
    /// Being rendered.
    Rendering,
    /// Finished successfully.
    Done,
    /// Failed.
    Failed,
    /// Cancelled by the user.
    Cancelled,
}

impl JobStatus {
    /// Whether the job no longer runs.
    pub fn is_finished(self) -> bool {
        matches!(
            self,
            JobStatus::Done | JobStatus::Failed | JobStatus::Cancelled
        )
    }

    /// Localization key of the status name.
    pub fn label_key(self) -> &'static str {
        match self {
            JobStatus::Queued => "queue.status_queued",
            JobStatus::Rendering => "queue.status_rendering",
            JobStatus::Done => "queue.status_done",
            JobStatus::Failed => "queue.status_failed",
            JobStatus::Cancelled => "queue.status_cancelled",
        }
    }
}

/// What the interface knows about a job of the render queue: a snapshot taken from the running queue.
#[derive(Debug, Clone, PartialEq)]
pub struct JobInfo {
    /// Identifier that stays valid for the life of the queue.
    pub id: u64,
    /// Job name, e.g. the view being rendered.
    pub name: String,
    /// Quality preset name.
    pub preset: String,
    /// Output resolution, e.g. `1920×1080`.
    pub resolution: String,
    /// Current status.
    pub status: JobStatus,
    /// Progress, 0..=1.
    pub progress: f32,
    /// Time spent rendering so far (or in total when finished).
    pub elapsed: Duration,
    /// Estimated time left while rendering.
    pub remaining: Option<Duration>,
    /// Why it failed, when it did.
    pub error: Option<String>,
    /// File the picture was written to, once done.
    pub output: Option<PathBuf>,
}

/// Image formats of the export dialog.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ImageFormat {
    /// Lossless with optional transparency.
    Png,
    /// Lossy, small files.
    Jpeg,
}

impl ImageFormat {
    /// Formats in display order.
    pub const ALL: [ImageFormat; 2] = [ImageFormat::Png, ImageFormat::Jpeg];

    /// Display name.
    pub fn name(self) -> &'static str {
        match self {
            ImageFormat::Png => "PNG",
            ImageFormat::Jpeg => "JPEG",
        }
    }

    /// File extension without the dot.
    pub fn extension(self) -> &'static str {
        match self {
            ImageFormat::Png => "png",
            ImageFormat::Jpeg => "jpg",
        }
    }

    /// Whether the format can store transparency.
    pub fn supports_alpha(self) -> bool {
        matches!(self, ImageFormat::Png)
    }
}

/// Resolutions offered for exports and renders (width and height at 16:9).
pub const RESOLUTIONS: [(u32, u32); 6] = [
    (640, 360),
    (1280, 720),
    (1920, 1080),
    (2560, 1440),
    (3840, 2160),
    (7680, 4320),
];

/// Picture shapes offered next to the resolution; the resolution fixes the width.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum AspectRatio {
    /// Widescreen, 16:9.
    #[default]
    Wide,
    /// Classic photo, 3:2.
    Photo,
    /// Screen, 4:3.
    Screen,
    /// Square.
    Square,
    /// Ultra wide, 21:9.
    Cinema,
    /// Portrait, 9:16.
    Portrait,
}

impl AspectRatio {
    /// Ratios in display order.
    pub const ALL: [AspectRatio; 6] = [
        AspectRatio::Wide,
        AspectRatio::Photo,
        AspectRatio::Screen,
        AspectRatio::Square,
        AspectRatio::Cinema,
        AspectRatio::Portrait,
    ];

    /// Display text.
    pub fn name(self) -> &'static str {
        match self {
            AspectRatio::Wide => "16:9",
            AspectRatio::Photo => "3:2",
            AspectRatio::Screen => "4:3",
            AspectRatio::Square => "1:1",
            AspectRatio::Cinema => "21:9",
            AspectRatio::Portrait => "9:16",
        }
    }

    /// Width and height proportions.
    pub fn proportions(self) -> (u32, u32) {
        match self {
            AspectRatio::Wide => (16, 9),
            AspectRatio::Photo => (3, 2),
            AspectRatio::Screen => (4, 3),
            AspectRatio::Square => (1, 1),
            AspectRatio::Cinema => (21, 9),
            AspectRatio::Portrait => (9, 16),
        }
    }
}

/// Where and how a finished picture is written.
#[derive(Debug, Clone, PartialEq)]
pub struct OutputTarget {
    /// Folder chosen by the user; files are never written outside it.
    pub folder: PathBuf,
    /// File name typed by the user, without folders; the extension follows the format.
    pub file_name: String,
    /// Image format.
    pub format: ImageFormat,
    /// JPEG quality, 0..=1.
    pub jpeg_quality: f32,
}

/// State of the export dialog.
#[derive(Debug, Clone, PartialEq)]
pub struct ExportState {
    /// Selected format index into [`ImageFormat::ALL`].
    pub format: usize,
    /// Selected resolution index into [`RESOLUTIONS`].
    pub resolution: usize,
    /// Selected index into [`AspectRatio::ALL`].
    pub aspect: usize,
    /// Selected quality, as an index of the four levels.
    pub quality: usize,
    /// JPEG quality, normalized.
    pub jpeg_quality: f32,
    /// Output folder.
    pub folder: String,
    /// File name without folders.
    pub file_name: String,
    /// Transparent background switch.
    pub transparent: bool,
    /// The user pressed "Export".
    pub request: bool,
}

/// Default folder offered for pictures, inside the user's pictures when known.
pub fn default_output_folder() -> String {
    let base = std::env::var_os("USERPROFILE")
        .or_else(|| std::env::var_os("HOME"))
        .map(PathBuf::from)
        .unwrap_or_default();
    base.join("Pictures")
        .join("Guanacaste Render")
        .display()
        .to_string()
}

impl Default for ExportState {
    fn default() -> Self {
        Self {
            format: 0,
            resolution: 2,
            aspect: 0,
            quality: 1,
            jpeg_quality: 0.9,
            folder: default_output_folder(),
            file_name: "render".into(),
            transparent: false,
            request: false,
        }
    }
}

/// The four quality levels in display order.
pub const QUALITIES: [RayTracingQuality; 4] = [
    RayTracingQuality::Low,
    RayTracingQuality::Medium,
    RayTracingQuality::High,
    RayTracingQuality::Ultra,
];

/// UI state of the render studio screen.
#[derive(Debug, Clone, PartialEq)]
pub struct RenderStudioState {
    /// Selected tab (0 preview, 1 queue).
    pub tab: usize,
    /// Snapshot of the queue, refreshed every frame.
    pub jobs: Vec<JobInfo>,
    /// Resolution index for the output panel.
    pub resolution: usize,
    /// Index into [`AspectRatio::ALL`].
    pub aspect: usize,
    /// Index into [`QUALITIES`].
    pub quality: usize,
    /// Camera preset for the next render.
    pub view: ViewPreset,
    /// Transparent background switch.
    pub transparent: bool,
    /// Format index into [`ImageFormat::ALL`].
    pub format: usize,
    /// Output folder.
    pub output: String,
    /// File name without folders.
    pub file_name: String,
    /// Last message to show under the actions, with whether it is an error.
    pub notice: Option<(String, bool)>,
}

impl Default for RenderStudioState {
    fn default() -> Self {
        Self {
            tab: 0,
            jobs: Vec::new(),
            resolution: 1,
            aspect: 0,
            quality: 1,
            view: ViewPreset::default(),
            transparent: false,
            format: 0,
            output: default_output_folder(),
            file_name: "render".into(),
            notice: None,
        }
    }
}

/// Longest side a picture may have; larger shapes are scaled down so the preview texture stays valid.
pub const MAX_SIDE: u32 = 8192;

/// Width and height of a picture for a resolution index and an aspect ratio index; the width comes from the
/// resolution and the height from the ratio.
pub fn output_size(resolution: usize, aspect: usize) -> (u32, u32) {
    let (width, _) = RESOLUTIONS[resolution.min(RESOLUTIONS.len() - 1)];
    let (w, h) = AspectRatio::ALL[aspect.min(AspectRatio::ALL.len() - 1)].proportions();
    let height = ((width as u64 * h as u64 + w as u64 / 2) / w as u64).max(1) as u32;
    let longest = width.max(height);
    if longest <= MAX_SIDE {
        return (width, height);
    }
    let shrink = |side: u32| ((side as u64 * MAX_SIDE as u64 / longest as u64) as u32).max(1);
    (shrink(width), shrink(height))
}
