//! Built-in materials for architecture, with physically plausible values (sRGB albedo, measured-like metal colors,
//! typical roughness and index of refraction).

use crate::features::materials::models::{Material, Procedural, ProceduralKind, TextureRef};
use crate::services::localization::tr;

/// A material named by the localization key `key`.
fn base(key: &'static str, color: [f32; 3], metallic: f32, roughness: f32) -> Material {
    Material {
        name: tr(key).to_owned(),
        base_color: [color[0], color[1], color[2], 1.0],
        metallic,
        roughness,
        ..Material::default()
    }
}

/// A clear or tinted transparent material.
fn clear(
    key: &'static str,
    tint: [f32; 3],
    transparency: f32,
    ior: f32,
    roughness: f32,
) -> Material {
    Material {
        transparency,
        ior,
        ..base(key, tint, 0.0, roughness)
    }
}

/// A material whose color and relief come from a procedural pattern; `tint` multiplies the pattern.
fn patterned(
    key: &'static str,
    kind: ProceduralKind,
    tint: [f32; 3],
    roughness: f32,
    bump: f32,
    tiles: f32,
) -> Material {
    let mut material = base(key, tint, 0.0, roughness);
    let pattern = Procedural::new(kind);
    material.maps.albedo = Some(TextureRef::procedural(pattern));
    if bump > 0.0 {
        material.maps.normal = Some(TextureRef::procedural(pattern));
        material.normal_strength = bump;
    }
    material.uv.scale = [tiles, tiles];
    material
}

