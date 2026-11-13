//! Color temperature conversion.

use crate::features::lighting::logic::{
    kelvin_to_rgb, photometric_color, planckian_xy, MAX_KELVIN, MIN_KELVIN,
};

#[test]
fn chromaticity_matches_published_black_body_values() {
    // CIE 1931 xy of the Planckian locus (Wyszecki & Stiles), tolerance of the cubic approximation.
    for (kelvin, x, y) in [
        (2700.0, 0.4599, 0.4106),
        (4000.0, 0.3805, 0.3768),
        (5000.0, 0.3451, 0.3516),
        (6500.0, 0.3135, 0.3237),
    ] {
        let (cx, cy) = planckian_xy(kelvin);
        assert!((cx - x).abs() < 2e-3, "x at {kelvin}: {cx}");
        assert!((cy - y).abs() < 2e-3, "y at {kelvin}: {cy}");
    }
}

#[test]
fn warm_light_is_orange() {
    let color = kelvin_to_rgb(2700.0);
    assert_eq!(color.r, 1.0);
    assert!((0.35..0.5).contains(&color.g), "g = {}", color.g);
    assert!((0.05..0.16).contains(&color.b), "b = {}", color.b);
}

#[test]
fn daylight_is_nearly_white() {
    let color = kelvin_to_rgb(6500.0);
    for channel in [color.r, color.g, color.b] {
        assert!((0.9..=1.0).contains(&channel), "{color:?}");
    }
}

#[test]
fn blue_share_grows_with_temperature() {
    let mut previous = -1.0;
    for kelvin in [2200.0, 3000.0, 4000.0, 5000.0, 6500.0, 10000.0] {
        let color = kelvin_to_rgb(kelvin);
        let ratio = color.b / color.r;
        assert!(ratio > previous, "{kelvin}: {ratio} <= {previous}");
        previous = ratio;
    }
    assert!(kelvin_to_rgb(20000.0).r < 0.8);
}

#[test]
fn channels_are_normalized_and_input_is_clamped() {
    for kelvin in [100.0, 1667.0, 3200.0, 9000.0, 25000.0, 90000.0] {
        let color = kelvin_to_rgb(kelvin);
        assert!((color.max_channel() - 1.0).abs() < 1e-6, "{kelvin}");
        assert!(color.r >= 0.0 && color.g >= 0.0 && color.b >= 0.0);
    }
    assert_eq!(kelvin_to_rgb(100.0), kelvin_to_rgb(MIN_KELVIN));
    assert_eq!(kelvin_to_rgb(90000.0), kelvin_to_rgb(MAX_KELVIN));
}

#[test]
fn photometric_color_has_unit_luminance() {
    for kelvin in [1800.0, 3000.0, 6500.0, 12000.0] {
        assert!((photometric_color(kelvin).luminance() - 1.0).abs() < 1e-4);
    }
}
