//! Per-pixel filters: exposure, contrast, color, tone mapping, LUT and the buffer conversions.

use super::{close, flat, rgb};
use crate::features::camera::logic::filters::color::{self, Lut1d};
use crate::features::camera::logic::filters::tone;
use crate::features::camera::models::image_buffer::{luminance, ImageBuffer};

#[test]
fn srgb_bytes_round_trip_exactly() {
    let bytes: Vec<u8> = (0..=255u8).flat_map(|v| [v, 255 - v, v / 2, v]).collect();
    let image = ImageBuffer::from_srgb8(256, 1, &bytes).expect("size matches");
    assert_eq!(image.to_srgb8(), bytes);
    assert!(ImageBuffer::from_srgb8(3, 3, &bytes).is_none());
    assert!(ImageBuffer::from_rgba(2, 2, vec![0.0; 15]).is_none());
}

#[test]
fn srgb_mid_grey_is_18_percent() {
    let image = ImageBuffer::from_srgb8(1, 1, &[119, 119, 119, 255]).expect("size matches");
    close(image.pixel(0, 0)[0], 0.1845, 1e-3);
}

#[test]
fn exposure_scales_light_by_powers_of_two() {
    let mut image = flat([0.1, 0.2, 0.4]);
    color::exposure(&mut image, 1.0);
    let p = image.pixel(2, 2);
    close(p[0], 0.2, 1e-6);
    close(p[1], 0.4, 1e-6);
    close(p[2], 0.8, 1e-6);
    close(p[3], 0.75, 0.0);
    color::exposure(&mut image, -1.0);
    close(image.pixel(0, 0)[2], 0.4, 1e-6);
}

#[test]
fn contrast_pivots_on_mid_grey() {
    for amount in [-1.0, -0.3, 0.0, 0.5, 1.0] {
        let mut grey = flat([0.18; 3]);
        color::contrast(&mut grey, amount);
        close(rgb(&grey, 0, 0)[1], 0.18, 1e-6);
    }
    let mut image = flat([0.36, 0.09, 0.0]);
    color::contrast(&mut image, 1.0);
    let p = rgb(&image, 1, 1);
    close(p[0], 0.18 * 4.0, 1e-5);
    close(p[1], 0.18 * 0.25, 1e-6);
    assert_eq!(p[2], 0.0);
}

#[test]
fn saturation_zero_is_grey_and_luminance_is_always_kept() {
    let mut grey = flat([0.8, 0.3, 0.1]);
    color::saturation(&mut grey, 0.0);
    let p = rgb(&grey, 0, 0);
    close(p[0], p[1], 1e-6);
    close(p[1], p[2], 1e-6);
    close(p[0], luminance([0.8, 0.3, 0.1]), 1e-6);
    for amount in [0.5, 1.0, 1.7] {
        let mut image = flat([0.8, 0.3, 0.1]);
        color::saturation(&mut image, amount);
        close(
            luminance(rgb(&image, 3, 3)),
            luminance([0.8, 0.3, 0.1]),
            1e-6,
        );
    }
    let mut same = flat([0.8, 0.3, 0.1]);
    color::saturation(&mut same, 1.0);
    let p = rgb(&same, 0, 0);
    close(p[0], 0.8, 1e-6);
    close(p[1], 0.3, 1e-6);
    close(p[2], 0.1, 1e-6);
}

#[test]
fn temperature_warms_and_cools_and_tint_moves_green() {
    let mut warm = flat([0.5; 3]);
    color::temperature_tint(&mut warm, 0.5, 0.0);
    let p = rgb(&warm, 0, 0);
    assert!(p[0] > p[1] && p[1] > p[2]);
    close(p[0] / p[2], 2.0_f32.powf(0.4), 1e-5);
    let mut cool = flat([0.5; 3]);
    color::temperature_tint(&mut cool, -0.5, 0.0);
    assert!(rgb(&cool, 0, 0)[2] > rgb(&cool, 0, 0)[0]);
    let mut magenta = flat([0.5; 3]);
    color::temperature_tint(&mut magenta, 0.0, 1.0);
    assert!(rgb(&magenta, 0, 0)[1] < 0.5);
    let mut neutral = flat([0.5; 3]);
    color::temperature_tint(&mut neutral, 0.0, 0.0);
    assert_eq!(rgb(&neutral, 0, 0), [0.5; 3]);
}

