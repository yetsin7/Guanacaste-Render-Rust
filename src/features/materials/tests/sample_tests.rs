//! The sampling pipeline end to end: factors, textures, UV mapping, filtering and procedural maps.

use crate::features::materials::logic::color::srgb_to_linear;
use crate::features::materials::models::{
    MapKind, Material, Procedural, ProceduralKind, TextureRef, Wrap,
};
use crate::features::materials::{sample, surface, MaterialMaps, TextureCache};
use crate::services::texture_cache::{Bitmap, LoadError, MipChain};
use std::sync::Arc;

fn near(a: f32, b: f32) -> bool {
    (a - b).abs() < 2e-3
}

fn chain(bitmap: Bitmap, srgb: bool) -> Option<Arc<MipChain>> {
    Some(Arc::new(MipChain::build(bitmap, srgb)))
}

/// 2x2 image: red, green / blue, white.
fn quad() -> Bitmap {
    Bitmap {
        width: 2,
        height: 2,
        pixels: vec![
            [255, 0, 0, 255],
            [0, 255, 0, 255],
            [0, 0, 255, 255],
            [255; 4],
        ],
    }
}

fn albedo_maps(bitmap: Bitmap) -> MaterialMaps {
    MaterialMaps {
        albedo: chain(bitmap, true),
        ..MaterialMaps::none()
    }
}

fn white() -> Material {
    Material {
        base_color: [1.0; 4],
        ..Material::default()
    }
}

#[test]
fn without_textures_the_sample_is_the_linear_base_color() {
    let m = Material {
        base_color: [0.5, 1.0, 0.0, 1.0],
        ..Material::default()
    };
    let c = sample(&m, &MaterialMaps::none(), [0.3, 0.7]);
    assert!(near(c[0], srgb_to_linear(0.5)) && near(c[1], 1.0) && c[2] == 0.0);
    assert_eq!(c[3], 1.0);
    assert!(near(srgb_to_linear(0.5), 0.2140));
}

#[test]
fn alpha_combines_base_alpha_and_transparency() {
    let m = Material {
        base_color: [1.0, 1.0, 1.0, 0.8],
        transparency: 0.5,
        ..Material::default()
    };
    assert!(near(sample(&m, &MaterialMaps::none(), [0.0; 2])[3], 0.4));
}

#[test]
fn the_albedo_map_is_sampled_at_texel_centers_and_tinted() {
    let maps = albedo_maps(quad());
    let m = white();
    let at = |u, v| sample(&m, &maps, [u, v]);
    assert!(near(at(0.25, 0.25)[0], 1.0) && at(0.25, 0.25)[1] < 1e-3); // red, top-left
    assert!(near(at(0.75, 0.25)[1], 1.0)); // green, top-right
    assert!(near(at(0.25, 0.75)[2], 1.0)); // blue, bottom-left
    let tinted = Material {
        base_color: [0.5, 1.0, 1.0, 1.0],
        ..white()
    };
    assert!(near(
        sample(&tinted, &maps, [0.25, 0.25])[0],
        srgb_to_linear(0.5)
    ));
}

#[test]
fn bilinear_filtering_blends_neighbouring_texels() {
    let maps = albedo_maps(quad());
    let mid = sample(&white(), &maps, [0.5, 0.25]);
    // Halfway between red and green, blended in linear light.
    assert!(near(mid[0], 0.5) && near(mid[1], 0.5), "{mid:?}");
}

#[test]
fn uv_scale_tiles_the_texture() {
    let maps = albedo_maps(quad());
    let mut m = white();
    m.uv.scale = [2.0, 2.0];
    let a = sample(&m, &maps, [0.25, 0.25]);
    let b = sample(&m, &maps, [0.75, 0.25]);
    assert_eq!(a, b, "one tile later the texture repeats");
}

#[test]
fn uv_offset_shifts_the_image() {
    let maps = albedo_maps(quad());
    let mut m = white();
    m.uv.offset = [0.5, 0.0];
    // The red texel that was at u = 0.25 now sits at u = -0.25, i.e. wrapped, green appears at 0.25.
    assert!(near(sample(&m, &maps, [0.25, 0.25])[1], 1.0));
}

