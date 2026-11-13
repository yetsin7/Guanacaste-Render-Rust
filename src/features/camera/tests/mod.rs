//! Unit tests of the camera domain.

mod camera_tests;
mod exposure_tests;
mod filter_tests;
mod spatial_tests;
mod stack_tests;
mod ui_tests;

use crate::features::camera::models::image_buffer::ImageBuffer;

/// A 5x5 image of one color.
pub(super) fn flat(rgb: [f32; 3]) -> ImageBuffer {
    ImageBuffer::filled(5, 5, [rgb[0], rgb[1], rgb[2], 0.75])
}

/// Asserts two numbers are within `tolerance`.
pub(super) fn close(a: f32, b: f32, tolerance: f32) {
    assert!(
        (a - b).abs() <= tolerance,
        "{a} is not within {tolerance} of {b}"
    );
}

/// RGB of pixel (`x`, `y`).
pub(super) fn rgb(image: &ImageBuffer, x: isize, y: isize) -> [f32; 3] {
    let p = image.pixel(x, y);
    [p[0], p[1], p[2]]
}
