//! Embedded textures of the detailed families (leaves, bark, ...): decoded from the document's RGBA arrays, cut out by
//! their alpha cutoff and written once as PNG files, because materials reference images by path. Files are named by
//! the hash of their content, so importing the same project again reuses them without decoding anything.

use super::base64;
use image::codecs::png::{CompressionType, FilterType, PngEncoder};
use image::{ExtendedColorType, ImageEncoder};
use sha2::{Digest, Sha256};
use std::path::{Path, PathBuf};

/// Largest side kept for an embedded texture; bigger images are averaged down by whole factors.
pub const MAX_SIDE: usize = 1024;

/// Folder where the texture files are kept: the application data folder, or the temporary folder when there is none
/// (and always in unit tests, which must not touch the user's data).
pub fn texture_dir() -> PathBuf {
    if cfg!(test) {
        return std::env::temp_dir().join("guanacaste-render-test-textures");
    }
    std::env::var_os("LOCALAPPDATA")
        .or_else(|| std::env::var_os("XDG_DATA_HOME"))
        .map_or_else(std::env::temp_dir, PathBuf::from)
        .join("Guanacaste Render")
        .join("imported-textures")
}

fn file_name(name: &str, key: &str) -> String {
    let clean: String = name
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || c == '-' {
                c
            } else {
                '_'
            }
        })
        .take(40)
        .collect();
    format!("{clean}-{key}.png")
}

/// Averages `rgba` (width x height) down by the integer `factor`, weighting colors by alpha.
fn shrink(rgba: &[u8], width: usize, height: usize, factor: usize) -> (Vec<u8>, usize, usize) {
    let (w, h) = (width / factor, height / factor);
    let mut out = vec![0u8; w * h * 4];
    for y in 0..h {
        for x in 0..w {
            let (mut c, mut a, mut n) = ([0f32; 3], 0f32, 0f32);
            for dy in 0..factor {
                for dx in 0..factor {
                    let at = ((y * factor + dy) * width + x * factor + dx) * 4;
                    let alpha = f32::from(rgba[at + 3]);
                    for k in 0..3 {
                        c[k] += f32::from(rgba[at + k]) * alpha;
                    }
                    a += alpha;
                    n += 1.0;
                }
            }
            let at = (y * w + x) * 4;
            for k in 0..3 {
                out[at + k] = if a > 0.0 { (c[k] / a).round() as u8 } else { 0 };
            }
            out[at + 3] = (a / n).round() as u8;
        }
    }
    (out, w, h)
}

/// Writes the texture and returns its path. `cutoff` > 0 turns the alpha into a hard cut-out at that coverage.
#[allow(clippy::manual_is_multiple_of)]
pub fn store(
    dir: &Path,
    name: &str,
    width: usize,
    height: usize,
    rgba_b64: &str,
    cutoff: f32,
) -> Option<PathBuf> {
    let key = format!("{:x}", Sha256::digest(rgba_b64.as_bytes()));
    let path = dir.join(file_name(name, &key[..16]));
    if path.exists() {
        return Some(path);
    }
    let bytes = base64::decode(rgba_b64)?;
    if width == 0 || height == 0 || bytes.len() != width * height * 4 {
        return None;
    }
    let factor = (width.max(height) / MAX_SIDE).max(1);
    let (mut pixels, w, h) = if factor > 1 && width % factor == 0 && height % factor == 0 {
        shrink(&bytes, width, height, factor)
    } else {
        (bytes, width, height)
    };
    if cutoff > 0.0 {
        let limit = (cutoff * 255.0).round() as u8;
        for pixel in pixels.chunks_exact_mut(4) {
            pixel[3] = if pixel[3] >= limit { 255 } else { 0 };
        }
    }
    std::fs::create_dir_all(dir).ok()?;
    let partial = path.with_extension("tmp");
    let file = std::fs::File::create(&partial).ok()?;
    PngEncoder::new_with_quality(
        std::io::BufWriter::new(file),
        CompressionType::Fast,
        FilterType::Sub,
    )
    .write_image(&pixels, w as u32, h as u32, ExtendedColorType::Rgba8)
    .ok()?;
    std::fs::rename(&partial, &path).ok()?;
    Some(path)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_texture_is_written_once_and_reused() {
        // A 2x2 image: opaque red, half-transparent green, transparent, opaque blue.
        let bytes: Vec<u8> = vec![255, 0, 0, 255, 0, 255, 0, 128, 0, 0, 0, 0, 0, 0, 255, 255];
        let mut text = String::new();
        let table = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
        for chunk in bytes.chunks(3) {
            let n = chunk
                .iter()
                .enumerate()
                .fold(0u32, |a, (i, b)| a | u32::from(*b) << (16 - 8 * i));
            for i in 0..4 {
                text.push(if i <= chunk.len() {
                    table[(n >> (18 - 6 * i) & 63) as usize] as char
                } else {
                    '='
                });
            }
        }
        let dir = std::env::temp_dir().join("guanacaste-render-texture-unit");
        let first = store(&dir, "leaf test", 2, 2, &text, 0.5).expect("written");
        assert!(first.exists());
        assert_eq!(
            store(&dir, "leaf test", 2, 2, &text, 0.5),
            Some(first.clone())
        );
        let image = image::open(&first).unwrap().to_rgba8();
        assert_eq!(
            image.get_pixel(1, 0)[3],
            255,
            "128/255 coverage passes a 0.5 cutoff"
        );
        assert_eq!(image.get_pixel(0, 1)[3], 0);
        assert!(store(&dir, "bad", 3, 3, &text, 0.0).is_none());
    }
}
