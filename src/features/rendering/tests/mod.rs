//! Unit tests of the rendering domain.

mod export_tests;
mod engine_tests;
mod output_tests;
mod queue_tests;
mod scenes;

use super::logic::plan::{build_spec, plan_from_export, plan_from_studio};
use super::logic::*;
use super::models::*;
use std::sync::Arc;

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
    let small = estimate_megabytes(ImageFormat::Png, (640, 360), 1.0);
    let large = estimate_megabytes(ImageFormat::Png, (7680, 4320), 1.0);
    assert!(large > small * 10.0);
    assert!(
        estimate_megabytes(ImageFormat::Jpeg, (3840, 2160), 0.2)
            < estimate_megabytes(ImageFormat::Jpeg, (3840, 2160), 1.0)
    );
}

#[test]
fn only_png_keeps_transparency() {
    assert!(ImageFormat::Png.supports_alpha());
    assert!(!ImageFormat::Jpeg.supports_alpha());
}

#[test]
fn resolution_labels_use_multiplication_sign() {
    assert_eq!(resolution_label(4), "3840×2160");
}

#[test]
fn aspect_ratios_derive_the_height_from_the_width() {
    assert_eq!(output_size(2, 0), (1920, 1080));
    assert_eq!(output_size(2, 1), (1920, 1280));
    assert_eq!(output_size(2, 3), (1920, 1920));
}

#[test]
fn huge_shapes_are_scaled_to_the_texture_limit() {
    let (width, height) = output_size(5, 5);
    assert!(width.max(height) <= MAX_SIDE);
    // 9:16 stays portrait with the same proportions.
    assert!(height > width);
    assert!((height as f32 / width as f32 - 16.0 / 9.0).abs() < 0.01);
}

#[test]
fn clock_formats_hours_minutes_seconds() {
    assert_eq!(clock(std::time::Duration::from_secs(3725)), "01:02:05");
}

#[test]
fn queue_summary_counts_by_status() {
    let job = |status| JobInfo {
        id: 0,
        name: String::new(),
        preset: String::new(),
        resolution: String::new(),
        status,
        progress: 0.0,
        elapsed: std::time::Duration::ZERO,
        remaining: None,
        error: None,
        output: None,
    };
    let jobs = [
        job(JobStatus::Queued),
        job(JobStatus::Rendering),
        job(JobStatus::Done),
        job(JobStatus::Failed),
    ];
    assert_eq!(summary(&jobs), (1, 1, 2));
}

#[test]
fn plans_become_jobs_only_for_scenes_with_geometry() {
    let plan = plan_from_studio(&RenderStudioState::default());
    assert!(build_spec(&Arc::new(scenes::empty()), &plan, false).is_none());
    let cube = Arc::new(scenes::cube_on_ground());
    let spec = build_spec(&cube, &plan, false).expect("cube has bounds");
    assert_eq!(spec.request.camera.width, 1280);
    assert_eq!(spec.request.camera.height, 720);
    let suffixed = build_spec(&cube, &plan, true).expect("cube has bounds");
    assert_ne!(suffixed.output.file_name, spec.output.file_name);
}

#[test]
fn transparency_follows_the_format() {
    let mut export = ExportState {
        transparent: true,
        ..ExportState::default()
    };
    assert!(plan_from_export(&export).transparent);
    export.format = 1;
    assert!(!plan_from_export(&export).transparent);
}
