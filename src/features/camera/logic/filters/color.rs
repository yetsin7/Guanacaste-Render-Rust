//! Per-pixel color filters. Every function edits the RGB of a linear-light [`ImageBuffer`] in place and leaves
//! alpha alone; see the buffer's documentation for the color contract.

use crate::features::camera::models::image_buffer::{
    linear_to_srgb, luminance, srgb_to_linear, ImageBuffer,
};

/// Linear mid grey, the pivot of [`contrast`].
pub const MID_GREY: f32 = 0.18;

/// Multiplies the light by `2^stops`.
pub fn exposure(image: &mut ImageBuffer, stops: f32) {
    let gain = 2.0_f32.powf(stops);
    image.map_rgb(|c| [c[0] * gain, c[1] * gain, c[2] * gain]);
}

/// Power-law contrast around mid grey: each channel becomes `0.18 * (x / 0.18)^(2^amount)`. `amount` 0 is the
/// identity, positive values spread tones away from grey, negative ones compress them.
pub fn contrast(image: &mut ImageBuffer, amount: f32) {
    let power = 2.0_f32.powf(amount);
    let curve = |x: f32| {
        if x <= 0.0 {
            0.0
        } else {
            MID_GREY * (x / MID_GREY).powf(power)
        }
    };
    image.map_rgb(|c| [curve(c[0]), curve(c[1]), curve(c[2])]);
}

/// Scales the distance of every color from its luminance: 1 is the identity, 0 is grey, 2 is double.
pub fn saturation(image: &mut ImageBuffer, amount: f32) {
    image.map_rgb(|c| {
        let y = luminance(c);
        [
            y + (c[0] - y) * amount,
            y + (c[1] - y) * amount,
            y + (c[2] - y) * amount,
        ]
    });
}

/// White balance shift. `temperature` in -1..=1 goes from cool (blue) to warm (orange); `tint` in -1..=1 goes
/// from green to magenta. Both are in stops-like exponents, so 0/0 is the identity and neutrals stay neutral
/// only at 0.
pub fn temperature_tint(image: &mut ImageBuffer, temperature: f32, tint: f32) {
    let red = 2.0_f32.powf(0.4 * temperature);
    let blue = 2.0_f32.powf(-0.4 * temperature);
    let green = 2.0_f32.powf(-0.3 * tint);
    image.map_rgb(|c| [c[0] * red, c[1] * green, c[2] * blue]);
}

/// Display gamma adjustment: `x^(1/gamma)`. Values above 1 brighten the midtones.
pub fn gamma(image: &mut ImageBuffer, gamma: f32) {
    let inverse = 1.0 / gamma.max(1e-3);
    image.map_rgb(|c| {
        [
            c[0].max(0.0).powf(inverse),
            c[1].max(0.0).powf(inverse),
            c[2].max(0.0).powf(inverse),
        ]
    });
}

/// Blends the image toward its brown-tinted luminance (the sepia tone) by `amount` (0..=1). Luminance is
/// preserved.
pub fn sepia(image: &mut ImageBuffer, amount: f32) {
    let tone = [1.0, 0.78, 0.52];
    let scale = 1.0 / luminance(tone);
    image.map_rgb(|c| {
        let y = luminance(c);
        let tinted = [
            y * tone[0] * scale,
            y * tone[1] * scale,
            y * tone[2] * scale,
        ];
        mix(c, tinted, amount)
    });
}

/// Blends the image toward its luminance (black and white) by `amount` (0..=1).
pub fn black_white(image: &mut ImageBuffer, amount: f32) {
    image.map_rgb(|c| {
        let y = luminance(c);
        mix(c, [y, y, y], amount)
    });
}

/// A one-dimensional look-up table applied to every channel of the display-referred (sRGB encoded) image.
#[derive(Debug, Clone, PartialEq)]
pub struct Lut1d {
    entries: Vec<f32>,
}

impl Lut1d {
    /// A table of `size` (at least 2) entries that maps every value to itself.
    pub fn identity(size: usize) -> Self {
        let size = size.max(2);
        let entries = (0..size).map(|i| i as f32 / (size - 1) as f32).collect();
        Self { entries }
    }

    /// An S-shaped curve: `strength` 0 is the identity, 1 is a full smoothstep (deep blacks, bright highlights).
    pub fn s_curve(size: usize, strength: f32) -> Self {
        let mut lut = Self::identity(size);
        for x in &mut lut.entries {
            *x += strength * (*x * *x * (3.0 - 2.0 * *x) - *x);
        }
        lut
    }

    /// Wraps explicit entries (at least 2), each expected in 0..=1.
    pub fn from_entries(entries: Vec<f32>) -> Option<Self> {
        (entries.len() >= 2).then_some(Self { entries })
    }

    /// The table value at `x` (0..=1), interpolated linearly between entries.
    pub fn sample(&self, x: f32) -> f32 {
        let position = x.clamp(0.0, 1.0) * (self.entries.len() - 1) as f32;
        let low = position.floor() as usize;
        let high = (low + 1).min(self.entries.len() - 1);
        let t = position - low as f32;
        self.entries[low] + (self.entries[high] - self.entries[low]) * t
    }
}

/// Applies `lut` to the sRGB-encoded channels and blends the result with the original by `amount` (0..=1).
pub fn lut(image: &mut ImageBuffer, lut: &Lut1d, amount: f32) {
    let apply = |v: f32| srgb_to_linear(lut.sample(linear_to_srgb(v)));
    image.map_rgb(|c| mix(c, [apply(c[0]), apply(c[1]), apply(c[2])], amount));
}

/// Linear blend `a + (b - a) * t` of two colors.
pub fn mix(a: [f32; 3], b: [f32; 3], t: f32) -> [f32; 3] {
    [
        a[0] + (b[0] - a[0]) * t,
        a[1] + (b[1] - a[1]) * t,
        a[2] + (b[2] - a[2]) * t,
    ]
}