#[test]
fn uv_rotation_turns_the_image_a_quarter_turn() {
    let maps = albedo_maps(quad());
    let mut m = white();
    m.uv.rotation = 90.0;
    let plain = sample(&white(), &maps, [0.75, 0.25]);
    let turned = sample(&m, &maps, [0.75, 0.25]);
    assert_ne!(plain, turned);
    // Rotating the lookup of the top-right corner by 90 degrees about the center lands on the bottom-right texel.
    assert!(near(turned[0], 1.0) && near(turned[1], 1.0) && near(turned[2], 1.0));
}

#[test]
fn wrap_modes_differ_outside_the_unit_square() {
    let maps = albedo_maps(quad());
    let mut m = white();
    let outside = [1.25, 0.25];
    m.uv.wrap = Wrap::Repeat;
    let repeat = sample(&m, &maps, outside);
    m.uv.wrap = Wrap::Clamp;
    let clamp = sample(&m, &maps, outside);
    m.uv.wrap = Wrap::Mirror;
    let mirror = sample(&m, &maps, outside);
    assert!(near(repeat[0], 1.0), "repeat wraps to red: {repeat:?}");
    assert!(
        near(clamp[1], 1.0),
        "clamp holds the green border: {clamp:?}"
    );
    assert!(near(mirror[1], 1.0), "mirror flips to green: {mirror:?}");
}

#[test]
fn large_footprints_select_blurrier_mips() {
    let mut checker = Bitmap::filled(64, 64, [0, 0, 0, 255]);
    for y in 0..64 {
        for x in 0..64 {
            if (x + y) % 2 == 0 {
                checker.pixels[y * 64 + x] = [255; 4];
            }
        }
    }
    let maps = albedo_maps(checker);
    let m = white();
    let sharp = surface(&m, &maps, [0.2890625, 0.2890625], 0.0).albedo[0];
    let blurred = surface(&m, &maps, [0.3, 0.3], 1.0).albedo[0];
    assert!(
        sharp < 0.2 || sharp > 0.8,
        "texel centers are pure: {sharp}"
    );
    assert!(
        (blurred - 0.5).abs() < 0.02,
        "a whole-image footprint averages to gray: {blurred}"
    );
}

#[test]
fn scalar_maps_multiply_their_factors() {
    let flat = |v: u8| chain(Bitmap::filled(2, 2, [v, v, v, 255]), false);
    let maps = MaterialMaps {
        roughness: flat(128),
        metallic: flat(255),
        ao: flat(51),
        ..MaterialMaps::none()
    };
    let m = Material {
        roughness: 0.8,
        metallic: 0.5,
        ..Material::default()
    };
    let s = surface(&m, &maps, [0.5; 2], 0.0);
    assert!(near(s.roughness, 0.8 * 128.0 / 255.0));
    assert!(near(s.metallic, 0.5));
    assert!(near(s.ao, 0.2));
}

#[test]
fn normal_maps_decode_and_respect_the_strength() {
    let tilted = chain(Bitmap::filled(2, 2, [255, 128, 200, 255]), false);
    let maps = MaterialMaps {
        normal: tilted,
        ..MaterialMaps::none()
    };
    let mut m = Material::default();
    let full = surface(&m, &maps, [0.5; 2], 0.0).normal;
    assert!(full[0] > 0.5, "leans along +X: {full:?}");
    assert!(near(
        (full[0] * full[0] + full[1] * full[1] + full[2] * full[2]).sqrt(),
        1.0
    ));
    m.normal_strength = 0.0;
    assert_eq!(surface(&m, &maps, [0.5; 2], 0.0).normal, [0.0, 0.0, 1.0]);
    assert_eq!(
        surface(&m, &MaterialMaps::none(), [0.5; 2], 0.0).normal,
        [0.0, 0.0, 1.0]
    );
}

#[test]
fn emission_multiplies_color_strength_and_map() {
    let mut m = Material {
        emission: [1.0, 0.0, 0.0],
        emission_strength: 3.0,
        ..Material::default()
    };
    assert!(near(
        surface(&m, &MaterialMaps::none(), [0.5; 2], 0.0).emission[0],
        3.0
    ));
    let half = MaterialMaps {
        emission: chain(Bitmap::filled(2, 2, [255, 255, 255, 255]), true),
        ..MaterialMaps::none()
    };
    m.emission_strength = 2.0;
    assert!(near(surface(&m, &half, [0.5; 2], 0.0).emission[0], 2.0));
    m.emission = [0.0; 3];
    assert_eq!(surface(&m, &half, [0.5; 2], 0.0).emission, [0.0; 3]);
}

