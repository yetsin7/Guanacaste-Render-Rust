//! Conversion between PBR materials and the color-only materials of an imported scene.

use crate::features::import_export::models::Material as SceneMaterial;
use crate::features::materials::models::Material;

/// The PBR material for an imported one: a diffuse material of that color.
pub fn from_scene(material: &SceneMaterial) -> Material {
    Material::from_color(material.name.clone(), material.color)
}

/// The color-only material of `material`: its base color with the opacity (alpha times 1 - transparency) as alpha.
pub fn to_scene(material: &Material) -> SceneMaterial {
    let c = material.base_color;
    SceneMaterial {
        name: material.name.clone(),
        color: [c[0], c[1], c[2], c[3] * (1.0 - material.transparency)],
        texture: None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scene_materials_round_trip_exactly() {
        for color in [
            [0.1, 0.2, 0.3, 1.0],
            [0.82, 0.7, 0.31, 0.3],
            [0.0, 1.0, 0.5, 0.0],
        ] {
            let scene = SceneMaterial {
                name: "Muro".into(),
                color,
                texture: None,
            };
            assert_eq!(to_scene(&from_scene(&scene)), scene);
        }
    }

    #[test]
    fn transparency_lowers_the_exported_alpha() {
        let mut m = Material::new("Vidrio");
        m.transparency = 0.75;
        assert!((to_scene(&m).color[3] - 0.25).abs() < 1e-6);
    }

    #[test]
    fn out_of_range_imported_colors_are_clamped() {
        let m = from_scene(&SceneMaterial {
            name: "x".into(),
            color: [2.0, -1.0, f32::NAN, 5.0],
            texture: None,
        });
        assert_eq!(m.base_color, [1.0, 0.0, 0.8, 1.0]);
    }
}
