//! The sample-image preview pipeline: crop to the camera framing, expose, filter.

use super::filters::color;
use crate::features::camera::models::camera::PhotoCamera;
use crate::features::camera::models::exposure::PREVIEW_SCENE_EV100;
use crate::features::camera::models::image_buffer::ImageBuffer;
use crate::features::camera::models::stack::FilterStack;
use crate::services::textures::Placeholder;

/// Width in pixels of the sample image used for previews.
pub const PREVIEW_WIDTH: usize = 480;

/// Largest centered crop of `image` with the given width/height `aspect`.
pub fn crop_to_aspect(image: &ImageBuffer, aspect: f32) -> ImageBuffer {
    let (w, h) = (image.width() as f32, image.height() as f32);
    let (crop_w, crop_h) = if w / h > aspect {
        ((h * aspect).round().max(1.0), h)
    } else {
        (w, (w / aspect).round().max(1.0))
    };
    let (crop_w, crop_h) = (crop_w as usize, crop_h as usize);
    let (left, top) = ((image.width() - crop_w) / 2, (image.height() - crop_h) / 2);
    let mut out = ImageBuffer::filled(crop_w, crop_h, [0.0; 4]);
    for y in 0..crop_h {
        for x in 0..crop_w {
            out.set_pixel(x, y, image.pixel((left + x) as isize, (top + y) as isize));
        }
    }
    out
}

/// Shrinks `image` to `width` pixels wide, averaging the pixels each output pixel covers; an image already that
/// narrow is returned unchanged.
pub fn downscale(image: &ImageBuffer, width: usize) -> ImageBuffer {
    if width == 0 || image.width() <= width {
        return image.clone();
    }
    let scale = image.width() as f32 / width as f32;
    let height = ((image.height() as f32 / scale).round() as usize).max(1);
    let mut out = ImageBuffer::filled(width, height, [0.0; 4]);
    for y in 0..height {
        for x in 0..width {
            let (x0, x1) = (x as f32 * scale, (x as f32 + 1.0) * scale);
            let (y0, y1) = (y as f32 * scale, (y as f32 + 1.0) * scale);
            let (xs, xe) = (x0 as isize, (x1.ceil() as isize).max(x0 as isize + 1));
            let (ys, ye) = (y0 as isize, (y1.ceil() as isize).max(y0 as isize + 1));
            let mut sum = [0.0_f32; 4];
            for sy in ys..ye {
                for sx in xs..xe {
                    let px = image.pixel(sx, sy);
                    for (total, value) in sum.iter_mut().zip(px) {
                        *total += value;
                    }
                }
            }
            let count = ((xe - xs) * (ye - ys)) as f32;
            out.set_pixel(x, y, sum.map(|total| total / count));
        }
    }
    out
}

/// The bundled villa placeholder as a linear image of [`PREVIEW_WIDTH`] pixels (a flat grey image if the bundled
/// file cannot be decoded).
pub fn sample_source() -> ImageBuffer {
    let fallback =
        || ImageBuffer::filled(PREVIEW_WIDTH, PREVIEW_WIDTH * 9 / 16, [0.2, 0.2, 0.2, 1.0]);
    let Ok(decoded) = image::load_from_memory_with_format(
        Placeholder::VillaRender.bytes(),
        image::ImageFormat::Png,
    ) else {
        return fallback();
    };
    let rgba = decoded.to_rgba8();
    let size = (rgba.width() as usize, rgba.height() as usize);
    ImageBuffer::from_srgb8(size.0, size.1, rgba.as_raw())
        .map(|full| downscale(&full, PREVIEW_WIDTH))
        .unwrap_or_else(fallback)
}

/// What the camera "sees" of `source`: cropped to the framing, brightened or darkened by the exposure settings
/// against the sample scene, then run through `stack`.
pub fn render_preview(
    source: &ImageBuffer,
    camera: &PhotoCamera,
    stack: &FilterStack,
) -> ImageBuffer {
    let mut image = crop_to_aspect(source, camera.aspect());
    let stops = super::exposure::exposure_stops(
        PREVIEW_SCENE_EV100,
        camera.ev100(),
        camera.exposure.compensation_ev,
    );
    color::exposure(&mut image, stops);
    stack.apply_in_place(&mut image);
    image
}
