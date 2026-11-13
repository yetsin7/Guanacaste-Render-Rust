//! Tone mapping operators: compress linear HDR light into the 0..=1 range a screen can show. Run them last in a
//! stack, after exposure and color grading.

use crate::features::camera::models::image_buffer::ImageBuffer;

/// Reinhard: `x / (1 + x)` per channel after multiplying by `2^stops`.
pub fn reinhard(image: &mut ImageBuffer, stops: f32) {
    let gain = 2.0_f32.powf(stops);
    image.map_rgb(|c| {
        let map = |x: f32| {
            let x = (x * gain).max(0.0);
            x / (1.0 + x)
        };
        [map(c[0]), map(c[1]), map(c[2])]
    });
}

/// ACES filmic curve (Narkowicz fit) per channel after multiplying by `2^stops`; the result is clamped to 0..=1.
pub fn aces(image: &mut ImageBuffer, stops: f32) {
    let gain = 2.0_f32.powf(stops);
    image.map_rgb(|c| {
        let map = |x: f32| {
            let x = (x * gain).max(0.0);
            ((x * (2.51 * x + 0.03)) / (x * (2.43 * x + 0.59) + 0.14)).clamp(0.0, 1.0)
        };
        [map(c[0]), map(c[1]), map(c[2])]
    });
}
