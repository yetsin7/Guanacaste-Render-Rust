//! Linear RGB color used by the sky and fog functions.

use serde::{Deserialize, Serialize};

/// A linear-light RGB color. Components are not clamped: the sun disc is brighter than 1.
#[derive(Debug, Clone, Copy, PartialEq, Default, Serialize, Deserialize)]
pub struct Rgb {
    /// Red.
    pub r: f32,
    /// Green.
    pub g: f32,
    /// Blue.
    pub b: f32,
}

impl Rgb {
    /// Black.
    pub const BLACK: Rgb = Rgb::splat(0.0);

    /// A color from linear components.
    pub const fn new(r: f32, g: f32, b: f32) -> Self {
        Self { r, g, b }
    }

    /// A gray with the same value in every channel.
    pub const fn splat(value: f32) -> Self {
        Self::new(value, value, value)
    }

    /// A color from 8-bit sRGB components (as written in a color picker).
    pub fn from_srgb8(r: u8, g: u8, b: u8) -> Self {
        let linear = |v: u8| {
            let c = f32::from(v) / 255.0;
            if c <= 0.04045 {
                c / 12.92
            } else {
                ((c + 0.055) / 1.055).powf(2.4)
            }
        };
        Self::new(linear(r), linear(g), linear(b))
    }

    /// Converts to 8-bit sRGB, clamping to the displayable range.
    pub fn to_srgb8(self) -> [u8; 3] {
        let encode = |c: f32| {
            let c = c.clamp(0.0, 1.0);
            let s = if c <= 0.003_130_8 {
                c * 12.92
            } else {
                1.055 * c.powf(1.0 / 2.4) - 0.055
            };
            (s * 255.0 + 0.5) as u8
        };
        [encode(self.r), encode(self.g), encode(self.b)]
    }

    /// Relative luminance.
    pub fn luma(self) -> f32 {
        0.2126 * self.r + 0.7152 * self.g + 0.0722 * self.b
    }

    /// Linear interpolation: `self` at 0, `other` at 1.
    pub fn mix(self, other: Rgb, t: f32) -> Rgb {
        self + (other - self) * t
    }

    /// Largest channel.
    pub fn max_channel(self) -> f32 {
        self.r.max(self.g).max(self.b)
    }
}

impl std::ops::Add for Rgb {
    type Output = Rgb;
    fn add(self, other: Rgb) -> Rgb {
        Rgb::new(self.r + other.r, self.g + other.g, self.b + other.b)
    }
}

impl std::ops::Sub for Rgb {
    type Output = Rgb;
    fn sub(self, other: Rgb) -> Rgb {
        Rgb::new(self.r - other.r, self.g - other.g, self.b - other.b)
    }
}

impl std::ops::Mul<f32> for Rgb {
    type Output = Rgb;
    fn mul(self, k: f32) -> Rgb {
        Rgb::new(self.r * k, self.g * k, self.b * k)
    }
}

impl std::ops::Mul for Rgb {
    type Output = Rgb;
    fn mul(self, other: Rgb) -> Rgb {
        Rgb::new(self.r * other.r, self.g * other.g, self.b * other.b)
    }
}
