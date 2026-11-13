//! Waves and water meshes.

use crate::features::landscape::logic::water::{
    water_cells, water_mesh, water_mesh_with, wave_height, wave_normal, MAX_WATER_CELLS,
};
use crate::features::landscape::models::WaterBody;

#[test]
fn waves_are_a_pure_function_of_time() {
    let water = WaterBody::ocean([0.0, 0.0]);
    assert_eq!(
        wave_height(&water, 3.0, 4.0, 1.5),
        wave_height(&water, 3.0, 4.0, 1.5)
    );
    assert_ne!(
        wave_height(&water, 3.0, 4.0, 0.0),
        wave_height(&water, 3.0, 4.0, 1.0)
    );
}

#[test]
fn waves_stay_within_the_amplitude() {
    let water = WaterBody::ocean([0.0, 0.0]);
    for step in 0..400 {
        let t = step as f32 * 0.37;
        let h = wave_height(&water, step as f32 * 1.3, step as f32 * 0.7, t);
        assert!((h - water.height).abs() <= water.wave_amplitude + 1e-4);
    }
}

#[test]
fn calm_water_is_flat_with_an_upward_normal() {
    let mut water = WaterBody::plane([0.0, 0.0], 2.0);
    water.wave_amplitude = 0.0;
    assert_eq!(wave_height(&water, 1.0, 1.0, 9.0), 2.0);
    assert_eq!(wave_normal(&water, 1.0, 1.0, 9.0), [0.0, 0.0, 1.0]);
    assert_eq!(water_cells(&water), [1, 1]);
}

#[test]
fn water_mesh_is_a_valid_grid_centered_on_the_body() {
    let water = WaterBody::plane([10.0, -5.0], 1.0);
    let mesh = water_mesh(&water, 0.5);
    assert!(mesh
        .indices
        .iter()
        .all(|i| (*i as usize) < mesh.positions.len()));
    let min_x = mesh
        .positions
        .iter()
        .map(|p| p[0])
        .fold(f32::INFINITY, f32::min);
    assert!((min_x - (10.0 - 15.0)).abs() < 1e-4);
}

#[test]
fn huge_oceans_are_capped() {
    let water = WaterBody::ocean([0.0, 0.0]);
    let cells = water_cells(&water);
    assert!(cells.iter().all(|c| *c <= MAX_WATER_CELLS));
    let mesh = water_mesh_with(&water, 0.0, [100_000, 3]);
    assert_eq!(mesh.positions.len(), (MAX_WATER_CELLS + 1) * 4);
}

#[test]
fn normals_are_unit_length() {
    let water = WaterBody::ocean([0.0, 0.0]);
    let n = wave_normal(&water, 5.0, 7.0, 2.0);
    assert!(((n[0] * n[0] + n[1] * n[1] + n[2] * n[2]) - 1.0).abs() < 1e-5);
}
