//! Render queue and export entities.

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

/// A job of the render queue.
#[derive(Debug, Clone, PartialEq)]
pub struct RenderJob {
    /// Job name.
    pub name: &'static str,
    /// Render preset.
    pub preset: &'static str,
    /// Output resolution, e.g. `3840×2160`.
    pub resolution: &'static str,
    /// Current status.
    pub status: JobStatus,
    /// Progress, 0..=1.
    pub progress: f32,
    /// Remaining time or result note.
    pub eta: &'static str,
}

/// Image formats of the export dialog.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ImageFormat {
    /// Lossless with optional transparency.
    Png,
    /// Lossy, small files.
    Jpeg,
    /// 16-bit float HDR.
    Exr,
}

impl ImageFormat {
    /// Formats in display order.
    pub const ALL: [ImageFormat; 3] = [ImageFormat::Png, ImageFormat::Jpeg, ImageFormat::Exr];

    /// File extension and display name.
    pub fn name(self) -> &'static str {
        match self {
            ImageFormat::Png => "PNG",
            ImageFormat::Jpeg => "JPEG",
            ImageFormat::Exr => "OpenEXR",
        }
    }

    /// Whether the format can store transparency.
    pub fn supports_alpha(self) -> bool {
        matches!(self, ImageFormat::Png | ImageFormat::Exr)
    }
}

/// Resolutions offered for exports and renders.
pub const RESOLUTIONS: [(u32, u32); 4] = [(1920, 1080), (2560, 1440), (3840, 2160), (7680, 4320)];

/// State of the export dialog.
#[derive(Debug, Clone, PartialEq)]
pub struct ExportState {
    /// Selected format index into [`ImageFormat::ALL`].
    pub format: usize,
    /// Selected resolution index into [`RESOLUTIONS`].
    pub resolution: usize,
    /// JPEG quality, normalized.
    pub quality: f32,
    /// Output file path.
    pub path: String,
    /// Transparent background switch.
    pub transparent: bool,
}

impl Default for ExportState {
    fn default() -> Self {
        Self {
            format: 0,
            resolution: 2,
            quality: 0.9,
            path: "C:/Proyectos/Guanacaste/Renders/villa.png".into(),
            transparent: false,
        }
    }
}

/// UI state of the render studio screen.
#[derive(Debug, Clone, PartialEq)]
pub struct RenderStudioState {
    /// Selected tab (0 preview, 1 queue).
    pub tab: usize,
    /// Jobs of the queue.
    pub jobs: Vec<RenderJob>,
    /// Resolution index for the output panel.
    pub resolution: usize,
    /// Samples per pixel, normalized.
    pub samples: f32,
    /// Denoise switch.
    pub denoise: bool,
    /// Output folder.
    pub output: String,
}
