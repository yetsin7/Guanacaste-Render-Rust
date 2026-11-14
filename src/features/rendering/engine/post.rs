//! What happens to the traced picture: noise removal, camera exposure, depth of field, tone mapping and the
//! filter stack of the camera domain, ending in an sRGB picture.

use super::buffers::FrameBuffers;
use super::denoise::denoise;
use super::parallel::for_each_row;
use super::sampling::aces;
use super::types::{PostSettings, RenderRequest};
use crate::features::camera::models::camera::PhotoCamera;
use crate::features::camera::models::exposure::PREVIEW_SCENE_EV100;
use crate::features::camera::models::filter::FilterKind;
use crate::features::camera::models::image_buffer::ImageBuffer;
use image::RgbaImage;

/// Largest blur radius in pixels; keeps the cost of the lens blur bounded.
const MAX_BLUR_RADIUS: f32 = 24.0;
/// Samples taken on the blur disc.
const BLUR_SAMPLES: usize = 32;

/// Brightness multiplier the camera exposure gives: 1 for the default camera settings.
pub fn exposure_gain(post: &PostSettings) -> f32 {
    post.photo.map_or(1.0, |photo| {
        photo
            .exposure
            .gain(photo.lens.aperture, PREVIEW_SCENE_EV100)
    })
}

/// Mid-gray the automatic exposure aims for (linear, before tone mapping).
const AUTO_KEY: f32 = 0.22;
/// The most the automatic exposure brightens a picture (about 3.5 stops).
const AUTO_MAX_GAIN: f32 = 20.0;

/// The gain that brings the geometric mean brightness of the surfaces (not the sky) to [`AUTO_KEY`], between 1 and
/// [`AUTO_MAX_GAIN`]. Bright pictures are left alone.
fn auto_gain(frame: &FrameBuffers) -> f32 {
    let (mut log_sum, mut count) = (0.0f64, 0usize);
    for (px, normal) in frame
        .color
        .chunks_exact(4)
        .zip(frame.normal.chunks_exact(3))
    {
        if normal == [0.0, 0.0, 0.0] {
            continue;
        }
        let luma = 0.2126 * px[0] + 0.7152 * px[1] + 0.0722 * px[2];
        log_sum += f64::from(luma.max(1e-4)).ln();
        count += 1;
    }
    if count == 0 {
        return 1.0;
    }
    let mean = (log_sum / count as f64).exp() as f32;
    (AUTO_KEY / mean).clamp(1.0, AUTO_MAX_GAIN)
}

/// Blurs pixels by their distance from the focus plane: each pixel gathers the neighbors whose own blur disc
/// reaches it, so sharp foreground objects do not smear into a blurred background.
fn depth_of_field(frame: &mut FrameBuffers, photo: &PhotoCamera) {
    let (w, h) = (frame.width, frame.height);
    let radius: Vec<f32> = frame
        .depth
        .iter()
        .map(|&d| (photo.blur_diameter_px(d.max(0.01), w as f32) * 0.5).clamp(0.0, MAX_BLUR_RADIUS))
        .collect();
    let source = frame.color.clone();
    let radius = &radius;
    for_each_row(&mut frame.color, w * 4, |y, row| {
        for x in 0..w {
            let r = radius[y * w + x];
            if r < 0.6 {
                continue;
            }
            let at = (y * w + x) * 4;
            let mut sum = [source[at], source[at + 1], source[at + 2]];
            let mut count = 1.0f32;
            for k in 0..BLUR_SAMPLES {
                let distance = r * ((k as f32 + 0.5) / BLUR_SAMPLES as f32).sqrt();
                let angle = k as f32 * 2.399_963;
                let sx = (x as f32 + distance * angle.cos()).round();
                let sy = (y as f32 + distance * angle.sin()).round();
                if sx < 0.0 || sy < 0.0 || sx >= w as f32 || sy >= h as f32 {
                    continue;
                }
                let q = sy as usize * w + sx as usize;
                if radius[q] + 0.5 < distance {
                    continue;
                }
                sum[0] += source[q * 4];
                sum[1] += source[q * 4 + 1];
                sum[2] += source[q * 4 + 2];
                count += 1.0;
            }
            row[x * 4..x * 4 + 3].copy_from_slice(&[
                sum[0] / count,
                sum[1] / count,
                sum[2] / count,
            ]);
        }
    });
}

/// Finishes a traced picture: denoise, exposure, depth of field, tone mapping (unless a filter does it), the
/// camera filters and the sRGB encoding.
pub fn finish(mut frame: FrameBuffers, request: &RenderRequest) -> RgbaImage {
    let post = &request.post;
    if post.denoise {
        denoise(&mut frame);
    }
    let mut gain = exposure_gain(post);
    if post.auto_exposure {
        gain *= auto_gain(&frame);
    }
    if (gain - 1.0).abs() > 1e-6 {
        for px in frame.color.chunks_exact_mut(4) {
            for c in &mut px[..3] {
                *c *= gain;
            }
        }
    }
    if let (Some(photo), true) = (&post.photo, post.depth_of_field) {
        depth_of_field(&mut frame, photo);
    }
    let tone_mapped_by_filter = post
        .filters
        .entries
        .iter()
        .any(|e| e.enabled && matches!(e.filter.kind, FilterKind::Aces | FilterKind::Reinhard));
    if !tone_mapped_by_filter {
        for px in frame.color.chunks_exact_mut(4) {
            for c in &mut px[..3] {
                *c = aces(*c);
            }
        }
    }
    let (w, h) = (frame.width, frame.height);
    let mut buffer = ImageBuffer::from_rgba(w, h, frame.color)
        .unwrap_or_else(|| ImageBuffer::filled(w, h, [0.0; 4]));
    post.filters.apply_in_place(&mut buffer);
    RgbaImage::from_raw(w as u32, h as u32, buffer.to_srgb8())
        .unwrap_or_else(|| RgbaImage::new(w as u32, h as u32))
}
