//! Large imported models in the 3D view: invisible materials are left out and chunks gather neighboring meshes, so
//! frustum culling can skip whole regions.

use crate::features::viewport::logic::frustum::may_be_visible;
use crate::features::viewport::logic::math;
use crate::features::viewport::logic::scene_buffers::{self, CHUNK_TARGET_VERTICES};
use crate::features::viewport::tests::cube;

#[test]
fn invisible_materials_are_not_drawn_or_framed() {
    let meshes = [
        cube([0.0, 0.0, 0.0], 1.0, Some(0)),
        cube([500.0, 0.0, 0.0], 1.0, Some(1)),
    ];
    let colors = [[1.0, 1.0, 1.0, 1.0], [0.5, 0.4, 0.3, 0.0]];
    let buffers = scene_buffers::build(&meshes, &colors);
    assert_eq!(buffers.triangles, 12, "only the visible cube is drawn");
    let bounds = buffers.bounds.expect("bounds");
    assert!(
        bounds.max[0] < 10.0,
        "the hidden cube does not stretch the framing"
    );
}

#[test]
fn meshes_are_grouped_by_place_not_by_file_order() {
    // Two far-apart clusters whose meshes alternate in the file.
    let vertices_per_cube = 8;
    let per_cluster = CHUNK_TARGET_VERTICES / vertices_per_cube * 3 / 2;
    let mut meshes = Vec::new();
    for i in 0..per_cluster {
        let along = (i % 50) as f32 * 0.3;
        meshes.push(cube([along, (i / 50 % 50) as f32 * 0.3, 0.0], 0.2, None));
        meshes.push(cube(
            [1000.0 + along, (i / 50 % 50) as f32 * 0.3, 0.0],
            0.2,
            None,
        ));
    }
    let buffers = scene_buffers::build(&meshes, &[]);
    assert!(buffers.chunks.len() >= 3);
    let wide = buffers
        .chunks
        .iter()
        .filter(|chunk| chunk.bounds.max[0] - chunk.bounds.min[0] > 500.0)
        .count();
    assert!(wide <= 1, "{wide} chunks straddle both clusters");
}

#[test]
fn a_camera_looking_at_one_cluster_skips_the_chunks_of_the_other() {
    let mut meshes = Vec::new();
    for i in 0..30_000 {
        let at = (i % 100) as f32 * 0.3;
        meshes.push(cube([at, (i / 100) as f32 * 0.3, 0.0], 0.2, None));
        meshes.push(cube([2000.0 + at, (i / 100) as f32 * 0.3, 0.0], 0.2, None));
    }
    let buffers = scene_buffers::build(&meshes, &[]);
    let view = math::look_at([15.0, -40.0, 20.0], [15.0, 40.0, 0.0], [0.0, 0.0, 1.0]);
    let projection = math::perspective(0.8, 1.6, 0.5, 500.0);
    let view_proj = math::mul(&projection, &view);
    let seen = buffers
        .chunks
        .iter()
        .filter(|chunk| may_be_visible(&view_proj, &chunk.bounds))
        .count();
    assert!(
        seen > 0 && seen < buffers.chunks.len(),
        "{seen} of {}",
        buffers.chunks.len()
    );
}
