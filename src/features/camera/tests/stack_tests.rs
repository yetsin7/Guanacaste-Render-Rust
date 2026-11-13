//! The filter chain, presets, serialization and the preview pipeline.

use super::{close, flat, rgb};
use crate::features::camera::logic::preview;
use crate::features::camera::models::camera::PhotoCamera;
use crate::features::camera::models::filter::{Filter, FilterKind};
use crate::features::camera::models::image_buffer::ImageBuffer;
use crate::features::camera::models::lens::Framing;
use crate::features::camera::models::preset::FilterPreset;
use crate::features::camera::models::stack::FilterStack;

/// A small picture with gradients, a bright spot and some color.
fn picture() -> ImageBuffer {
    let mut image = ImageBuffer::filled(24, 16, [0.0; 4]);
    for y in 0..16 {
        for x in 0..24 {
            let (u, v) = (x as f32 / 23.0, y as f32 / 15.0);
            image.set_pixel(x, y, [u * 0.9, v * 0.5 + 0.05, (1.0 - u) * 0.6, 1.0]);
        }
    }
    image.set_pixel(12, 8, [8.0, 7.0, 6.0, 1.0]);
    image
}

#[test]
fn empty_stack_and_disabled_entries_change_nothing() {
    let image = picture();
    assert_eq!(FilterStack::new().apply(&image), image);
    let mut stack = FilterStack::new();
    stack.push(FilterKind::Sepia);
    stack.push(FilterKind::Vignette);
    stack.set_enabled(0, false);
    stack.set_enabled(1, false);
    assert_eq!(stack.apply(&image), image);
    assert_eq!((stack.len(), stack.active_count()), (2, 0));
    stack.set_enabled(1, true);
    assert_ne!(stack.apply(&image), image);
    assert_eq!(stack.active_count(), 1);
}

#[test]
fn apply_leaves_the_input_untouched_and_order_matters() {
    let image = flat([0.5; 3]);
    let mut first = FilterStack::new();
    first.push_filter(Filter::with_params(FilterKind::Exposure, &[2.0]));
    first.push_filter(Filter::with_params(FilterKind::Reinhard, &[0.0]));
    let mut second = FilterStack::new();
    second.push_filter(Filter::with_params(FilterKind::Reinhard, &[0.0]));
    second.push_filter(Filter::with_params(FilterKind::Exposure, &[2.0]));
    let a = rgb(&first.apply(&image), 0, 0)[0];
    let b = rgb(&second.apply(&image), 0, 0)[0];
    close(a, 2.0 / 3.0, 1e-6);
    close(b, (0.5 / 1.5) * 4.0, 1e-6);
    assert_eq!(rgb(&image, 0, 0), [0.5; 3]);
}

#[test]
fn list_editing_keeps_the_order_consistent() {
    let mut stack = FilterStack::new();
    for kind in [
        FilterKind::Exposure,
        FilterKind::Contrast,
        FilterKind::Gamma,
    ] {
        stack.push(kind);
    }
    assert!(stack.move_entry(2, 0));
    let kinds: Vec<FilterKind> = stack.entries.iter().map(|e| e.filter.kind).collect();
    assert_eq!(
        kinds,
        [
            FilterKind::Gamma,
            FilterKind::Exposure,
            FilterKind::Contrast
        ]
    );
    assert!(!stack.move_entry(0, 3));
    assert!(stack.remove(1));
    assert!(!stack.remove(5));
    assert!(!stack.set_enabled(9, false));
    assert_eq!(stack.len(), 2);
}

#[test]
fn parameters_are_clamped_and_bad_input_is_ignored() {
    let mut filter = Filter::new(FilterKind::Vignette);
    filter.set_param(0, 7.0);
    assert_eq!(filter.params[0], 1.0);
    filter.set_param(1, f32::NAN);
    assert_eq!(filter.params[1], 0.5);
    filter.set_param(9, 1.0);
    assert_eq!(filter.params.len(), 3);
    let built = Filter::with_params(FilterKind::Saturation, &[9.0, 1.0, 1.0]);
    assert_eq!(built.params, [2.0]);
    let short = Filter::with_params(FilterKind::Bloom, &[2.0]);
    assert_eq!(short.params, [2.0, 0.3, 0.3]);
}

#[test]
fn every_filter_kind_works_with_defaults_and_keeps_alpha_and_size() {
    for kind in FilterKind::ALL {
        let mut image = picture();
        Filter::new(kind).apply(&mut image);
        assert_eq!((image.width(), image.height()), (24, 16), "{kind:?}");
        for px in image.data().chunks_exact(4) {
            assert!(px.iter().all(|v| v.is_finite()), "{kind:?} produced {px:?}");
            assert_eq!(px[3], 1.0, "{kind:?} changed alpha");
        }
        assert_eq!(kind.specs().len(), Filter::new(kind).params.len());
    }
}

