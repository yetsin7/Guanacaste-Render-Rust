//! Scene buffers, grid and uniform block.

use crate::features::import_export::models::Mesh;
use crate::features::viewport::logic::grid;
use crate::features::viewport::logic::scene_buffers::{self, VERTEX_STRIDE};
use crate::features::viewport::models::Camera;
use crate::features::viewport::services::globals::{Globals, GLOBALS_SIZE};
use crate::features::viewport::tests::{bounds, cube};

fn index_at(bytes: &[u8], i: usize) -> u32 {
    u32::from_ne_bytes(bytes[i * 4..i * 4 + 4].try_into().unwrap())
}

#[test]
fn a_cube_becomes_one_chunk_with_exact_sizes_and_bounds() {
    let buffers = scene_buffers::build(&[cube([1.0, 2.0, 3.0], 2.0, None)], &[]);
    assert_eq!(buffers.chunks.len(), 1);
    let chunk = &buffers.chunks[0];
    assert_eq!((chunk.vertex_count, chunk.index_count), (8, 36));
    assert_eq!(chunk.vertex_bytes.len(), 8 * VERTEX_STRIDE);
    assert_eq!(chunk.index_bytes.len(), 36 * 4);
    assert_eq!(buffers.triangles, 12);
    assert_eq!(
        buffers.bounds,
        Some(bounds([0.0, 1.0, 2.0], [2.0, 3.0, 4.0]))
    );
    assert!(!chunk.transparent);
}

#[test]
fn indices_are_rebased_per_chunk_and_stay_in_range() {
    let meshes = vec![cube([0.0; 3], 1.0, None), cube([5.0, 0.0, 0.0], 1.0, None)];
    let buffers = scene_buffers::build(&meshes, &[]);
    assert_eq!(buffers.chunks.len(), 1);
    let chunk = &buffers.chunks[0];
    assert_eq!(chunk.vertex_count, 16);
    let max = (0..chunk.index_count as usize)
        .map(|i| index_at(&chunk.index_bytes, i))
        .max()
        .unwrap();
    assert_eq!(max, 15);
}

#[test]
fn material_color_is_baked_and_alpha_selects_the_blended_chunks_last() {
    let colors = [[1.0, 0.0, 0.0, 1.0], [0.0, 0.0, 1.0, 0.3]];
    let meshes = vec![
        cube([0.0; 3], 1.0, Some(1)),
        cube([3.0, 0.0, 0.0], 1.0, Some(0)),
    ];
    let buffers = scene_buffers::build(&meshes, &colors);
    assert_eq!(buffers.chunks.len(), 2);
    assert!(!buffers.chunks[0].transparent && buffers.chunks[1].transparent);
    assert_eq!(&buffers.chunks[0].vertex_bytes[12..16], &[255, 0, 0, 255]);
    assert_eq!(&buffers.chunks[1].vertex_bytes[12..16], &[0, 0, 255, 77]);
}

#[test]
fn large_scenes_are_split_into_several_chunks() {
    let big = |x: f32| Mesh {
        positions: (0..250_000).map(|i| [x, i as f32, 0.0]).collect(),
        indices: vec![0, 1, 2],
        material: None,
    };
    let buffers = scene_buffers::build(&[big(0.0), big(1.0), big(2.0)], &[]);
    assert_eq!(buffers.chunks.len(), 3);
    assert_eq!(buffers.vertices, 750_000);
}

#[test]
fn bad_triangles_and_meshes_are_dropped_not_drawn() {
    let mut broken = cube([0.0; 3], 1.0, None);
    broken.indices.extend([0, 1, 99]);
    broken.positions[7] = [f32::NAN, 0.0, 0.0];
    let empty = Mesh::default();
    let buffers = scene_buffers::build(&[broken, empty], &[]);
    assert_eq!(buffers.skipped_meshes, 1);
    assert!(
        buffers.triangles < 12,
        "triangles using the NaN vertex are skipped"
    );
    let b = buffers.bounds.unwrap();
    assert!(b.min.iter().chain(b.max.iter()).all(|v| v.is_finite()));
}

#[test]
fn empty_scene_still_has_a_grid() {
    let buffers = scene_buffers::build(&[], &[]);
    assert!(buffers.chunks.is_empty() && buffers.bounds.is_none());
    assert!(buffers.overlay.grid_vertices > 0);
    assert_eq!(buffers.overlay.axes_vertices, 6);
}

#[test]
fn grid_scales_with_the_model_and_sits_on_the_ground() {
    assert_eq!(grid::nice_step(0.7), 1.0);
    assert_eq!(grid::nice_step(3.0), 5.0);
    assert_eq!(grid::nice_step(12.0), 20.0);
    let small = grid::overlay(Some(&bounds([0.0; 3], [10.0, 10.0, 3.0])));
    let large = grid::overlay(Some(&bounds([0.0; 3], [1000.0, 1000.0, 30.0])));
    assert!(large.step > small.step);
    for overlay in [&small, &large] {
        assert_eq!(
            overlay.bytes.len() as u32,
            (overlay.grid_vertices + overlay.axes_vertices) * 16
        );
        assert_eq!(overlay.grid_vertices % 2, 0);
        assert!(overlay.grid_vertices / 4 <= 401);
    }
    // Model entirely above ground: the grid lies at its lowest point.
    let raised = grid::overlay(Some(&bounds([0.0, 0.0, 10.0], [4.0, 4.0, 12.0])));
    let z = f32::from_ne_bytes(raised.bytes[8..12].try_into().unwrap());
    assert_eq!(z, 10.0);
}

#[test]
fn uniform_block_has_the_wgsl_size_and_carries_the_eye() {
    let camera = Camera::looking_at([1.0, 2.0, 3.0], [0.0; 3]);
    let globals = Globals::new(&camera, 1.5, 20.0);
    let bytes = globals.to_bytes();
    assert_eq!(bytes.len(), GLOBALS_SIZE);
    let float = |i: usize| f32::from_ne_bytes(bytes[i * 4..i * 4 + 4].try_into().unwrap());
    assert_eq!([float(16), float(17), float(18)], [1.0, 2.0, 3.0]);
    let light = [float(20), float(21), float(22)];
    let length = (light[0] * light[0] + light[1] * light[1] + light[2] * light[2]).sqrt();
    assert!((length - 1.0).abs() < 1e-5 && light[2] > 0.0);
}
