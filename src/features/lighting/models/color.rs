//! Linear RGB triplets used for light colors and contributions.

use serde::{Deserialize, Serialize};
use std::ops::{Add, AddAssign, Mul};

/// A linear (not gamma-encoded) RGB value. Light colors are normalized so the largest channel is 1; contributions
/// carry physical magnitudes (lux per channel).
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
    pub const BLACK: Self = Self::new(0.0, 0.0, 0.0);
    /// White.
    pub const WHITE: Self = Self::new(1.0, 1.0, 1.0);

    /// A color from its three channels.
    pub const fn new(r: f32, g: f32, b: f32) -> Self {
        Self { r, g, b }
    }

    /// The largest channel.
    pub fn max_channel(self) -> f32 {
        self.r.max(self.g).max(self.b)
    }

    /// Photometric luminance (Rec. 709 / sRGB weights).
    pub fn luminance(self) -> f32 {
        0.2126 * self.r + 0.7152 * self.g + 0.0722 * self.b
    }

    /// Channel-wise product.
    pub fn tint(self, other: Self) -> Self {
        Self::new(self.r * other.r, self.g * other.g, self.b * other.b)
    }

    /// True when no channel is positive.
    pub fn is_black(self) -> bool {
        self.max_channel() <= 0.0
    }
}

impl Add for Rgb {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        Self::new(self.r + other.r, self.g + other.g, self.b + other.b)
    }
}

impl AddAssign for Rgb {
    fn add_assign(&mut self, other: Self) {
        *self = *self + other;
    }
}

impl Mul<f32> for Rgb {
    type Output = Self;

    fn mul(self, scale: f32) -> Self {
        Self::new(self.r * scale, self.g * scale, self.b * scale)
    }
}