#[test]
fn gamma_lifts_midtones_and_keeps_black_and_white() {
    let mut image = ImageBuffer::from_rgba(
        3,
        1,
        vec![
            0.0, 0.0, 0.0, 1.0, 0.18, 0.18, 0.18, 1.0, 1.0, 1.0, 1.0, 1.0,
        ],
    )
    .expect("size matches");
    color::gamma(&mut image, 2.2);
    assert_eq!(image.pixel(0, 0)[0], 0.0);
    close(image.pixel(1, 0)[0], 0.18_f32.powf(1.0 / 2.2), 1e-6);
    close(image.pixel(2, 0)[0], 1.0, 1e-6);
}

#[test]
fn reinhard_curve_values() {
    let mut image = flat([1.0, 3.0, 0.0]);
    tone::reinhard(&mut image, 0.0);
    let p = rgb(&image, 0, 0);
    close(p[0], 0.5, 1e-6);
    close(p[1], 0.75, 1e-6);
    assert_eq!(p[2], 0.0);
    let mut bright = flat([1000.0; 3]);
    tone::reinhard(&mut bright, 0.0);
    assert!(rgb(&bright, 0, 0)[0] < 1.0);
    let mut lifted = flat([0.5, 0.5, 0.5]);
    tone::reinhard(&mut lifted, 1.0);
    close(rgb(&lifted, 0, 0)[0], 0.5, 1e-6);
}

#[test]
fn aces_curve_values_and_monotonicity() {
    let mut image = flat([0.0, 1.0, 100.0]);
    tone::aces(&mut image, 0.0);
    let p = rgb(&image, 0, 0);
    assert_eq!(p[0], 0.0);
    close(p[1], 2.54 / 3.16, 1e-4);
    assert_eq!(p[2], 1.0);
    let mut last = -1.0;
    for step in 0..200 {
        let mut one = flat([step as f32 * 0.05; 3]);
        tone::aces(&mut one, 0.0);
        let value = rgb(&one, 0, 0)[0];
        assert!(value >= last && (0.0..=1.0).contains(&value));
        last = value;
    }
}

#[test]
fn sepia_and_black_white_keep_luminance() {
    let source = [0.6, 0.4, 0.2];
    let mut tinted = flat(source);
    color::sepia(&mut tinted, 1.0);
    let p = rgb(&tinted, 0, 0);
    assert!(p[0] > p[1] && p[1] > p[2]);
    close(luminance(p), luminance(source), 1e-5);
    let mut mono = flat(source);
    color::black_white(&mut mono, 1.0);
    let p = rgb(&mono, 0, 0);
    close(p[0], p[1], 1e-6);
    close(p[1], p[2], 1e-6);
    close(p[0], luminance(source), 1e-6);
    for filter in [color::sepia, color::black_white] {
        let mut none = flat(source);
        filter(&mut none, 0.0);
        assert_eq!(rgb(&none, 0, 0), source);
    }
}

#[test]
fn lut_identity_changes_nothing_and_s_curve_adds_contrast() {
    let mut image = flat([0.02, 0.18, 0.9]);
    color::lut(&mut image, &Lut1d::identity(256), 1.0);
    let p = rgb(&image, 0, 0);
    close(p[0], 0.02, 1e-3);
    close(p[1], 0.18, 1e-3);
    close(p[2], 0.9, 1e-3);
    let curve = Lut1d::s_curve(64, 1.0);
    assert!(curve.sample(0.25) < 0.25 && curve.sample(0.75) > 0.75);
    close(curve.sample(0.5), 0.5, 1e-3);
    assert_eq!((curve.sample(0.0), curve.sample(1.0)), (0.0, 1.0));
    let mut shadows = flat([0.05; 3]);
    color::lut(&mut shadows, &curve, 1.0);
    assert!(rgb(&shadows, 0, 0)[0] < 0.05);
    let mut half = flat([0.05; 3]);
    color::lut(&mut half, &curve, 0.0);
    assert_eq!(rgb(&half, 0, 0), [0.05; 3]);
}

#[test]
fn lut_interpolates_and_validates_entries() {
    let lut = Lut1d::from_entries(vec![0.0, 1.0, 0.0]).expect("three entries");
    close(lut.sample(0.25), 0.5, 1e-6);
    close(lut.sample(0.75), 0.5, 1e-6);
    close(lut.sample(7.0), 0.0, 1e-6);
    assert!(Lut1d::from_entries(vec![0.5]).is_none());
}
