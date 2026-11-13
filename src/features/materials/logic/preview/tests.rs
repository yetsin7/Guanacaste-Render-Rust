//! Preview rendering: coverage, lighting direction, material response and live updates.

use super::*;
use crate::features::materials::models::{Procedural, ProceduralKind, TextureRef};
use crate::services::texture_cache::{Bitmap, MipChain};
use std::sync::Arc;

const SIZE: usize = 64;

fn params(shape: PreviewShape) -> PreviewParams {
    PreviewParams {
        shape,
        size: SIZE,
        yaw: 0.6,
        pitch: 0.45,
    }
}

fn paint(material: &Material, shape: PreviewShape) -> PreviewImage {
    render(material, &MaterialMaps::none(), &params(shape))
}

fn at(image: &PreviewImage, x: usize, y: usize) -> [u8; 4] {
    image.pixels[y * image.size + x]
}

fn brightness(p: [u8; 4]) -> u32 {
    p[0] as u32 + p[1] as u32 + p[2] as u32
}

fn matte(color: [f32; 3]) -> Material {
    Material {
        base_color: [color[0], color[1], color[2], 1.0],
        roughness: 0.9,
        ..Material::default()
    }
}

#[test]
fn image_has_the_requested_size_and_is_deterministic() {
    let m = matte([0.5; 3]);
    let a = paint(&m, PreviewShape::Sphere);
    assert_eq!(a.size, SIZE);
    assert_eq!(a.pixels.len(), SIZE * SIZE);
    assert_eq!(a, paint(&m, PreviewShape::Sphere));
}

#[test]
fn the_sphere_covers_the_expected_area_and_leaves_corners_empty() {
    let image = paint(&matte([0.5; 3]), PreviewShape::Sphere);
    assert_eq!(at(&image, 0, 0)[3], 0);
    assert_eq!(at(&image, SIZE - 1, SIZE - 1)[3], 0);
    assert_eq!(at(&image, SIZE / 2, SIZE / 2)[3], 255);
    let covered: f32 = image.pixels.iter().map(|p| p[3] as f32 / 255.0).sum();
    let expected = std::f32::consts::PI * 0.81 / 4.0 * (SIZE * SIZE) as f32;
    assert!(
        (covered - expected).abs() / expected < 0.03,
        "{covered} vs {expected}"
    );
}

#[test]
fn the_sphere_edge_is_antialiased() {
    let image = paint(&matte([0.5; 3]), PreviewShape::Sphere);
    assert!(image.pixels.iter().any(|p| p[3] > 0 && p[3] < 255));
}

#[test]
fn the_cube_fills_its_silhouette() {
    let image = paint(&matte([0.5; 3]), PreviewShape::Cube);
    assert_eq!(at(&image, SIZE / 2, SIZE / 2)[3], 255);
    assert_eq!(at(&image, 0, 0)[3], 0);
    let solid = image.pixels.iter().filter(|p| p[3] == 255).count();
    assert!(
        solid > SIZE * SIZE / 5 && solid < SIZE * SIZE * 3 / 5,
        "{solid}"
    );
}

#[test]
fn the_light_comes_from_the_upper_left() {
    let image = render(
        &matte([0.8; 3]),
        &MaterialMaps::none(),
        &PreviewParams {
            yaw: 0.0,
            pitch: 0.0,
            ..params(PreviewShape::Sphere)
        },
    );
    let lit = at(&image, SIZE * 3 / 10 + 4, SIZE * 3 / 10 + 4);
    let shadow = at(&image, SIZE * 7 / 10 - 4, SIZE * 7 / 10 - 4);
    assert!(
        brightness(lit) > brightness(shadow) + 60,
        "{lit:?} {shadow:?}"
    );
}

#[test]
fn albedo_color_drives_the_hue() {
    let red = at(
        &paint(&matte([0.9, 0.1, 0.1]), PreviewShape::Sphere),
        32,
        32,
    );
    assert!(red[0] > red[1] + 40 && red[0] > red[2] + 40, "{red:?}");
    let blue = at(
        &paint(&matte([0.1, 0.1, 0.9]), PreviewShape::Sphere),
        32,
        32,
    );
    assert!(blue[2] > blue[0] + 40, "{blue:?}");
}

#[test]
fn changing_any_parameter_changes_the_image() {
    let base = matte([0.5; 3]);
    let reference = paint(&base, PreviewShape::Sphere);
    let variants = [
        Material {
            roughness: 0.1,
            ..base.clone()
        },
        Material {
            metallic: 1.0,
            ..base.clone()
        },
        Material {
            transparency: 0.6,
            ..base.clone()
        },
        Material {
            emission: [1.0, 0.5, 0.2],
            ..base.clone()
        },
        Material {
            base_color: [0.2, 0.5, 0.2, 1.0],
            ..base.clone()
        },
    ];
    for variant in variants {
        assert_ne!(
            paint(&variant, PreviewShape::Sphere),
            reference,
            "{variant:?}"
        );
    }
}

#[test]
fn smooth_metal_reflects_the_environment_more_than_matte_paint() {
    let mirror = Material {
        base_color: [0.9, 0.9, 0.9, 1.0],
        metallic: 1.0,
        roughness: 0.05,
        ..Material::default()
    };
    let image = paint(&mirror, PreviewShape::Sphere);
    let values: Vec<u32> = image
        .pixels
        .iter()
        .filter(|p| p[3] == 255)
        .map(|p| brightness(*p))
        .collect();
    let spread = values.iter().max().unwrap() - values.iter().min().unwrap();
    let flat = paint(&matte([0.9; 3]), PreviewShape::Sphere);
    let flat_values: Vec<u32> = flat
        .pixels
        .iter()
        .filter(|p| p[3] == 255)
        .map(|p| brightness(*p))
        .collect();
    let flat_spread = flat_values.iter().max().unwrap() - flat_values.iter().min().unwrap();
    assert!(spread > flat_spread / 2, "{spread} {flat_spread}");
    // A metal's reflections are tinted by its color, so a dark metal reflects less light than a bright one.
    let dark_metal = Material {
        base_color: [0.05; 4],
        ..mirror.clone()
    };
    let dark = brightness(at(&paint(&dark_metal, PreviewShape::Sphere), 32, 32));
    let bright = brightness(at(&paint(&mirror, PreviewShape::Sphere), 32, 32));
    assert!(dark < bright, "{dark} {bright}");
}

