//! Decoding of image files from disk (PNG and JPEG).

use super::bitmap::Bitmap;
use image::{ImageDecoder, ImageReader};
use std::path::Path;

/// Largest file read, in bytes.
const MAX_FILE_BYTES: u64 = 256 * 1024 * 1024;
/// Largest decoded image, in bytes of RGBA data.
const MAX_DECODED_BYTES: u64 = 256 * 1024 * 1024;

/// Why a texture could not be loaded.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LoadError {
    /// The file does not exist.
    NotFound,
    /// The file exists but could not be read.
    Unreadable,
    /// The file is not a PNG or JPEG image, or it is damaged.
    Unsupported,
    /// The image is larger than the memory limit.
    TooLarge,
}

/// Reads and decodes the image at `path` into 8-bit RGBA.
pub fn load_file(path: &Path) -> Result<Bitmap, LoadError> {
    let meta = std::fs::metadata(path).map_err(|e| match e.kind() {
        std::io::ErrorKind::NotFound => LoadError::NotFound,
        _ => LoadError::Unreadable,
    })?;
    if !meta.is_file() {
        return Err(LoadError::Unreadable);
    }
    if meta.len() > MAX_FILE_BYTES {
        return Err(LoadError::TooLarge);
    }
    let bytes = std::fs::read(path).map_err(|_| LoadError::Unreadable)?;
    decode(&bytes)
}

/// Decodes PNG or JPEG `bytes` (the format is guessed from the content, not the name).
pub fn decode(bytes: &[u8]) -> Result<Bitmap, LoadError> {
    let reader = ImageReader::new(std::io::Cursor::new(bytes))
        .with_guessed_format()
        .map_err(|_| LoadError::Unsupported)?;
    let decoder = reader.into_decoder().map_err(|_| LoadError::Unsupported)?;
    let (w, h) = decoder.dimensions();
    if (w as u64) * (h as u64) * 4 > MAX_DECODED_BYTES {
        return Err(LoadError::TooLarge);
    }
    let image = image::DynamicImage::from_decoder(decoder).map_err(|_| LoadError::Unsupported)?;
    let rgba = image.to_rgba8();
    let pixels = rgba.pixels().map(|p| p.0).collect::<Vec<[u8; 4]>>();
    if pixels.is_empty() {
        return Err(LoadError::Unsupported);
    }
    Ok(Bitmap {
        width: rgba.width() as usize,
        height: rgba.height() as usize,
        pixels,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn png(width: u32, height: u32) -> Vec<u8> {
        let image = image::RgbaImage::from_fn(width, height, |x, y| {
            image::Rgba([x as u8 * 10, y as u8 * 10, 7, 200])
        });
        let mut out = Vec::new();
        image
            .write_to(&mut std::io::Cursor::new(&mut out), image::ImageFormat::Png)
            .expect("encode");
        out
    }

    #[test]
    fn decodes_png_bytes() {
        let bitmap = decode(&png(3, 2)).expect("png");
        assert_eq!((bitmap.width, bitmap.height), (3, 2));
        assert_eq!(bitmap.at(2, 1), [20, 10, 7, 200]);
    }

    #[test]
    fn decodes_jpeg_bytes() {
        let image = image::RgbImage::from_pixel(8, 8, image::Rgb([200, 40, 40]));
        let mut out = Vec::new();
        image
            .write_to(
                &mut std::io::Cursor::new(&mut out),
                image::ImageFormat::Jpeg,
            )
            .expect("encode jpeg");
        let bitmap = decode(&out).expect("jpeg");
        assert_eq!((bitmap.width, bitmap.height), (8, 8));
        let p = bitmap.at(4, 4);
        assert!(p[0] > 180 && p[1] < 70 && p[3] == 255);
    }

    #[test]
    fn garbage_is_unsupported() {
        assert_eq!(decode(b"not an image at all"), Err(LoadError::Unsupported));
        assert_eq!(decode(&[]), Err(LoadError::Unsupported));
    }

    #[test]
    fn missing_files_are_reported() {
        assert_eq!(
            load_file(Path::new("definitely/not/here.png")),
            Err(LoadError::NotFound)
        );
    }

    #[test]
    fn loads_a_file_from_disk() {
        let path = std::env::temp_dir().join(format!("gr-tex-{}.png", std::process::id()));
        std::fs::write(&path, png(4, 4)).expect("write");
        let bitmap = load_file(&path).expect("load");
        let _ = std::fs::remove_file(&path);
        assert_eq!((bitmap.width, bitmap.height), (4, 4));
    }
}
