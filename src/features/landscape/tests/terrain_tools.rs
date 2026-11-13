//! Brushes, generation, heightmaps and sampling.

use crate::features::landscape::logic::actions;
use crate::features::landscape::logic::generate::{auto_paint, generate_fbm};
use crate::features::landscape::logic::heightmap::{apply_heightmap, decode_png, HeightmapError};
use crate::features::landscape::logic::noise::{fbm, Rng};
use crate::features::landscape::logic::sample::{height_at, slope_deg_at};
use crate::features::landscape::logic::sculpt::{falloff_weight, paint, sculpt};
use crate::features::landscape::models::{
    Brush, FbmSettings, LandscapeState, SoilLayer, Terrain, Tool,
};

fn small() -> Terrain {
    Terrain::flat(33, 33, 1.0)
}

#[test]
fn raise_lifts_the_center_and_leaves_far_vertices() {
    let mut terrain = small();
    let brush = Brush {
        radius: 5.0,
        strength: 1.0,
        falloff: 0.5,
        ..Brush::default()
    };
    assert!(sculpt(&mut terrain, Tool::Raise, &brush, 0.0, 0.0, 0.5, 1));
    assert!(height_at(&terrain, 0.0, 0.0) > 1.0);
    assert_eq!(height_at(&terrain, 14.0, 14.0), 0.0);
}

#[test]
fn lower_is_the_opposite_of_raise() {
    let mut terrain = small();
    let brush = Brush {
        radius: 5.0,
        strength: 1.0,
        ..Brush::default()
    };
    sculpt(&mut terrain, Tool::Lower, &brush, 0.0, 0.0, 0.5, 1);
    assert!(height_at(&terrain, 0.0, 0.0) < -1.0);
}

#[test]
fn flatten_converges_to_the_target() {
    let mut terrain = small();
    terrain.heights.iter_mut().for_each(|h| *h = 10.0);
    let brush = Brush {
        radius: 8.0,
        strength: 1.0,
        falloff: 0.0,
        target_height: 2.0,
        ..Brush::default()
    };
    for _ in 0..40 {
        sculpt(&mut terrain, Tool::Flatten, &brush, 0.0, 0.0, 1.0, 1);
    }
    assert!((height_at(&terrain, 0.0, 0.0) - 2.0).abs() < 0.01);
}

#[test]
fn smooth_reduces_a_spike() {
    let mut terrain = small();
    let center = terrain.index(16, 16);
    terrain.heights[center] = 10.0;
    let brush = Brush {
        radius: 4.0,
        strength: 1.0,
        ..Brush::default()
    };
    sculpt(&mut terrain, Tool::Smooth, &brush, 0.0, 0.0, 1.0, 1);
    assert!(terrain.heights[center] < 10.0);
}

#[test]
fn noise_is_reproducible_for_the_same_seed_and_salt() {
    let brush = Brush {
        radius: 6.0,
        strength: 1.0,
        ..Brush::default()
    };
    let (mut a, mut b) = (small(), small());
    sculpt(&mut a, Tool::Noise, &brush, 0.0, 0.0, 1.0, 5);
    sculpt(&mut b, Tool::Noise, &brush, 0.0, 0.0, 1.0, 5);
    assert_eq!(a, b);
    assert!(a.heights.iter().any(|h| *h != 0.0));
}

#[test]
fn falloff_is_one_in_the_core_and_zero_at_the_radius() {
    let brush = Brush {
        radius: 10.0,
        falloff: 0.5,
        ..Brush::default()
    };
    assert_eq!(falloff_weight(&brush, 0.0), 1.0);
    assert_eq!(falloff_weight(&brush, 10.0), 0.0);
    let mid = falloff_weight(&brush, 7.5);
    assert!(mid > 0.0 && mid < 1.0);
    let hard = Brush {
        falloff: 0.0,
        ..brush
    };
    assert_eq!(falloff_weight(&hard, 9.9), 1.0);
}

#[test]
fn painting_keeps_weights_summing_to_255() {
    let mut terrain = small();
    let brush = Brush {
        radius: 6.0,
        strength: 0.7,
        falloff: 0.4,
        ..Brush::default()
    };
    for _ in 0..5 {
        paint(&mut terrain, SoilLayer::Sand, &brush, 0.0, 0.0, 0.3);
        paint(&mut terrain, SoilLayer::Rock, &brush, 2.0, 1.0, 0.3);
    }
    for weights in &terrain.layers {
        assert_eq!(weights.iter().map(|w| u32::from(*w)).sum::<u32>(), 255);
    }
    let center = terrain.layers[terrain.index(16, 16)];
    assert!(center[SoilLayer::Sand.index()] > 0 || center[SoilLayer::Rock.index()] > 0);
}

