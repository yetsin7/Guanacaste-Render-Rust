//! Importing a grayscale PNG heightmap: black is the low height, white the high one.

use crate::features::landscape::models::Terrain;

/// Largest heightmap side accepted, in pixels.
pub const MAX_HEIGHTMAP_SIDE: u32 = 8192;

/// A decoded heightmap with values in `0.0..=1.0`, row 0 being the north (+Y) edge.
#[derive(Debug, Clone, PartialEq)]
pub struct HeightImage {
    /// Width in pixels.
    pub width: usize,
    /// Height in pixels.
    pub height: usize,
    /// Normalized values, row by row.
    pub values: Vec<f32>,
}

/// Why a heightmap could not be read.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HeightmapError {
    /// The bytes are not a PNG image that can be decoded.
    NotPng,
    /// The image is empty or larger than [`MAX_HEIGHTMAP_SIDE`].
    BadSize,
}

/// Decodes PNG bytes (8 or 16 bits, gray or color: color is converted to luminance).
pub fn decode_png(bytes: &[u8]) -> Result<HeightImage, HeightmapError> {
    let image = image::load_from_memory_with_format(bytes, image::ImageFormat::Png)
        .map_err(|_| HeightmapError::NotPng)?;
    let (width, height) = (image.width(), image.height());
    if width == 0 || height == 0 || width > MAX_HEIGHTMAP_SIDE || height > MAX_HEIGHTMAP_SIDE {
        return Err(HeightmapError::BadSize);
    }
    let gray = image.into_luma16();
    let values = gray
        .into_raw()
        .into_iter()
        .map(|value| f32::from(value) / f32::from(u16::MAX))
        .collect();
    Ok(HeightImage {
        width: width as usize,
        height: height as usize,
        values,
    })
}

impl HeightImage {
    /// Bilinear sample at `u, v` in `0.0..=1.0` (`v = 0` is the top row of the image).
    pub fn sample(&self, u: f32, v: f32) -> f32 {
        let x = u.clamp(0.0, 1.0) * (self.width - 1) as f32;
        let y = v.clamp(0.0, 1.0) * (self.height - 1) as f32;
        let (x0, y0) = (x.floor() as usize, y.floor() as usize);
        let (x1, y1) = ((x0 + 1).min(self.width - 1), (y0 + 1).min(self.height - 1));
        let (fx, fy) = (x - x0 as f32, y - y0 as f32);
        let at = |px: usize, py: usize| self.values[py * self.width + px];
        let top = at(x0, y0) + (at(x1, y0) - at(x0, y0)) * fx;
        let bottom = at(x0, y1) + (at(x1, y1) - at(x0, y1)) * fx;
        top + (bottom - top) * fy
    }
}

/// Resamples `image` over the whole terrain, mapping black to `low` and white to `high` meters.
/// The top row of the image becomes the north (+Y) edge of the terrain.
pub fn apply_heightmap(terrain: &mut Terrain, image: &HeightImage, low: f32, high: f32) {
    let columns = (terrain.columns - 1).max(1) as f32;
    let rows = (terrain.rows - 1).max(1) as f32;
    for row in 0..terrain.rows {
        for column in 0..terrain.columns {
            let value = image.sample(column as f32 / columns, 1.0 - row as f32 / rows);
            let index = terrain.index(column, row);
            terrain.heights[index] = low + (high - low) * value;
        }
    }
}
