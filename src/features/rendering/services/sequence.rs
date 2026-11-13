//! Renders an animation frame by frame into a numbered PNG sequence.

use super::frame_source::{FrameRequest, FrameSource};
use crate::features::movie::models::Animation;
use std::fmt;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};

/// What to export and where.
#[derive(Debug, Clone, PartialEq)]
pub struct ExportSettings {
    /// Folder that receives the frames (created when missing).
    pub folder: PathBuf,
    /// File name prefix, also the name of the video (`<prefix>_000001.png`, `<prefix>.mp4`).
    pub prefix: String,
    /// Frame width in pixels.
    pub width: u32,
    /// Frame height in pixels.
    pub height: u32,
    /// Encode an MP4 with ffmpeg when it is installed.
    pub encode_video: bool,
}

/// Why an export stopped.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ExportError {
    /// A file or folder could not be written.
    Io(String),
    /// The frame source failed.
    Render(String),
    /// The user cancelled.
    Cancelled,
}

impl fmt::Display for ExportError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ExportError::Io(detail) | ExportError::Render(detail) => formatter.write_str(detail),
            ExportError::Cancelled => formatter.write_str("cancelled"),
        }
    }
}

/// A file name that is safe on every platform: no separators or reserved characters.
pub fn clean_prefix(prefix: &str) -> String {
    let cleaned: String = prefix
        .trim()
        .chars()
        .map(|c| {
            if c.is_alphanumeric() || matches!(c, '-' | '_' | ' ') {
                c
            } else {
                '_'
            }
        })
        .collect();
    if cleaned.is_empty() {
        "recorrido".into()
    } else {
        cleaned
    }
}

/// Name of frame `index` (zero-based): `<prefix>_000001.png` for the first one.
pub fn frame_file_name(prefix: &str, index: u32) -> String {
    format!("{prefix}_{:06}.png", index + 1)
}

/// The numbered-input pattern ffmpeg uses for the sequence.
pub fn frame_pattern(prefix: &str) -> String {
    format!("{prefix}_%06d.png")
}

/// Renders every frame of `animation` through `source` into `settings.folder`, calling `progress(done, total)`
/// after each one and stopping early when `cancel` is set. Returns the number of frames written.
pub fn export_sequence(
    animation: &Animation,
    source: &mut dyn FrameSource,
    settings: &ExportSettings,
    cancel: &AtomicBool,
    progress: &mut dyn FnMut(u32, u32),
) -> Result<u32, ExportError> {
    std::fs::create_dir_all(&settings.folder).map_err(|e| ExportError::Io(e.to_string()))?;
    let total = animation.frame_count();
    progress(0, total);
    for index in 0..total {
        if cancel.load(Ordering::Relaxed) {
            return Err(ExportError::Cancelled);
        }
        let time = animation.frame_time(index);
        let request = FrameRequest {
            index,
            time,
            pose: animation.sample(time),
            channels: animation.sample_channels(time),
            width: settings.width,
            height: settings.height,
        };
        let image = source.render(&request).map_err(ExportError::Render)?;
        write_frame(&settings.folder, &settings.prefix, index, &image)?;
        progress(index + 1, total);
    }
    Ok(total)
}

fn write_frame(
    folder: &Path,
    prefix: &str,
    index: u32,
    image: &image::RgbaImage,
) -> Result<(), ExportError> {
    let path = folder.join(frame_file_name(prefix, index));
    image
        .save_with_format(&path, image::ImageFormat::Png)
        .map_err(|e| ExportError::Io(format!("{}: {e}", path.display())))
}
