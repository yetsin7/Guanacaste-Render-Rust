//! Procedural textures: determinism, range, seamless tiling and recognizable structure.

use super::*;

fn kind(kind: ProceduralKind) -> Procedural {
    Procedural::new(kind)
}

#[test]
fn every_pattern_is_deterministic_and_in_range() {
    for k in ProceduralKind::ALL {
        let p = kind(k);
        for i in 0..40 {
            let (u, v) = (i as f32 * 0.0731, i as f32 * 0.0419);
            let a = color_at(&p, u, v);
            assert_eq!(a, color_at(&p, u, v));
            assert!(a.iter().all(|c| (0.0..=1.0).contains(c)), "{k:?} {a:?}");
        }
    }
}

#[test]
fn every_pattern_tiles_seamlessly() {
    for k in ProceduralKind::ALL {
        for scale in [1.0, 2.0] {
            let p = Procedural { scale, ..kind(k) };
            for i in 0..60 {
                let t = i as f32 * 0.0167 + 0.003;
                let (a, b) = (color_at(&p, t, 0.37), color_at(&p, t + 1.0, 0.37));
                let (c, d) = (color_at(&p, 0.61, t), color_at(&p, 0.61, t + 1.0));
                for channel in 0..3 {
                    assert!((a[channel] - b[channel]).abs() < 1e-3, "{k:?} u seam");
                    assert!((c[channel] - d[channel]).abs() < 1e-3, "{k:?} v seam");
                }
            }
        }
    }
}

#[test]
fn seeds_give_different_variations() {
    for k in ProceduralKind::ALL {
        let a = bake(&Procedural { seed: 1, ..kind(k) }, 32, false);
        let b = bake(&Procedural { seed: 2, ..kind(k) }, 32, false);
        assert_ne!(a, b, "{k:?}");
    }
}

#[test]
fn patterns_look_different_from_each_other() {
    let bakes: Vec<_> = ProceduralKind::ALL
        .iter()
        .map(|k| bake(&kind(*k), 32, false))
        .collect();
    for i in 0..bakes.len() {
        for j in i + 1..bakes.len() {
            assert_ne!(bakes[i], bakes[j]);
        }
    }
}

#[test]
fn bricks_have_lighter_mortar_between_reddish_bricks() {
    let p = kind(ProceduralKind::Brick);
    // The middle of a brick (row 0, brick 0) versus the joint right at the row boundary.
    let body = color_at(&p, 0.125, 0.0625);
    let joint = color_at(&p, 0.125, 0.0);
    assert!(
        body[0] > body[2] * 1.8,
        "brick body should be reddish {body:?}"
    );
    assert!(joint[2] > body[2] + 0.1, "mortar should be paler {joint:?}");
}

#[test]
fn wood_has_gaps_between_planks_and_brown_tones() {
    let p = kind(ProceduralKind::Wood);
    let plank = color_at(&p, 0.125, 0.4);
    let gap = color_at(&p, 0.25, 0.4);
    assert!(plank[0] > plank[2], "wood is warm {plank:?}");
    assert!(gap[0] < plank[0], "gap darker than plank");
}

#[test]
fn grass_is_green_and_noise_is_gray() {
    let g = color_at(&kind(ProceduralKind::Grass), 0.3, 0.6);
    assert!(g[1] > g[0] && g[1] > g[2], "{g:?}");
    let n = color_at(&kind(ProceduralKind::Noise), 0.3, 0.6);
    assert!(n[0] == n[1] && n[1] == n[2]);
}

#[test]
fn marble_is_mostly_light_with_dark_veins() {
    let p = kind(ProceduralKind::Marble);
    let (mut min, mut max) = (1.0_f32, 0.0_f32);
    for i in 0..4096 {
        let l = color_at(&p, (i % 64) as f32 / 64.0, (i / 64) as f32 / 64.0)[0];
        min = min.min(l);
        max = max.max(l);
    }
    assert!(max > 0.8 && min < 0.5, "{min} {max}");
}

#[test]
fn scale_rounds_to_whole_repeats() {
    assert_eq!(
        Procedural {
            scale: 0.2,
            ..kind(ProceduralKind::Noise)
        }
        .scale_cells(),
        1
    );
    assert_eq!(
        Procedural {
            scale: 2.6,
            ..kind(ProceduralKind::Noise)
        }
        .scale_cells(),
        3
    );
    assert_eq!(
        Procedural {
            scale: 99.0,
            ..kind(ProceduralKind::Noise)
        }
        .scale_cells(),
        8
    );
    assert_eq!(
        Procedural {
            scale: f32::NAN,
            ..kind(ProceduralKind::Noise)
        }
        .scale_cells(),
        1
    );
}

#[test]
fn bake_has_the_requested_size_and_grayscale_option() {
    let color = bake(&kind(ProceduralKind::Brick), 16, false);
    assert_eq!(
        (color.width, color.height, color.pixels.len()),
        (16, 16, 256)
    );
    let gray = bake(&kind(ProceduralKind::Brick), 16, true);
    assert!(gray
        .pixels
        .iter()
        .all(|p| p[0] == p[1] && p[1] == p[2] && p[3] == 255));
    assert_ne!(color, gray);
}
