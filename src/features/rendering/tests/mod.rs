//! Unit tests of the rendering domain.

use super::logic::*;
use super::models::*;
use crate::services::sample_data::rendering as sample;

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

#[test]
fn larger_resolutions_estimate_larger_files() {
    let small = estimate_megabytes(ImageFormat::Png, 0, 1.0);
    let large = estimate_megabytes(ImageFormat::Png, 3, 1.0);
    assert!(large > small * 10.0);
    assert!(
        estimate_megabytes(ImageFormat::Jpeg, 2, 0.2)
            < estimate_megabytes(ImageFormat::Jpeg, 2, 1.0)
    );
}

#[test]
fn queue_summary_and_clearing() {
    let mut jobs = sample::jobs();
    let (queued, rendering, finished) = summary(&jobs);
    assert_eq!(queued + rendering + finished, jobs.len());
    let removed = clear_finished(&mut jobs);
    assert_eq!(removed, finished);
    assert!(jobs.iter().all(|job| !job.status.is_finished()));
}

#[test]
fn only_png_and_exr_keep_transparency() {
    assert!(ImageFormat::Png.supports_alpha());
    assert!(!ImageFormat::Jpeg.supports_alpha());
}

#[test]
fn resolution_labels_use_multiplication_sign() {
    assert_eq!(resolution_label(2), "3840×2160");
}
