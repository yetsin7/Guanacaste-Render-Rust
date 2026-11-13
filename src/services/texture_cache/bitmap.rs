//! CPU images and their mipmap chains.

/// An 8-bit RGBA image, row-major, top row first.
#[derive(Debug, Clone, PartialEq)]
pub struct Bitmap {
    /// Width in pixels (at least 1).
    pub width: usize,
    /// Height in pixels (at least 1).
    pub height: usize,
    /// `width * height` pixels.
    pub pixels: Vec<[u8; 4]>,
}

impl Bitmap {
    /// A bitmap of one color.
    pub fn filled(width: usize, height: usize, color: [u8; 4]) -> Self {
        let (width, height) = (width.max(1), height.max(1));
        Self {
            width,
            height,
            pixels: vec![color; width * height],
        }
    }

    /// Pixel at `(x, y)`; coordinates must be inside the image.
    pub fn at(&self, x: usize, y: usize) -> [u8; 4] {
        self.pixels[y * self.width + x]
    }

    /// Memory held by the pixels, in bytes.
    pub fn bytes(&self) -> usize {
        self.pixels.len() * 4
    }
}

/// How the color channels of a texture are stored, which decides how mip levels are averaged.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Usage {
    /// Colors in sRGB (albedo, emission): averaged in linear light.
    Color,
    /// Linear data (roughness, metal, AO, normal maps).
    Data,
    /// A grayscale height map that is converted to a tangent-space normal map on load.
    Height,
}

/// A full mipmap chain: level 0 is the original, each next level halves both sides down to 1x1.
#[derive(Debug, Clone, PartialEq)]
pub struct MipChain {
    /// Levels, largest first.
    pub levels: Vec<Bitmap>,
}

impl MipChain {
    /// Builds the chain of `base`; sRGB images are averaged in linear light when `srgb` is set.
    pub fn build(base: Bitmap, srgb: bool) -> Self {
        let mut levels = vec![base];
        while let Some(last) = levels.last().filter(|l| l.width > 1 || l.height > 1) {
            let next = halve(last, srgb);
            levels.push(next);
        }
        Self { levels }
    }

    /// Memory held by every level, in bytes.
    pub fn bytes(&self) -> usize {
        self.levels.iter().map(Bitmap::bytes).sum()
    }

    /// Size of the largest level.
    pub fn size(&self) -> (usize, usize) {
        (self.levels[0].width, self.levels[0].height)
    }

    /// Drops the top levels until the largest side is at most `max_side` (never the last level).
    pub fn limit_size(&mut self, max_side: usize) {
        while self.levels.len() > 1 && self.levels[0].width.max(self.levels[0].height) > max_side {
            self.levels.remove(0);
        }
    }
}

fn to_linear(value: u8) -> f32 {
    let c = value as f32 / 255.0;
    if c <= 0.04045 {
        c / 12.92
    } else {
        ((c + 0.055) / 1.055).powf(2.4)
    }
}

fn to_srgb(linear: f32) -> u8 {
    let c = linear.clamp(0.0, 1.0);
    let encoded = if c <= 0.003_130_8 {
        c * 12.92
    } else {
        1.055 * c.powf(1.0 / 2.4) - 0.055
    };
    (encoded * 255.0 + 0.5) as u8
}

/// One box-filter reduction step (odd sides reuse the last row/column).
fn halve(source: &Bitmap, srgb: bool) -> Bitmap {
    let width = (source.width / 2).max(1);
    let height = (source.height / 2).max(1);
    let mut pixels = Vec::with_capacity(width * height);
    for y in 0..height {
        for x in 0..width {
            let xs = [
                (x * 2).min(source.width - 1),
                (x * 2 + 1).min(source.width - 1),
            ];
            let ys = [
                (y * 2).min(source.height - 1),
                (y * 2 + 1).min(source.height - 1),
            ];
            let mut sum = [0.0_f32; 4];
            for &sy in &ys {
                for &sx in &xs {
                    let p = source.at(sx, sy);
                    for c in 0..3 {
                        sum[c] += if srgb { to_linear(p[c]) } else { p[c] as f32 };
                    }
                    sum[3] += p[3] as f32;
                }
            }
            let mut out = [0_u8; 4];
            for c in 0..3 {
                out[c] = if srgb {
                    to_srgb(sum[c] / 4.0)
                } else {
                    (sum[c] / 4.0 + 0.5) as u8
                };
            }
            out[3] = (sum[3] / 4.0 + 0.5) as u8;
            pixels.push(out);
        }
    }
    Bitmap {
        width,
        height,
        pixels,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn chain_halves_down_to_one_pixel() {
        let chain = MipChain::build(Bitmap::filled(8, 4, [10, 20, 30, 255]), false);
        let sizes: Vec<_> = chain.levels.iter().map(|l| (l.width, l.height)).collect();
        assert_eq!(sizes, [(8, 4), (4, 2), (2, 1), (1, 1)]);
        assert_eq!(chain.levels[3].at(0, 0), [10, 20, 30, 255]);
        assert_eq!(chain.bytes(), (32 + 8 + 2 + 1) * 4);
    }

    #[test]
    fn non_power_of_two_sides_still_reach_one_by_one() {
        let chain = MipChain::build(Bitmap::filled(5, 3, [0; 4]), true);
        let last = chain.levels.last().map(|l| (l.width, l.height));
        assert_eq!(last, Some((1, 1)));
    }

    #[test]
    fn srgb_mips_average_in_linear_light() {
        let mut base = Bitmap::filled(2, 1, [0, 0, 0, 255]);
        base.pixels[1] = [255, 255, 255, 255];
        let linear = MipChain::build(base.clone(), true);
        let naive = MipChain::build(base, false);
        let lit = linear.levels.last().unwrap().at(0, 0)[0];
        assert!(lit > 180 && lit < 195, "{lit}"); // sRGB of 0.5 linear is about 188
        assert_eq!(naive.levels.last().unwrap().at(0, 0)[0], 128);
    }

    #[test]
    fn limit_size_drops_top_levels_but_keeps_one() {
        let mut chain = MipChain::build(Bitmap::filled(64, 64, [1; 4]), false);
        chain.limit_size(16);
        assert_eq!(chain.size(), (16, 16));
        chain.limit_size(0);
        assert_eq!(chain.size(), (1, 1));
    }
}
