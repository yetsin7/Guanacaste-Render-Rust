//! Unit tests of the rendering vocabulary.

use super::models::*;

#[test]
fn quality_cycles_through_four_levels() {
    let mut quality = RayTracingQuality::Low;
    let mut seen = Vec::new();
    for _ in 0..4 {
        seen.push(quality.samples());
        quality = quality.next();
    }
    assert_eq!(seen, vec![2, 4, 8, 16]);
    assert_eq!(quality, RayTracingQuality::Low);
}

#[test]
fn upscale_cycles_back_to_off() {
    let mut mode = UpscaleMode::Off;
    for _ in 0..4 {
        mode = mode.next();
    }
    assert_eq!(mode, UpscaleMode::Off);
}
