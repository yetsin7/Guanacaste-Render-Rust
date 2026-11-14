//! Looking into an equirectangular panorama: the part of the sphere seen by a virtual camera (yaw, pitch, field of
//! view) is resampled into an ordinary picture. Longitude runs across the panorama and latitude down it, with the
//! middle of the picture straight ahead of the camera that took it.

use eframe::egui::{Color32, ColorImage};
use image::RgbaImage;

/// Bilinear sample of `source` at the direction `(x, y, z)` (x right, y ahead, z up).
fn sample(source: &RgbaImage, x: f32, y: f32, z: f32) -> [f32; 3] {
    let (w, h) = (source.width() as f32, source.height() as f32);
    let lon = x.atan2(y);
    let lat = z.clamp(-1.0, 1.0).asin();
    let u = ((lon / std::f32::consts::PI + 1.0) * 0.5 * w).rem_euclid(w);
    let v = ((0.5 - lat / std::f32::consts::PI) * h).clamp(0.0, h - 1.0);
    let (x0, y0) = (u.floor(), v.floor());
    let (fx, fy) = (u - x0, v - y0);
    let at = |px: f32, py: f32| {
        let px = (px as u32) % source.width();
        let py = (py as u32).min(source.height() - 1);
        source.get_pixel(px, py).0
    };
    let (a, b, c, d) = (
        at(x0, y0),
        at(x0 + 1.0, y0),
        at(x0, y0 + 1.0),
        at(x0 + 1.0, y0 + 1.0),
    );
    let mut out = [0.0; 3];
    for k in 0..3 {
        let top = f32::from(a[k]) * (1.0 - fx) + f32::from(b[k]) * fx;
        let bottom = f32::from(c[k]) * (1.0 - fx) + f32::from(d[k]) * fx;
        out[k] = top * (1.0 - fy) + bottom * fy;
    }
    out
}

/// The view of `source` looking `yaw_deg` to the right and `pitch_deg` up, with a horizontal field of view of
/// `fov_deg`, as a picture of `size` pixels.
pub fn reproject(
    source: &RgbaImage,
    yaw_deg: f32,
    pitch_deg: f32,
    fov_deg: f32,
    size: [usize; 2],
) -> ColorImage {
    let (width, height) = (size[0].max(1), size[1].max(1));
    if source.width() < 2 || source.height() < 2 {
        return ColorImage::filled([width, height], Color32::BLACK);
    }
    let half = (fov_deg.clamp(10.0, 150.0).to_radians() * 0.5).tan();
    let (sin_y, cos_y) = yaw_deg.to_radians().sin_cos();
    let (sin_p, cos_p) = pitch_deg.clamp(-89.0, 89.0).to_radians().sin_cos();
    let mut pixels = Vec::with_capacity(width * height);
    for py in 0..height {
        for px in 0..width {
            let sx = (2.0 * (px as f32 + 0.5) / width as f32 - 1.0) * half;
            let sz = (1.0 - 2.0 * (py as f32 + 0.5) / height as f32) * half * height as f32
                / width as f32;
            // Pitch about the X axis, then yaw about Z.
            let (y1, z1) = (cos_p - sz * sin_p, sz * cos_p + sin_p);
            let (x2, y2) = (sx * cos_y + y1 * sin_y, -sx * sin_y + y1 * cos_y);
            let color = sample(source, x2, y2, z1 / (sx * sx + y1 * y1 + z1 * z1).sqrt());
            pixels.push(Color32::from_rgb(
                color[0] as u8,
                color[1] as u8,
                color[2] as u8,
            ));
        }
    }
    ColorImage::new([width, height], pixels)
}

#[cfg(test)]
mod tests {
    use super::*;
    use image::Rgba;

    /// Left half red, right half blue: red is the western hemisphere.
    fn halves() -> RgbaImage {
        RgbaImage::from_fn(64, 32, |x, _| {
            if x < 32 {
                Rgba([255, 0, 0, 255])
            } else {
                Rgba([0, 0, 255, 255])
            }
        })
    }

    #[test]
    fn looking_ahead_sees_the_middle_and_turning_moves_across_the_picture() {
        let source = halves();
        let ahead = reproject(&source, 0.0, 0.0, 60.0, [9, 9]);
        // The seam is straight ahead: the left of the view is red, the right blue.
        assert!(ahead.pixels[4 * 9 + 1].r() > 200);
        assert!(ahead.pixels[4 * 9 + 7].b() > 200);
        let right = reproject(&source, 90.0, 0.0, 60.0, [9, 9]);
        assert!(
            right.pixels[4 * 9 + 4].b() > 200,
            "90 degrees right is blue"
        );
        let left = reproject(&source, -90.0, 0.0, 60.0, [9, 9]);
        assert!(left.pixels[4 * 9 + 4].r() > 200, "90 degrees left is red");
    }

    #[test]
    fn an_empty_source_gives_a_black_picture() {
        let empty = RgbaImage::new(0, 0);
        assert_eq!(reproject(&empty, 0.0, 0.0, 90.0, [4, 3]).pixels.len(), 12);
    }
}
