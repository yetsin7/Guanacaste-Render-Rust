//! The naming rules: material and type names, in Spanish and English, pick the surface family.

use super::text::fold;
use super::*;
use crate::features::materials::models::ProceduralKind;

const GRAY: [f32; 4] = [0.7, 0.7, 0.7, 1.0];

fn kind(name: &str, class: &str) -> SurfaceKind {
    classify(&SurfaceInfo {
        name,
        type_name: None,
        element: None,
        class,
        color: GRAY,
    })
}

fn info<'a>(name: &'a str, class: &'a str, color: [f32; 4]) -> SurfaceInfo<'a> {
    SurfaceInfo {
        name,
        type_name: None,
        element: None,
        class,
        color,
    }
}

#[test]
fn text_is_folded_without_accents() {
    assert_eq!(fold("CERÁMICA Baño"), "ceramica bano");
    assert_eq!(fold("Hormigón"), "hormigon");
}

#[test]
fn roof_names_become_clay_tile() {
    assert_eq!(kind("Cubiertas", "IFCSLAB"), SurfaceKind::RoofTile);
    assert_eq!(kind("techo general", "IFCCOVERING"), SurfaceKind::RoofTile);
    assert_eq!(kind("x", "IFCROOF"), SurfaceKind::RoofTile);
    assert_eq!(kind("Teja de barro", "IFCSLAB"), SurfaceKind::RoofTile);
}

#[test]
fn default_revit_walls_become_stucco() {
    assert_eq!(kind("Predeterminado", "IFCWALL"), SurfaceKind::Stucco);
    assert_eq!(
        kind("Ladrillo, común", "IFCWALLSTANDARDCASE"),
        SurfaceKind::Stucco
    );
    assert_eq!(
        kind("Brick, Common", "IFCWALLSTANDARDCASE"),
        SurfaceKind::Stucco
    );
    assert_eq!(kind("Estuco fino", "IFCSLAB"), SurfaceKind::Stucco);
    assert_eq!(kind("Repello", "IFCWALL"), SurfaceKind::Stucco);
    assert_eq!(kind("Brick, Common", "IFCSLAB"), SurfaceKind::Brick);
}

#[test]
fn wood_stone_metal_glass_and_lawn_are_recognized() {
    assert_eq!(kind("Madera - Tinte", "IFCWINDOW"), SurfaceKind::Wood);
    assert_eq!(kind("Wood-Marvin-Pine", "IFCDOOR"), SurfaceKind::Wood);
    assert_eq!(kind("PVC-MADERA-OSCURA", "IFCCOVERING"), SurfaceKind::Wood);
    assert_eq!(kind("MAM- Piedra Cantera", "IFCWALL"), SurfaceKind::Stone);
    assert_eq!(
        kind("ACERO DE CAJA METALICA", "IFCBEAM"),
        SurfaceKind::Metal
    );
    assert_eq!(kind("Aluminio", "IFCMEMBER"), SurfaceKind::Metal);
    assert_eq!(kind("Vidrio claro", "IFCWINDOW"), SurfaceKind::Glass);
    assert_eq!(kind("Grama", "IFCSLAB"), SurfaceKind::Grass);
    assert_eq!(kind("Césped", "IFCSLAB"), SurfaceKind::Grass);
}

#[test]
fn ceramics_ceilings_concrete_and_trim() {
    assert_eq!(
        kind("Cerámica Dormitorios", "IFCSLAB"),
        SurfaceKind::Ceramic
    );
    assert_eq!(
        kind("Azulejo Baño Privado", "IFCWALLSTANDARDCASE"),
        SurfaceKind::Ceramic
    );
    assert_eq!(
        kind("Ceramic - Wood Pattern", "IFCWALLSTANDARDCASE"),
        SurfaceKind::Wood
    );
    assert_eq!(
        kind("CRF- Cielo de PVC", "IFCCOVERING"),
        SurfaceKind::Plaster
    );
    assert_eq!(
        kind("Gypsum Wall Board", "IFCCOVERING"),
        SurfaceKind::Plaster
    );
    assert_eq!(
        kind("Hormigón, Moldeado in situ, gris", "IFCBEAM"),
        SurfaceKind::Concrete
    );
    assert_eq!(kind("PS- Cascote", "IFCSLAB"), SurfaceKind::Concrete);
    assert_eq!(kind("Rodapie interior", "IFCRAILING"), SurfaceKind::Paint);
    assert_eq!(
        kind("Textil beige", "IFCFURNISHINGELEMENT"),
        SurfaceKind::Fabric
    );
}

#[test]
fn class_decides_when_the_name_says_nothing() {
    assert_eq!(kind("???", "IFCDOOR"), SurfaceKind::Wood);
    assert_eq!(kind("???", "IFCSLAB"), SurfaceKind::Concrete);
    assert_eq!(kind("???", "IFCWALL"), SurfaceKind::Stucco);
    assert_eq!(kind("???", "IFCRAILING"), SurfaceKind::Metal);
    assert_eq!(kind("???", "IFCWINDOW"), SurfaceKind::Paint);
    assert_eq!(kind("???", "IFCSANITARYTERMINAL"), SurfaceKind::Plastic);
}

#[test]
fn stone_white_paint_is_not_stone() {
    assert_ne!(
        kind("Ultrex-Marvin-Stonewhite", "IFCBUILDINGELEMENTPROXY"),
        SurfaceKind::Stone
    );
}