/// Every preset, in menu order.
pub fn all() -> Vec<Material> {
    let white = [1.0; 3];
    let mut light = base("material.preset_light", [1.0, 0.9, 0.75], 0.0, 0.4);
    light.emission = [1.0, 0.88, 0.7];
    light.emission_strength = 8.0;
    vec![
        patterned(
            "material.preset_concrete",
            ProceduralKind::Concrete,
            white,
            0.9,
            0.5,
            1.0,
        ),
        base(
            "material.preset_concrete_polished",
            [0.6, 0.6, 0.59],
            0.0,
            0.3,
        ),
        patterned(
            "material.preset_brick",
            ProceduralKind::Brick,
            white,
            0.85,
            0.9,
            2.0,
        ),
        patterned(
            "material.preset_oak",
            ProceduralKind::Wood,
            white,
            0.55,
            0.3,
            1.0,
        ),
        patterned(
            "material.preset_walnut",
            ProceduralKind::Wood,
            [0.62, 0.45, 0.4],
            0.45,
            0.3,
            1.0,
        ),
        clear("material.preset_glass", [0.95, 1.0, 0.97], 0.92, 1.52, 0.02),
        clear(
            "material.preset_glass_tinted",
            [0.45, 0.65, 0.6],
            0.75,
            1.52,
            0.03,
        ),
        base("material.preset_steel", [0.56, 0.57, 0.58], 1.0, 0.35),
        base("material.preset_aluminium", [0.91, 0.92, 0.92], 1.0, 0.45),
        base("material.preset_copper", [0.95, 0.64, 0.54], 1.0, 0.3),
        base("material.preset_gold", [1.0, 0.77, 0.34], 1.0, 0.2),
        base("material.preset_chrome", [0.55, 0.56, 0.55], 1.0, 0.03),
        base("material.preset_paint_white", [0.92, 0.91, 0.88], 0.0, 0.85),
        base("material.preset_paint_grey", [0.5, 0.52, 0.55], 0.0, 0.7),
        base("material.preset_plaster", [0.88, 0.86, 0.82], 0.0, 0.9),
        Material {
            ior: 1.55,
            ..base("material.preset_ceramic", [0.93, 0.93, 0.91], 0.0, 0.08)
        },
        base("material.preset_terracotta", [0.72, 0.36, 0.24], 0.0, 0.7),
        patterned(
            "material.preset_marble",
            ProceduralKind::Marble,
            white,
            0.12,
            0.0,
            1.0,
        ),
        patterned(
            "material.preset_granite",
            ProceduralKind::Concrete,
            [0.4, 0.4, 0.44],
            0.25,
            0.0,
            1.0,
        ),
        patterned(
            "material.preset_grass",
            ProceduralKind::Grass,
            white,
            0.95,
            0.6,
            3.0,
        ),
        clear(
            "material.preset_water",
            [0.05, 0.25, 0.3],
            0.85,
            1.333,
            0.02,
        ),
        patterned(
            "material.preset_asphalt",
            ProceduralKind::Concrete,
            [0.3, 0.3, 0.31],
            0.95,
            0.7,
            2.0,
        ),
        base("material.preset_rubber", [0.16, 0.16, 0.16], 0.0, 0.8),
        light,
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::features::materials::logic::color::{luminance, srgb_to_linear};
    use std::collections::HashSet;

    #[test]
    fn there_are_distinct_named_presets_for_every_family() {
        let presets = all();
        assert!(presets.len() >= 20);
        let names: HashSet<_> = presets.iter().map(|m| m.name.clone()).collect();
        assert_eq!(names.len(), presets.len(), "names must be unique");
        for family in [
            "Concreto",
            "Madera",
            "Vidrio",
            "Acero",
            "Pintura",
            "Cerámica",
            "Césped",
            "Agua",
        ] {
            assert!(
                presets.iter().any(|m| m.name.starts_with(family)),
                "missing family {family}"
            );
        }
        assert!(presets.iter().all(|m| !m.name.starts_with("material.")));
    }

    #[test]
    fn every_preset_is_already_within_valid_ranges() {
        for m in all() {
            assert_eq!(m.clone().clamped(), m, "{} has out-of-range values", m.name);
        }
    }

    #[test]
    fn metals_are_bright_and_dielectrics_are_not_perfect_white_or_black() {
        for m in all() {
            let albedo = luminance([0, 1, 2].map(|i| srgb_to_linear(m.base_color[i])));
            if m.metallic > 0.5 {
                assert_eq!(m.metallic, 1.0, "{}: metals are fully metallic", m.name);
                assert!(albedo > 0.15, "{}: metal reflectance {albedo}", m.name);
            } else {
                assert_eq!(m.metallic, 0.0, "{}: no half-metals", m.name);
                if !m.is_transparent() && m.maps.albedo.is_none() {
                    // Natural dielectrics reflect between ~3% (charcoal) and ~90% (fresh paint) of light.
                    assert!((0.02..0.9).contains(&albedo), "{}: albedo {albedo}", m.name);
                }
            }
        }
    }

    #[test]
    fn transparent_materials_have_a_realistic_ior_and_are_smooth() {
        let presets = all();
        let transparent: Vec<_> = presets.iter().filter(|m| m.is_transparent()).collect();
        assert!(transparent.len() >= 3);
        for m in transparent {
            assert!((1.3..=1.6).contains(&m.ior), "{} ior {}", m.name, m.ior);
            assert!(m.roughness <= 0.1, "{} should be smooth", m.name);
        }
    }

    #[test]
    fn water_and_glass_differ_in_ior() {
        let presets = all();
        let ior = |name: &str| {
            presets
                .iter()
                .find(|m| m.name.starts_with(name))
                .unwrap()
                .ior
        };
        assert!(ior("Agua") < ior("Vidrio claro"));
    }

    #[test]
    fn exactly_the_light_is_emissive() {
        let emissive: Vec<_> = all().into_iter().filter(Material::is_emissive).collect();
        assert_eq!(emissive.len(), 1);
        assert!(emissive[0].emission_strength > 1.0);
    }

    #[test]
    fn patterned_presets_carry_tileable_textures_and_a_bump() {
        let presets = all();
        let brick = presets
            .iter()
            .find(|m| m.name.starts_with("Ladrillo"))
            .unwrap();
        assert!(brick.maps.albedo.is_some() && brick.maps.normal.is_some());
        let marble = presets
            .iter()
            .find(|m| m.name.starts_with("Mármol"))
            .unwrap();
        assert!(marble.maps.normal.is_none());
    }
}
