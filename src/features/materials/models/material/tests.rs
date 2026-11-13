//! Tests of the material model: serialization and range clamping.

use super::*;
use crate::features::materials::models::{Procedural, ProceduralKind};

fn rich() -> Material {
    let mut m = Material::new("Roble");
    m.base_color = [0.123_456_79, 0.5, 0.987_654_3, 0.9];
    m.metallic = 0.25;
    m.roughness = 0.333_333_33;
    m.transparency = 0.1;
    m.ior = 1.457_891;
    m.emission = [0.1, 0.2, 0.3];
    m.emission_strength = 2.5;
    m.normal_strength = 0.75;
    m.uv = UvTransform {
        scale: [2.5, 3.0],
        rotation: 33.3,
        offset: [0.125, -0.5],
        wrap: Wrap::Mirror,
    };
    m.maps.albedo = Some(TextureRef::file("C:\\tex\\roble.jpg"));
    let mut normal = TextureRef::procedural(Procedural::new(ProceduralKind::Wood));
    normal.is_height = true;
    m.maps.normal = Some(normal);
    m.maps.roughness = Some(TextureRef::file("r.png"));
    m
}

#[test]
fn json_round_trip_is_exact() {
    let original = rich();
    let json = serde_json::to_string(&original).expect("serialize");
    let back: Material = serde_json::from_str(&json).expect("deserialize");
    assert_eq!(back, original);
}

#[test]
fn missing_fields_take_defaults() {
    let m: Material = serde_json::from_str(r#"{"name":"Solo nombre"}"#).expect("parse");
    assert_eq!(m.name, "Solo nombre");
    assert_eq!(m.roughness, Material::default().roughness);
    assert!(m.maps.albedo.is_none());
    let empty: Material = serde_json::from_str("{}").expect("parse");
    assert_eq!(empty, Material::default());
}

#[test]
fn unknown_fields_are_ignored() {
    let m: Material =
        serde_json::from_str(r#"{"name":"x","future_field":[1,2,3]}"#).expect("parse");
    assert_eq!(m.name, "x");
}

#[test]
fn clamping_forces_valid_ranges() {
    let mut m = Material {
        metallic: 4.0,
        roughness: -1.0,
        ior: 99.0,
        base_color: [f32::NAN, 2.0, -3.0, f32::INFINITY],
        normal_strength: 9.0,
        ..Material::default()
    };
    m.uv.scale = [0.0, f32::NAN];
    let c = m.clamped();
    assert_eq!((c.metallic, c.roughness, c.ior), (1.0, 0.0, 2.8));
    assert_eq!(c.base_color, [0.8, 1.0, 0.0, 1.0]);
    assert_eq!(c.uv.scale, [0.01, 1.0]);
    assert_eq!(c.normal_strength, 2.0);
}

#[test]
fn from_color_keeps_the_color_including_alpha() {
    let m = Material::from_color("Vidrio", [0.2, 0.4, 0.6, 0.25]);
    assert_eq!(m.base_color, [0.2, 0.4, 0.6, 0.25]);
    assert_eq!(m.transparency, 0.0);
    assert!(m.is_transparent());
    assert!(!Material::from_color("x", [1.0; 4]).is_transparent());
}

#[test]
fn emission_needs_color_or_map_and_strength() {
    let mut m = Material::default();
    assert!(!m.is_emissive());
    m.emission = [1.0, 0.8, 0.5];
    assert!(m.is_emissive());
    m.emission_strength = 0.0;
    assert!(!m.is_emissive());
    m.emission_strength = 1.0;
    m.emission = [0.0; 3];
    m.maps.emission = Some(TextureRef::file("e.png"));
    assert!(m.is_emissive());
}

#[test]
fn every_map_slot_is_reachable() {
    let mut maps = TextureMaps::default();
    for kind in MapKind::ALL {
        assert!(maps.get(kind).is_none());
        *maps.slot_mut(kind) = Some(TextureRef::file(format!("{kind:?}.png")));
        assert!(maps.get(kind).is_some());
    }
    assert_eq!(maps.ao, Some(TextureRef::file("AmbientOcclusion.png")));
    assert!(MapKind::Albedo.is_color() && !MapKind::Normal.is_color());
}
