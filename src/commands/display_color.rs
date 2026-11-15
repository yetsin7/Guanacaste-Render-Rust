//! The color the real-time 3D view paints a material with. A textured material keeps a white tint in its summary
//! color (the picture carries the real color), so painting the tint alone shows leaves, lawns and wood as white or
//! pale. Here the tint is multiplied by the average of its albedo texture, read once per texture and cached.

use crate::features::materials::logic::procedural::color_at;
use crate::features::materials::models::TextureSource;
use crate::features::projects::models::MaterialDoc;
use std::collections::HashMap;

/// Side of the grid sampled to average a texture.
const SAMPLES: u32 = 16;

/// Averages of albedo textures, keyed by their source.
#[derive(Default)]
pub struct TextureAverages {
    cache: HashMap<String, Option<[f32; 3]>>,
}

impl TextureAverages {
    /// The RGBA color the 3D view shows for `material`.
    pub fn display_color(&mut self, material: &MaterialDoc) -> [f32; 4] {
        let Some(albedo) = material.pbr.as_ref().and_then(|p| p.maps.albedo.as_ref()) else {
            return material.color;
        };
        let key = match &albedo.source {
            TextureSource::File { path } => format!("file:{path}"),
            TextureSource::Procedural(p) => format!("proc:{:?}:{}:{}", p.kind, p.scale, p.seed),
        };
        let average = *self
            .cache
            .entry(key)
            .or_insert_with(|| average_of(&albedo.source));
        match average {
            Some(avg) => {
                let c = material.color;
                [c[0] * avg[0], c[1] * avg[1], c[2] * avg[2], c[3]]
            }
            None => material.color,
        }
    }
}

/// Mean sRGB color of a texture; pixels that are mostly transparent (leaf cut-outs) are left out.
fn average_of(source: &TextureSource) -> Option<[f32; 3]> {
    match source {
        TextureSource::Procedural(procedural) => {
            let mut sum = [0.0f32; 3];
            for y in 0..SAMPLES {
                for x in 0..SAMPLES {
                    let (u, v) = (
                        (x as f32 + 0.5) / SAMPLES as f32,
                        (y as f32 + 0.5) / SAMPLES as f32,
                    );
                    let c = color_at(procedural, u, v);
                    for i in 0..3 {
                        sum[i] += c[i];
                    }
                }
            }
            let n = (SAMPLES * SAMPLES) as f32;
            Some(sum.map(|s| s / n))
        }
        TextureSource::File { path } => {
            let image = if let Some(key) = path.strip_prefix("builtin:") {
                let bytes = crate::features::materials::logic::bundled_textures::image(key)?;
                image::load_from_memory(bytes).ok()?
            } else {
                image::open(path).ok()?
            }
            .thumbnail(64, 64)
            .to_rgba8();
            let (mut sum, mut weight) = ([0.0f32; 3], 0.0f32);
            for pixel in image.pixels() {
                let alpha = f32::from(pixel[3]) / 255.0;
                if alpha < 0.5 {
                    continue;
                }
                for i in 0..3 {
                    sum[i] += f32::from(pixel[i]) / 255.0 * alpha;
                }
                weight += alpha;
            }
            (weight > 0.0).then(|| sum.map(|s| s / weight))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::features::materials::models::{Material, Procedural, ProceduralKind, TextureRef};

    fn textured(source: TextureRef) -> MaterialDoc {
        let mut pbr = Material::default();
        pbr.maps.albedo = Some(source);
        MaterialDoc {
            name: "m".into(),
            color: [1.0, 1.0, 1.0, 1.0],
            pbr: Some(pbr),
        }
    }

    /// Embedded textures retain their representative color in the interactive viewport.
    #[test]
    fn bundled_texture_is_visible_in_the_viewport() {
        let color = TextureAverages::default()
            .display_color(&textured(TextureRef::file("builtin:ambientcg-bricks105")));
        assert!(color[0] > color[1] && color[0] < 0.95, "{color:?}");
    }

    #[test]
    fn a_leaf_picture_paints_green_not_white() {
        let dir = std::env::temp_dir().join(format!("gr-display-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("leaf.png");
        // Green leaf on a transparent background: the background must not wash the color out.
        let image = image::RgbaImage::from_fn(8, 8, |x, _| {
            if x < 4 {
                image::Rgba([40, 110, 30, 255])
            } else {
                image::Rgba([255, 255, 255, 0])
            }
        });
        image.save(&path).unwrap();
        let color = TextureAverages::default()
            .display_color(&textured(TextureRef::file(path.to_string_lossy())));
        assert!(
            color[1] > color[0] && color[1] > color[2],
            "green dominates: {color:?}"
        );
        assert!(color[1] < 0.6, "not washed out: {color:?}");
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn a_procedural_lawn_is_green_and_a_plain_color_stays() {
        let mut averages = TextureAverages::default();
        let lawn = averages.display_color(&textured(TextureRef::procedural(Procedural::new(
            ProceduralKind::Lawn,
        ))));
        assert!(lawn[1] > lawn[0] && lawn[1] > lawn[2], "lawn: {lawn:?}");
        let plain = MaterialDoc {
            name: "p".into(),
            color: [0.2, 0.3, 0.4, 1.0],
            pbr: None,
        };
        assert_eq!(averages.display_color(&plain), [0.2, 0.3, 0.4, 1.0]);
        let missing = textured(TextureRef::file("Z:/no/existe.png"));
        assert_eq!(averages.display_color(&missing), [1.0, 1.0, 1.0, 1.0]);
    }
}
