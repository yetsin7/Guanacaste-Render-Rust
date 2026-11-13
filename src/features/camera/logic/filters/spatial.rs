//! Filters that look at neighbouring pixels: bloom, sharpening and chromatic aberration.

use crate::features::camera::models::image_buffer::ImageBuffer;

/// RGB copy of an image (alpha dropped), three floats per pixel.
fn rgb_of(image: &ImageBuffer) -> Vec<f32> {
    image
        .data()
        .chunks_exact(4)
        .flat_map(|px| [px[0], px[1], px[2]])
        .collect()
}

/// Box blur of one line of `len` values read with `stride`, clamped at the borders, using a running sum.
fn blur_line(data: &mut [f32], start: usize, stride: usize, len: usize, radius: usize) {
    let line: Vec<f32> = (0..len).map(|i| data[start + i * stride]).collect();
    let at = |i: isize| line[i.clamp(0, len as isize - 1) as usize];
    let radius = radius as isize;
    let mut sum: f32 = (-radius..=radius).map(at).sum();
    let window = (2 * radius + 1) as f32;
    for i in 0..len as isize {
        data[start + i as usize * stride] = sum / window;
        sum += at(i + radius + 1) - at(i - radius);
    }
}

/// Separable box blur of an RGB buffer, repeated `passes` times (three passes approximate a Gaussian).
pub fn box_blur_rgb(data: &mut [f32], width: usize, height: usize, radius: usize, passes: usize) {
    if radius == 0 || width == 0 || height == 0 {
        return;
    }
    for _ in 0..passes {
        for y in 0..height {
            for channel in 0..3 {
                blur_line(data, y * width * 3 + channel, 3, width, radius);
            }
        }
        for x in 0..width {
            for channel in 0..3 {
                blur_line(data, x * 3 + channel, width * 3, height, radius);
            }
        }
    }
}

/// Adds a glow: the light above `threshold` (linear) is blurred over `radius` (0..=1, a fraction of 4 % of the
/// longest side) and added back scaled by `intensity`.
pub fn bloom(image: &mut ImageBuffer, threshold: f32, intensity: f32, radius: f32) {
    let (width, height) = (image.width(), image.height());
    let mut glow = rgb_of(image);
    for value in &mut glow {
        *value = (*value - threshold).max(0.0);
    }
    let pixels = (radius.clamp(0.0, 1.0) * 0.04 * width.max(height) as f32).round() as usize;
    box_blur_rgb(&mut glow, width, height, pixels.max(1), 3);
    for (px, glow) in image
        .data_mut()
        .chunks_exact_mut(4)
        .zip(glow.chunks_exact(3))
    {
        for channel in 0..3 {
            px[channel] += intensity * glow[channel];
        }
    }
}

/// Unsharp mask: `x + amount * (x - blur(x))` with a box blur of `radius` pixels.
pub fn sharpen(image: &mut ImageBuffer, amount: f32, radius: f32) {
    let (width, height) = (image.width(), image.height());
    let mut soft = rgb_of(image);
    box_blur_rgb(
        &mut soft,
        width,
        height,
        radius.round().max(1.0) as usize,
        1,
    );
    for (px, soft) in image
        .data_mut()
        .chunks_exact_mut(4)
        .zip(soft.chunks_exact(3))
    {
        for channel in 0..3 {
            px[channel] += amount * (px[channel] - soft[channel]);
        }
    }
}

/// Bilinear sample of one channel at a fractional position, clamped at the borders.
fn sample(image: &ImageBuffer, x: f32, y: f32, channel: usize) -> f32 {
    let (x0, y0) = (x.floor(), y.floor());
    let (tx, ty) = (x - x0, y - y0);
    let at = |dx: isize, dy: isize| image.pixel(x0 as isize + dx, y0 as isize + dy)[channel];
    let top = at(0, 0) * (1.0 - tx) + at(1, 0) * tx;
    let bottom = at(0, 1) * (1.0 - tx) + at(1, 1) * tx;
    top * (1.0 - ty) + bottom * ty
}

/// Lateral chromatic aberration: red is pushed away from the center and blue pulled toward it, by up to
/// `amount` x 1 % of the longest side at the corners. The center and green never move.
pub fn chromatic_aberration(image: &mut ImageBuffer, amount: f32) {
    let source = image.clone();
    let (width, height) = (image.width() as f32, image.height() as f32);
    let (cx, cy) = ((width - 1.0) / 2.0, (height - 1.0) / 2.0);
    let half_diagonal = cx.hypot(cy).max(1e-3);
    let max_shift = amount * 0.01 * width.max(height);
    image.map_rgb_at(|x, y, c| {
        let (dx, dy) = (x as f32 - cx, y as f32 - cy);
        let shift = max_shift * dx.hypot(dy) / half_diagonal;
        let length = dx.hypot(dy).max(1e-3);
        let (ux, uy) = (dx / length * shift, dy / length * shift);
        [
            sample(&source, x as f32 - ux, y as f32 - uy, 0),
            c[1],
            sample(&source, x as f32 + ux, y as f32 + uy, 2),
        ]
    });
}