#[test]
fn every_filter_parameter_extreme_is_finite() {
    for kind in FilterKind::ALL {
        for high in [false, true] {
            let values: Vec<f32> = kind
                .specs()
                .iter()
                .map(|s| if high { s.max } else { s.min })
                .collect();
            let mut image = picture();
            Filter::with_params(kind, &values).apply(&mut image);
            assert!(
                image.data().iter().all(|v| v.is_finite()),
                "{kind:?} {values:?}"
            );
        }
    }
}

#[test]
fn presets_are_distinct_deterministic_and_finite() {
    let image = picture();
    let mut results = Vec::new();
    for preset in FilterPreset::ALL {
        let stack = preset.stack();
        assert!(!stack.is_empty() && stack.active_count() == stack.len());
        let out = stack.apply(&image);
        assert_eq!(out, stack.apply(&image));
        assert!(out.data().iter().all(|v| v.is_finite()));
        results.push(out);
    }
    for i in 0..results.len() {
        for j in i + 1..results.len() {
            assert_ne!(
                results[i],
                results[j],
                "{:?} vs {:?}",
                FilterPreset::ALL[i],
                FilterPreset::ALL[j]
            );
        }
    }
}

#[test]
fn tone_mapped_presets_stay_displayable_and_black_white_is_grey() {
    let image = picture();
    for preset in [
        FilterPreset::Natural,
        FilterPreset::Cinematic,
        FilterPreset::BlackWhite,
    ] {
        let out = preset.stack().apply(&image);
        for px in out.data().chunks_exact(4) {
            assert!(
                px[..3].iter().all(|v| (-0.2..=1.0).contains(v)),
                "{preset:?} {px:?}"
            );
        }
    }
    let mono = FilterPreset::BlackWhite.stack().apply(&image);
    for px in mono.data().chunks_exact(4) {
        close(px[0], px[1], 1e-5);
        close(px[1], px[2], 1e-5);
    }
}

#[test]
fn stack_round_trips_through_json_and_repairs_damage() {
    for preset in FilterPreset::ALL {
        let stack = preset.stack();
        assert_eq!(FilterStack::from_json(&stack.to_json()), Some(stack));
    }
    let damaged = r#"{"entries":[{"enabled":true,"filter":{"kind":"Vignette","params":[5.0]}},
        {"enabled":false,"filter":{"kind":"Gamma","params":[1.0,2.0,3.0]}}]}"#;
    let stack = FilterStack::from_json(damaged).expect("parses");
    assert_eq!(stack.entries[0].filter.params, [1.0, 0.0, 0.05]);
    assert_eq!(stack.entries[1].filter.params, [1.0]);
    assert!(!stack.entries[1].enabled);
    assert!(FilterStack::from_json("not json").is_none());
    assert!(FilterStack::from_json(
        r#"{"entries":[{"enabled":true,"filter":{"kind":"Nope","params":[]}}]}"#
    )
    .is_none());
}

#[test]
fn crop_downscale_and_sample_source_have_the_expected_shapes() {
    let image = ImageBuffer::filled(300, 200, [0.3, 0.3, 0.3, 1.0]);
    let wide = preview::crop_to_aspect(&image, 16.0 / 9.0);
    assert_eq!((wide.width(), wide.height()), (300, 169));
    let square = preview::crop_to_aspect(&image, 1.0);
    assert_eq!((square.width(), square.height()), (200, 200));
    let tall = preview::crop_to_aspect(&image, 0.5);
    assert_eq!((tall.width(), tall.height()), (100, 200));
    let small = preview::downscale(&image, 75);
    assert_eq!((small.width(), small.height()), (75, 50));
    close(rgb(&small, 10, 10)[0], 0.3, 1e-6);
    assert_eq!(preview::downscale(&image, 600), image);
    let source = preview::sample_source();
    assert_eq!(source.width(), preview::PREVIEW_WIDTH);
    assert!(source.height() > 100);
    assert!(source.data().iter().any(|v| *v > 0.5));
}

#[test]
fn preview_follows_framing_and_exposure() {
    let source = ImageBuffer::filled(480, 320, [0.1, 0.1, 0.1, 1.0]);
    let mut camera = PhotoCamera {
        framing: Framing::Scope,
        ..Default::default()
    };
    let stack = FilterStack::new();
    let out = preview::render_preview(&source, &camera, &stack);
    close(out.width() as f32 / out.height() as f32, 2.39, 0.02);
    let base = rgb(&out, 5, 5)[0];
    camera.exposure.compensation_ev = 1.0;
    let brighter = preview::render_preview(&source, &camera, &stack);
    close(rgb(&brighter, 5, 5)[0], base * 2.0, 1e-4);
}