#[test]
fn fbm_is_deterministic_bounded_and_seed_dependent() {
    let a = fbm(3.3, 1.7, 9, 5, 2.0, 0.5);
    assert_eq!(a, fbm(3.3, 1.7, 9, 5, 2.0, 0.5));
    assert!((-1.0..=1.0).contains(&a));
    assert_ne!(a, fbm(3.3, 1.7, 10, 5, 2.0, 0.5));
    let mut rng = Rng::new(1);
    assert!((0.0..1.0).contains(&rng.next_f32()));
}

#[test]
fn generation_stays_in_range_and_repeats() {
    let settings = FbmSettings {
        amplitude: 10.0,
        base_height: -2.0,
        ..FbmSettings::default()
    };
    let (mut a, mut b) = (small(), small());
    generate_fbm(&mut a, &settings);
    generate_fbm(&mut b, &settings);
    assert_eq!(a, b);
    let (low, high) = a.height_range();
    assert!(low >= -2.0 && high <= 8.0 && high > low);
}

#[test]
fn auto_paint_puts_sand_low_and_rock_on_cliffs() {
    let mut terrain = small();
    for row in 0..33 {
        for column in 0..33 {
            let index = terrain.index(column, row);
            terrain.heights[index] = if column > 20 { 40.0 } else { -1.0 };
        }
    }
    auto_paint(&mut terrain, 0.6, 38.0);
    let low = terrain.layers[terrain.index(5, 16)];
    assert!(low[SoilLayer::Sand.index()] > 128);
    let cliff = terrain.layers[terrain.index(20, 16)];
    assert!(cliff[SoilLayer::Rock.index()] > 128);
}

fn png(width: u32, height: u32, pixels: Vec<u8>) -> Vec<u8> {
    let image = image::GrayImage::from_raw(width, height, pixels).expect("size matches");
    let mut bytes = std::io::Cursor::new(Vec::new());
    image
        .write_to(&mut bytes, image::ImageFormat::Png)
        .expect("png encodes");
    bytes.into_inner()
}

#[test]
fn heightmap_maps_black_and_white_and_orients_north_up() {
    // 2x2: top row black, bottom row white. Top row is the north (+Y) edge.
    let image = decode_png(&png(2, 2, vec![0, 0, 255, 255])).expect("decodes");
    let mut terrain = Terrain::flat(5, 5, 1.0);
    apply_heightmap(&mut terrain, &image, 0.0, 10.0);
    assert!((terrain.heights[terrain.index(2, 4)] - 0.0).abs() < 1e-3);
    assert!((terrain.heights[terrain.index(2, 0)] - 10.0).abs() < 1e-3);
}

#[test]
fn bad_heightmaps_are_rejected_without_panicking() {
    assert_eq!(decode_png(b"not a png"), Err(HeightmapError::NotPng));
    let mut state = LandscapeState::default();
    actions::import_heightmap(&mut state, &[1, 2, 3]);
    assert!(matches!(
        state.status,
        Some(crate::features::landscape::models::Status::Error(_))
    ));
}

#[test]
fn height_and_slope_follow_a_ramp() {
    let mut terrain = small();
    for row in 0..33 {
        for column in 0..33 {
            let index = terrain.index(column, row);
            terrain.heights[index] = column as f32; // 1 m rise per 1 m: 45 degrees
        }
    }
    let x = terrain.origin[0] + 10.5;
    assert!((height_at(&terrain, x, 0.0) - 10.5).abs() < 1e-3);
    assert!((slope_deg_at(&terrain, x, 0.3) - 45.0).abs() < 1.0);
    assert_eq!(
        height_at(&terrain, -1.0e6, 0.0),
        terrain.heights[terrain.index(0, 16)]
    );
}

#[test]
fn strokes_bump_the_revision_only_on_change() {
    let mut state = LandscapeState {
        tool: Tool::Raise,
        ..LandscapeState::default()
    };
    let before = state.revision;
    assert!(actions::apply_stroke(&mut state, 0.0, 0.0, 0.2));
    assert_eq!(state.revision, before + 1);
    assert!(!actions::apply_stroke(&mut state, 5.0e5, 0.0, 0.2));
}