#[test]
fn transparent_materials_show_the_checkerboard_behind() {
    let glass = Material {
        transparency: 0.95,
        roughness: 0.2,
        ..matte([0.9; 3])
    };
    let solid = matte([0.9; 3]);
    // Pixels (28, 32) and (36, 32) sit in neighbouring 8 px checker cells near the center.
    let difference = |m: &Material| {
        let image = paint(m, PreviewShape::Sphere);
        brightness(at(&image, 31, 32)).abs_diff(brightness(at(&image, 32, 32)))
    };
    assert!(difference(&glass) > difference(&solid) + 30);
}

#[test]
fn emission_glows_even_on_the_shadow_side() {
    let lamp = Material {
        emission: [1.0, 0.7, 0.3],
        emission_strength: 6.0,
        ..matte([0.2; 3])
    };
    let off = at(&paint(&matte([0.2; 3]), PreviewShape::Sphere), 48, 48);
    let on = at(&paint(&lamp, PreviewShape::Sphere), 48, 48);
    assert!(brightness(on) > brightness(off) + 100);
    assert!(on[0] > on[2]);
}

fn two_tone() -> MaterialMaps {
    // Left half red, right half green.
    let mut bitmap = Bitmap::filled(2, 1, [255, 0, 0, 255]);
    bitmap.pixels[1] = [0, 255, 0, 255];
    MaterialMaps {
        albedo: Some(Arc::new(MipChain::build(bitmap, true))),
        ..MaterialMaps::none()
    }
}

#[test]
fn an_albedo_texture_is_painted_onto_the_shape() {
    let material = Material {
        roughness: 0.9,
        ..Material::default()
    };
    let cube = PreviewParams {
        yaw: 0.0,
        pitch: 0.0,
        ..params(PreviewShape::Cube)
    };
    let image = render(&material, &two_tone(), &cube);
    let left = at(&image, SIZE / 2 - 8, SIZE / 2);
    let right = at(&image, SIZE / 2 + 8, SIZE / 2);
    assert!(left[0] > left[1] + 30, "left should be red {left:?}");
    assert!(right[1] > right[0] + 30, "right should be green {right:?}");
}

#[test]
fn tiling_repeats_the_texture() {
    let mut material = Material::default();
    material.uv.scale = [4.0, 4.0];
    let tiled = render(
        &material,
        &two_tone(),
        &PreviewParams {
            yaw: 0.0,
            pitch: 0.0,
            ..params(PreviewShape::Cube)
        },
    );
    let reds = tiled
        .pixels
        .iter()
        .filter(|p| p[3] == 255 && p[0] as u32 > p[1] as u32 + 30)
        .count();
    let greens = tiled
        .pixels
        .iter()
        .filter(|p| p[3] == 255 && p[1] as u32 > p[0] as u32 + 30)
        .count();
    assert!(reds > 100 && greens > 100);
}

#[test]
fn rotating_the_object_changes_a_textured_preview_only() {
    let material = Material::default();
    let maps = two_tone();
    let a = render(
        &material,
        &maps,
        &PreviewParams {
            yaw: 0.0,
            ..params(PreviewShape::Sphere)
        },
    );
    let b = render(
        &material,
        &maps,
        &PreviewParams {
            yaw: 1.5,
            ..params(PreviewShape::Sphere)
        },
    );
    assert_ne!(a, b);
    // A plain material on a sphere looks the same from any yaw.
    let plain = matte([0.5; 3]);
    let c = render(
        &plain,
        &MaterialMaps::none(),
        &PreviewParams {
            yaw: 0.0,
            ..params(PreviewShape::Sphere)
        },
    );
    let d = render(
        &plain,
        &MaterialMaps::none(),
        &PreviewParams {
            yaw: 1.5,
            ..params(PreviewShape::Sphere)
        },
    );
    assert_eq!(c, d);
}

#[test]
fn a_normal_map_changes_the_shading() {
    use crate::features::materials::logic::maps::MaterialMaps as Maps;
    use crate::services::texture_cache::TextureCache;
    let mut flat = matte([0.7; 3]);
    let reference = paint(&flat, PreviewShape::Sphere);
    flat.maps.normal = Some(TextureRef::procedural(Procedural::new(
        ProceduralKind::Brick,
    )));
    let mut cache = TextureCache::default();
    let (maps, failures) = Maps::resolve(&mut cache, &flat);
    assert!(failures.is_empty() && maps.normal.is_some());
    let bumped = render(&flat, &maps, &params(PreviewShape::Sphere));
    assert_ne!(bumped, reference);
    // Strength 0 disables the relief again.
    flat.normal_strength = 0.0;
    let off = render(&flat, &maps, &params(PreviewShape::Sphere));
    assert_eq!(off, reference);
}

#[test]
fn degenerate_sizes_are_clamped() {
    let m = matte([0.5; 3]);
    let tiny = render(
        &m,
        &MaterialMaps::none(),
        &PreviewParams {
            size: 0,
            ..params(PreviewShape::Sphere)
        },
    );
    assert_eq!(tiny.size, 8);
}