#[test]
fn soil_pipes_and_invisible_styles_are_hidden() {
    assert_eq!(kind("PS- Relleno", "IFCSLAB"), SurfaceKind::Soil);
    assert_eq!(kind("Tierra Negra", "IFCSLAB"), SurfaceKind::Soil);
    let typed = SurfaceInfo {
        type_name: Some("Talud de tierra"),
        ..info("Predeterminado", "IFCSLAB", GRAY)
    };
    assert_eq!(classify(&typed), SurfaceKind::Soil);
    let named = SurfaceInfo {
        element: Some("Tierra negra 12"),
        ..info("Predeterminado", "IFCSLAB", GRAY)
    };
    assert_eq!(classify(&named), SurfaceKind::Soil);
    assert_eq!(
        kind("Predeterminado", "IFCFLOWSEGMENT"),
        SurfaceKind::Hidden
    );
    assert_eq!(
        kind("Tubería PVC", "IFCBUILDINGELEMENTPROXY"),
        SurfaceKind::Hidden
    );
    assert_eq!(
        classify(&info("Default", "IFCFLOWTERMINAL", [1.0, 1.0, 1.0, 0.0])),
        SurfaceKind::Hidden
    );
    assert!(SurfaceKind::Soil.is_hidden() && !SurfaceKind::Glass.is_hidden());
}

#[test]
fn transparent_styles_are_glass() {
    let blue = [0.0, 0.0, 1.0, 0.25];
    assert_eq!(
        classify(&info("Revestimiento - Blanco", "IFCWINDOW", blue)),
        SurfaceKind::Glass
    );
    let glass = derive(&info("Madera - Tinte", "IFCWINDOW", blue));
    assert!(glass.transparency > 0.8 && glass.roughness < 0.1);
    assert!(
        glass.base_color[2] > 0.8,
        "pale blue-green, not the file's pure blue"
    );
    assert!(glass.base_color[0] > 0.5);
}

#[test]
fn derived_stucco_is_light_and_textured_whatever_the_file_color() {
    for color in [
        [0.67, 0.39, 0.41, 1.0],
        [0.64, 0.72, 0.81, 1.0],
        [0.75, 0.75, 0.75, 1.0],
    ] {
        let m = derive(&info("Predeterminado", "IFCWALL", color));
        assert!(
            m.base_color[0] > 0.9 && m.base_color[2] > 0.7,
            "{:?}",
            m.base_color
        );
        let albedo = m.maps.albedo.as_ref().expect("stucco has a texture");
        assert!(matches!(
            &albedo.source,
            crate::features::materials::models::TextureSource::Procedural(p) if p.kind == ProceduralKind::Stucco
        ));
        assert!(m.maps.normal.is_some() && m.roughness > 0.9);
        assert!(m.uv.scale[0] > 0.0 && m.uv.scale[0] < 2.0);
    }
}

#[test]
fn roof_tile_is_terracotta_with_relief() {
    let m = derive(&info("Cubiertas", "IFCSLAB", [0.39, 0.11, 0.11, 1.0]));
    assert!(m.base_color[0] > m.base_color[2] * 2.0);
    assert!(m.maps.normal.is_some() && m.normal_strength > 1.0);
}

#[test]
fn lawn_is_less_saturated_than_a_pure_green() {
    let m = derive(&info("Grama", "IFCSLAB", [0.0, 0.8, 0.0, 1.0]));
    let [r, g, b, _] = m.base_color;
    assert!(g - r < 0.2 && g - b < 0.35, "{:?}", m.base_color);
}

#[test]
fn window_frames_turn_dark_bronze_but_wood_stays_wood() {
    let frame = derive(&info(
        "Revestimiento - Blanco",
        "IFCWINDOW",
        [0.95, 0.95, 0.93, 1.0],
    ));
    assert!(frame.base_color[0] < 0.3 && frame.metallic > 0.3);
    let wood = derive(&info(
        "Madera - Tinte",
        "IFCWINDOW",
        [0.84, 0.58, 0.36, 1.0],
    ));
    assert!(wood.maps.albedo.is_some() && wood.base_color[0] > 0.7);
}

#[test]
fn red_default_steel_becomes_dark_metal() {
    let m = derive(&info(
        "ACERO DE CAJA METALICA",
        "IFCBEAM",
        [0.84, 0.11, 0.09, 1.0],
    ));
    assert!(m.metallic > 0.5 && m.base_color[0] < 0.4);
}

#[test]
fn hidden_materials_are_invisible_and_visible_ones_are_not() {
    assert!(is_hidden(&derive(&info("PS- Relleno", "IFCSLAB", GRAY))));
    assert!(!is_hidden(&derive(&info("Grama", "IFCSLAB", GRAY))));
    assert!(!is_hidden(&derive(&info("Vidrio", "IFCWINDOW", GRAY))));
}

#[test]
fn every_family_has_a_label_key() {
    use std::collections::HashSet;
    let kinds = [
        SurfaceKind::Stucco,
        SurfaceKind::Plaster,
        SurfaceKind::Paint,
        SurfaceKind::RoofTile,
        SurfaceKind::Wood,
        SurfaceKind::Stone,
        SurfaceKind::Brick,
        SurfaceKind::Concrete,
        SurfaceKind::Metal,
        SurfaceKind::Glass,
        SurfaceKind::Ceramic,
        SurfaceKind::Grass,
        SurfaceKind::Fabric,
        SurfaceKind::Plastic,
        SurfaceKind::Soil,
        SurfaceKind::Hidden,
    ];
    let keys: HashSet<_> = kinds.iter().map(|k| k.label_key()).collect();
    assert_eq!(keys.len(), kinds.len());
    for key in keys {
        assert_ne!(
            crate::services::localization::tr(key),
            key,
            "{key} is not translated"
        );
    }
}