#[test]
fn bad_coordinates_never_panic() {
    let maps = albedo_maps(quad());
    for uv in [
        [f32::NAN, 0.0],
        [f32::INFINITY, f32::NEG_INFINITY],
        [1e30, -1e30],
    ] {
        let c = sample(&white(), &maps, uv);
        assert!(c.iter().all(|v| v.is_finite()), "{uv:?} -> {c:?}");
    }
}

#[test]
fn sampling_is_pure() {
    let maps = albedo_maps(quad());
    let m = white();
    assert_eq!(sample(&m, &maps, [0.4, 0.6]), sample(&m, &maps, [0.4, 0.6]));
}

#[test]
fn procedural_textures_resolve_through_the_cache_and_vary_across_uv() {
    let mut m = white();
    m.maps.albedo = Some(TextureRef::procedural(Procedural::new(
        ProceduralKind::Brick,
    )));
    let mut cache = TextureCache::default();
    let (maps, failures) = MaterialMaps::resolve(&mut cache, &m);
    assert!(failures.is_empty());
    assert_eq!(cache.len(), 1);
    let mortar = sample(&m, &maps, [0.125, 0.0]);
    let brick = sample(&m, &maps, [0.125, 0.0625]);
    assert!(brick[0] > brick[2] * 2.0, "brick is red");
    assert!(mortar[2] > brick[2], "mortar is paler");
    // Resolving again reuses the baked texture.
    let _ = MaterialMaps::resolve(&mut cache, &m);
    assert_eq!(cache.len(), 1);
}

#[test]
fn procedural_slots_use_a_grayscale_bake_for_data_maps() {
    let mut m = white();
    m.maps.roughness = Some(TextureRef::procedural(Procedural::new(
        ProceduralKind::Brick,
    )));
    let mut cache = TextureCache::default();
    let (maps, _) = MaterialMaps::resolve(&mut cache, &m);
    let chain = maps.roughness.expect("resolved");
    assert!(chain.levels[0]
        .pixels
        .iter()
        .all(|p| p[0] == p[1] && p[1] == p[2]));
}

#[test]
fn a_missing_file_is_reported_and_the_slot_falls_back_to_the_factor() {
    let mut m = Material {
        base_color: [0.5, 0.5, 0.5, 1.0],
        ..Material::default()
    };
    m.maps.albedo = Some(TextureRef::file("no/such/texture.png"));
    let mut cache = TextureCache::default();
    let (maps, failures) = MaterialMaps::resolve(&mut cache, &m);
    assert_eq!(failures, [(MapKind::Albedo, LoadError::NotFound)]);
    assert!(maps.albedo.is_none());
    assert!(near(sample(&m, &maps, [0.1; 2])[0], srgb_to_linear(0.5)));
}

#[test]
fn a_png_on_disk_drives_the_albedo() {
    let path = std::env::temp_dir().join(format!("gr-sample-{}.png", std::process::id()));
    image::RgbaImage::from_pixel(4, 4, image::Rgba([0, 0, 255, 255]))
        .save(&path)
        .expect("write png");
    let mut m = white();
    m.maps.albedo = Some(TextureRef::file(path.to_string_lossy().into_owned()));
    let mut cache = TextureCache::default();
    let (maps, failures) = MaterialMaps::resolve(&mut cache, &m);
    let _ = std::fs::remove_file(&path);
    assert!(failures.is_empty());
    let c = sample(&m, &maps, [0.5; 2]);
    assert!(near(c[2], 1.0) && c[0] < 1e-3);
}

#[test]
fn a_height_flagged_normal_map_is_converted_to_a_normal_map() {
    let path = std::env::temp_dir().join(format!("gr-bump-{}.png", std::process::id()));
    let image = image::RgbaImage::from_fn(16, 16, |x, _| {
        let v = (x * 16) as u8;
        image::Rgba([v, v, v, 255])
    });
    image.save(&path).expect("write");
    let mut m = white();
    let mut bump = TextureRef::file(path.to_string_lossy().into_owned());
    bump.is_height = true;
    m.maps.normal = Some(bump);
    let mut cache = TextureCache::default();
    let (maps, failures) = MaterialMaps::resolve(&mut cache, &m);
    let _ = std::fs::remove_file(&path);
    assert!(failures.is_empty());
    let n = surface(&m, &maps, [0.5; 2], 0.0).normal;
    assert!(
        n[0] < -0.03,
        "a ramp rising to the right tilts the normal left: {n:?}"
    );
}
