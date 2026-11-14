//! Site measuring: focus bounds without strays or hidden pieces, ground level, and the main facade.

use super::framing::SOUTH;
use super::site::measure;
use crate::features::import_export::models::Mesh;

/// A closed box as a mesh (12 triangles).
fn block(min: [f32; 3], max: [f32; 3]) -> Mesh {
    let mut positions = Vec::new();
    for z in [min[2], max[2]] {
        for (x, y) in [
            (min[0], min[1]),
            (max[0], min[1]),
            (max[0], max[1]),
            (min[0], max[1]),
        ] {
            positions.push([x, y, z]);
        }
    }
    let indices = vec![
        0, 1, 2, 0, 2, 3, 4, 6, 5, 4, 7, 6, 0, 4, 5, 0, 5, 1, 1, 5, 6, 1, 6, 2, 2, 6, 7, 2, 7, 3,
        3, 7, 4, 3, 4, 0,
    ];
    Mesh {
        positions,
        indices,
        material: None,
        uvs: Vec::new(),
    }
}

#[test]
fn far_away_strays_do_not_stretch_the_focus() {
    let mut meshes: Vec<Mesh> = (0..20)
        .map(|i| block([i as f32 * 0.5, 0.0, 0.0], [i as f32 * 0.5 + 0.4, 6.0, 3.0]))
        .collect();
    meshes.push(block([400.0, 300.0, 0.0], [401.0, 301.0, 1.0]));
    let visible = vec![true; meshes.len()];
    let focus = measure(&meshes, &visible, &[]).focus.expect("focus");
    assert!(focus.max[0] < 20.0 && focus.max[1] < 20.0, "{focus:?}");
}

#[test]
fn hidden_meshes_are_not_measured() {
    let meshes = vec![
        block([0.0; 3], [5.0, 5.0, 3.0]),
        block([-50.0; 3], [50.0, 50.0, 12.0]),
    ];
    let focus = measure(&meshes, &[true, false], &[]).focus.expect("focus");
    assert_eq!(focus.max, [5.0, 5.0, 3.0]);
}

#[test]
fn the_ground_is_where_the_big_level_surfaces_are() {
    // A big floor at 2 m and a small buried footing at 0 m.
    let meshes = vec![
        block([0.0, 0.0, 1.9], [20.0, 20.0, 2.0]),
        block([0.0, 0.0, 0.0], [1.0, 1.0, 0.1]),
    ];
    let site = measure(&meshes, &[true, true], &[]);
    assert!((site.ground_z - 1.95).abs() < 0.15, "{}", site.ground_z);
}

#[test]
fn the_facade_is_the_side_with_most_openings() {
    let mut meshes = vec![block([0.0, 0.0, 0.0], [10.0, 8.0, 3.0])];
    // Three windows on the +X side, one on the south side.
    for (a, b) in [
        ([10.0, 1.0, 1.0], [10.1, 2.0, 2.0]),
        ([10.0, 3.0, 1.0], [10.1, 4.0, 2.0]),
        ([10.0, 5.0, 1.0], [10.1, 6.0, 2.0]),
        ([4.0, -0.1, 1.0], [5.0, 0.0, 2.0]),
    ] {
        meshes.push(block(a, b));
    }
    let visible = vec![true; meshes.len()];
    let site = measure(&meshes, &visible, &[1, 2, 3, 4]);
    assert!(site.facade.abs() < 0.01, "toward +X, got {}", site.facade);
    // Without openings the south wins.
    let plain = measure(&meshes[..1], &[true], &[]);
    assert!((plain.facade.sin() - SOUTH.sin()).abs() < 0.01 && plain.facade.cos().abs() < 0.01);
}
