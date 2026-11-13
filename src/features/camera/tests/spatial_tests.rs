//! Filters that depend on position or neighbours: vignette, grain, bloom, sharpen, chromatic aberration.

use super::{close, flat, rgb};
use crate::features::camera::logic::filters::{effects, spatial};
use crate::features::camera::models::image_buffer::ImageBuffer;

#[test]
fn vignette_spares_the_center_and_darkens_the_corners() {
    let mut image = flat([0.5; 3]);
    effects::vignette(&mut image, 0.6, 0.5, 0.5);
    assert_eq!(rgb(&image, 2, 2), [0.5; 3]);
    close(rgb(&image, 0, 0)[0], 0.5 * 0.4, 1e-5);
    close(rgb(&image, 4, 4)[0], 0.5 * 0.4, 1e-5);
    assert_eq!(image.pixel(0, 0)[3], 0.75);
}

#[test]
fn vignette_darkens_monotonically_outward() {
    let mut image = ImageBuffer::filled(33, 33, [1.0; 4]);
    effects::vignette(&mut image, 0.8, 0.2, 0.8);
    let mut last = 1.0;
    for x in 16..33 {
        let value = rgb(&image, x, 16)[0];
        assert!(value <= last + 1e-6);
        last = value;
    }
    assert!(last < 1.0);
    let mut none = ImageBuffer::filled(9, 9, [1.0; 4]);
    effects::vignette(&mut none, 0.0, 0.5, 0.5);
    assert_eq!(rgb(&none, 0, 0), [1.0; 3]);
}

#[test]
fn grain_is_deterministic_bounded_and_zero_mean() {
    let base = ImageBuffer::filled(64, 64, [0.5, 0.5, 0.5, 1.0]);
    let mut a = base.clone();
    let mut b = base.clone();
    effects::grain(&mut a, 0.5, 42);
    effects::grain(&mut b, 0.5, 42);
    assert_eq!(a, b);
    let mut other = base.clone();
    effects::grain(&mut other, 0.5, 43);
    assert_ne!(a, other);
    let bound = 0.5 * 0.2 * (0.5_f32 + 0.05).sqrt() + 1e-6;
    let mut sum = 0.0;
    for y in 0..64 {
        for x in 0..64 {
            let p = rgb(&a, x, y);
            assert!(p[0] == p[1] && p[1] == p[2], "grain is monochrome");
            assert!((p[0] - 0.5).abs() <= bound);
            sum += p[0] - 0.5;
        }
    }
    assert!((sum / 4096.0).abs() < 0.003, "mean offset {}", sum / 4096.0);
    let mut off = base.clone();
    effects::grain(&mut off, 0.0, 1);
    assert_eq!(off, base);
}

#[test]
fn bloom_spreads_only_light_above_the_threshold() {
    let mut image = ImageBuffer::filled(21, 21, [0.0, 0.0, 0.0, 1.0]);
    image.set_pixel(10, 10, [5.0, 5.0, 5.0, 1.0]);
    let before: f32 = image.data().iter().step_by(4).sum();
    spatial::bloom(&mut image, 1.0, 0.5, 0.1);
    assert!(
        rgb(&image, 11, 10)[0] > 0.0,
        "light leaks to the neighbours"
    );
    assert!(rgb(&image, 10, 10)[0] > 5.0, "the source stays bright");
    let after: f32 = image.data().iter().step_by(4).sum();
    close(after - before, 0.5 * 4.0, 1e-3);
    assert_eq!(image.pixel(10, 10)[3], 1.0);
}

#[test]
fn bloom_does_nothing_below_the_threshold_or_at_zero_intensity() {
    let mut dim = flat([0.8; 3]);
    spatial::bloom(&mut dim, 1.0, 1.0, 0.5);
    assert_eq!(rgb(&dim, 2, 2), [0.8; 3]);
    let mut off = ImageBuffer::filled(9, 9, [3.0; 4]);
    spatial::bloom(&mut off, 1.0, 0.0, 0.5);
    assert_eq!(rgb(&off, 4, 4), [3.0; 3]);
}

#[test]
fn sharpen_keeps_flat_areas_and_overshoots_edges() {
    let mut flat_image = flat([0.4; 3]);
    spatial::sharpen(&mut flat_image, 1.0, 1.0);
    assert_eq!(rgb(&flat_image, 2, 2), [0.4; 3]);
    let mut step = ImageBuffer::filled(12, 3, [0.2, 0.2, 0.2, 1.0]);
    for y in 0..3 {
        for x in 6..12 {
            step.set_pixel(x, y, [0.8, 0.8, 0.8, 1.0]);
        }
    }
    spatial::sharpen(&mut step, 1.0, 1.0);
    assert!(rgb(&step, 5, 1)[0] < 0.2, "dark side undershoots");
    assert!(rgb(&step, 6, 1)[0] > 0.8, "bright side overshoots");
    close(rgb(&step, 0, 1)[0], 0.2, 1e-6);
    close(rgb(&step, 11, 1)[0], 0.8, 1e-6);
}

#[test]
fn box_blur_preserves_the_mean_of_a_constant_and_smooths_a_spike() {
    let mut data = vec![0.0_f32; 9 * 9 * 3];
    data[(4 * 9 + 4) * 3] = 9.0;
    spatial::box_blur_rgb(&mut data, 9, 9, 1, 1);
    close(data[(4 * 9 + 4) * 3], 1.0, 1e-6);
    close(data[(4 * 9 + 5) * 3], 1.0, 1e-6);
    assert_eq!(data[(4 * 9 + 7) * 3], 0.0);
    let total: f32 = data.iter().step_by(3).sum();
    close(total, 9.0, 1e-4);
}

#[test]
fn chromatic_aberration_leaves_center_and_green_alone() {
    let mut ramp = ImageBuffer::filled(11, 11, [0.0; 4]);
    for y in 0..11 {
        for x in 0..11 {
            let v = x as f32 / 10.0;
            ramp.set_pixel(x, y, [v, v, v, 1.0]);
        }
    }
    let original = ramp.clone();
    spatial::chromatic_aberration(&mut ramp, 0.0);
    assert_eq!(ramp, original);
    spatial::chromatic_aberration(&mut ramp, 1.0);
    assert_eq!(rgb(&ramp, 5, 5), rgb(&original, 5, 5));
    for (x, y) in [(0, 0), (10, 3), (2, 10), (9, 9)] {
        assert_eq!(rgb(&ramp, x, y)[1], rgb(&original, x, y)[1]);
    }
    let right = rgb(&ramp, 10, 5);
    assert!(
        right[0] < right[1],
        "red is pulled inward on the right edge: {right:?}"
    );
    let left = rgb(&ramp, 0, 5);
    assert!(left[0] > left[1], "and on the left edge: {left:?}");
}
