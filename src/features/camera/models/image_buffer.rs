//! A small floating point image used by the camera filters and shared with the render agent.
//!
//! # Contract
//!
//! * Pixels are stored row by row (top row first) as interleaved `[r, g, b, a]` `f32` values.
//! * Color channels are **linear light**: `1.0` is diffuse white, `0.18` is mid grey and values above `1.0` are
//!   allowed (HDR). A renderer can hand its radiance buffer over unchanged; filters never clamp except where a
//!   curve says so. Alpha is straight (not premultiplied) and is never changed by a filter.
//! * 8-bit images are converted with [`ImageBuffer::from_srgb8`] and [`ImageBuffer::to_srgb8`], which apply the
//!   sRGB transfer curve, so a file decoded, left untouched by the filters and encoded again is bit exact.

/// Rec. 709 luminance of a linear RGB triple.
pub fn luminance(rgb: [f32; 3]) -> f32 {
    0.2126 * rgb[0] + 0.7152 * rgb[1] + 0.0722 * rgb[2]
}

/// sRGB-encoded value (0..1) to linear light.
pub fn srgb_to_linear(value: f32) -> f32 {
    if value <= 0.04045 {
        value / 12.92
    } else {
        ((value + 0.055) / 1.055).powf(2.4)
    }
}

/// Linear light to an sRGB-encoded value; the input is clamped to 0..1.
pub fn linear_to_srgb(value: f32) -> f32 {
    let value = value.clamp(0.0, 1.0);
    if value <= 0.003_130_8 {
        value * 12.92
    } else {
        1.055 * value.powf(1.0 / 2.4) - 0.055
    }
}

/// A linear-light RGBA `f32` image.
#[derive(Debug, Clone, PartialEq)]
pub struct ImageBuffer {
    width: usize,
    height: usize,
    data: Vec<f32>,
}

impl ImageBuffer {
    /// An image of `width` x `height` pixels, all set to `rgba`.
    pub fn filled(width: usize, height: usize, rgba: [f32; 4]) -> Self {
        let mut data = Vec::with_capacity(width * height * 4);
        for _ in 0..width * height {
            data.extend_from_slice(&rgba);
        }
        Self {
            width,
            height,
            data,
        }
    }

    /// Wraps interleaved linear RGBA data; `None` when the length is not `width * height * 4`.
    pub fn from_rgba(width: usize, height: usize, data: Vec<f32>) -> Option<Self> {
        (data.len() == width * height * 4).then_some(Self {
            width,
            height,
            data,
        })
    }

    /// Decodes interleaved sRGB RGBA bytes (alpha stays linear); `None` when the length does not match.
    pub fn from_srgb8(width: usize, height: usize, bytes: &[u8]) -> Option<Self> {
        if bytes.len() != width * height * 4 {
            return None;
        }
        let data = bytes
            .chunks_exact(4)
            .flat_map(|px| {
                [
                    srgb_to_linear(f32::from(px[0]) / 255.0),
                    srgb_to_linear(f32::from(px[1]) / 255.0),
                    srgb_to_linear(f32::from(px[2]) / 255.0),
                    f32::from(px[3]) / 255.0,
                ]
            })
            .collect();
        Some(Self {
            width,
            height,
            data,
        })
    }

    /// Encodes to interleaved sRGB RGBA bytes, clamping to the displayable range.
    pub fn to_srgb8(&self) -> Vec<u8> {
        let byte = |v: f32| (v.clamp(0.0, 1.0) * 255.0).round() as u8;
        self.data
            .chunks_exact(4)
            .flat_map(|px| {
                [
                    byte(linear_to_srgb(px[0])),
                    byte(linear_to_srgb(px[1])),
                    byte(linear_to_srgb(px[2])),
                    byte(px[3]),
                ]
            })
            .collect()
    }

    /// Width in pixels.
    pub fn width(&self) -> usize {
        self.width
    }

    /// Height in pixels.
    pub fn height(&self) -> usize {
        self.height
    }

    /// The interleaved RGBA data.
    pub fn data(&self) -> &[f32] {
        &self.data
    }

    /// The interleaved RGBA data, mutable.
    pub fn data_mut(&mut self) -> &mut [f32] {
        &mut self.data
    }

    /// The pixel at (`x`, `y`); coordinates outside the image are clamped to the border.
    pub fn pixel(&self, x: isize, y: isize) -> [f32; 4] {
        let x = x.clamp(0, self.width as isize - 1) as usize;
        let y = y.clamp(0, self.height as isize - 1) as usize;
        let at = (y * self.width + x) * 4;
        [
            self.data[at],
            self.data[at + 1],
            self.data[at + 2],
            self.data[at + 3],
        ]
    }

    /// Sets the pixel at (`x`, `y`); out-of-range coordinates are ignored.
    pub fn set_pixel(&mut self, x: usize, y: usize, rgba: [f32; 4]) {
        if x < self.width && y < self.height {
            let at = (y * self.width + x) * 4;
            self.data[at..at + 4].copy_from_slice(&rgba);
        }
    }

    /// Applies `f` to the RGB of every pixel, keeping alpha.
    pub fn map_rgb(&mut self, f: impl Fn([f32; 3]) -> [f32; 3]) {
        for px in self.data.chunks_exact_mut(4) {
            let out = f([px[0], px[1], px[2]]);
            px[..3].copy_from_slice(&out);
        }
    }

    /// Applies `f(x, y, rgb)` to every pixel, keeping alpha.
    pub fn map_rgb_at(&mut self, f: impl Fn(usize, usize, [f32; 3]) -> [f32; 3]) {
        let width = self.width.max(1);
        for (index, px) in self.data.chunks_exact_mut(4).enumerate() {
            let out = f(index % width, index / width, [px[0], px[1], px[2]]);
            px[..3].copy_from_slice(&out);
        }
    }
}
