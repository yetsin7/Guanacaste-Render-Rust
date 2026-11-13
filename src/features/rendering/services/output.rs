//! Writing finished pictures to the folder the user chose, and never anywhere else.

use crate::features::rendering::models::{ImageFormat, OutputTarget};
use image::codecs::jpeg::JpegEncoder;
use image::{ExtendedColorType, ImageEncoder, RgbaImage};
use std::fs::{self, File};
use std::io::BufWriter;
use std::path::PathBuf;

/// Why a picture could not be written.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OutputError {
    /// No folder was given.
    NoFolder,
    /// The folder exists as a file or could not be created.
    BadFolder(String),
    /// Writing or encoding failed.
    Write(String),
}

/// Names Windows reserves for devices; a file cannot be called like this even with an extension.
const RESERVED: [&str; 22] = [
    "CON", "PRN", "AUX", "NUL", "COM1", "COM2", "COM3", "COM4", "COM5", "COM6", "COM7", "COM8",
    "COM9", "LPT1", "LPT2", "LPT3", "LPT4", "LPT5", "LPT6", "LPT7", "LPT8", "LPT9",
];

/// Turns whatever the user typed into a plain file stem: no folders, no forbidden characters, no extension
/// of a known image format, never empty.
pub fn safe_stem(typed: &str) -> String {
    let last = typed.rsplit(['/', '\\']).next().unwrap_or("");
    let cleaned: String = last
        .chars()
        .filter(|c| !c.is_control() && !"<>:\"|?*".contains(*c))
        .collect();
    let mut stem = cleaned.trim().trim_matches('.').trim().to_owned();
    for extension in [".png", ".jpg", ".jpeg"] {
        if stem.to_lowercase().ends_with(extension) {
            stem.truncate(stem.len() - extension.len());
            stem = stem.trim().trim_end_matches('.').to_owned();
        }
    }
    let base = stem.split('.').next().unwrap_or("").to_uppercase();
    if stem.is_empty() || RESERVED.contains(&base.as_str()) {
        return "render".to_owned();
    }
    stem
}

/// The path a picture will be written to: inside `target.folder`, named after the typed name, with the
/// extension of the format, and never replacing an existing file (a counter is appended instead).
pub fn resolve_path(target: &OutputTarget) -> Result<PathBuf, OutputError> {
    if target.folder.as_os_str().is_empty() {
        return Err(OutputError::NoFolder);
    }
    let stem = safe_stem(&target.file_name);
    let extension = target.format.extension();
    let mut path = target.folder.join(format!("{stem}.{extension}"));
    let mut counter = 2;
    while path.exists() && counter < 10_000 {
        path = target.folder.join(format!("{stem}_{counter}.{extension}"));
        counter += 1;
    }
    if path.parent() != Some(target.folder.as_path()) {
        return Err(OutputError::Write(path.display().to_string()));
    }
    Ok(path)
}

/// Writes `image` as the target says and returns the file it created. The folder is created when missing.
pub fn save_image(image: &RgbaImage, target: &OutputTarget) -> Result<PathBuf, OutputError> {
    if target.folder.as_os_str().is_empty() {
        return Err(OutputError::NoFolder);
    }
    fs::create_dir_all(&target.folder).map_err(|e| OutputError::BadFolder(e.to_string()))?;
    let path = resolve_path(target)?;
    let write = |path: &PathBuf| -> Result<(), String> {
        match target.format {
            ImageFormat::Png => image
                .save_with_format(path, image::ImageFormat::Png)
                .map_err(|e| e.to_string()),
            ImageFormat::Jpeg => {
                let flat = flatten(image);
                let file = File::create(path).map_err(|e| e.to_string())?;
                let quality = (target.jpeg_quality.clamp(0.0, 1.0) * 99.0).round() as u8 + 1;
                JpegEncoder::new_with_quality(BufWriter::new(file), quality)
                    .write_image(
                        flat.as_raw(),
                        flat.width(),
                        flat.height(),
                        ExtendedColorType::Rgb8,
                    )
                    .map_err(|e| e.to_string())
            }
        }
    };
    write(&path).map_err(OutputError::Write)?;
    Ok(path)
}

/// The picture over a white background, as three channels (JPEG has no transparency).
fn flatten(image: &RgbaImage) -> image::RgbImage {
    let mut out = image::RgbImage::new(image.width(), image.height());
    for (x, y, pixel) in image.enumerate_pixels() {
        let a = pixel[3] as u32;
        let mix = |c: u8| ((c as u32 * a + 255 * (255 - a) + 127) / 255) as u8;
        out.put_pixel(
            x,
            y,
            image::Rgb([mix(pixel[0]), mix(pixel[1]), mix(pixel[2])]),
        );
    }
    out
}
