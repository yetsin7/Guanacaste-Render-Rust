//! Vegetation scattering.

use crate::features::landscape::logic::sample::{height_at, slope_deg_at};
use crate::features::landscape::logic::scatter::{
    erase_in_circle, min_spacing, reseat, scatter_everywhere, scatter_in_circle,
};
use crate::features::landscape::models::{ScatterSettings, Terrain, MAX_INSTANCES};

fn hill() -> Terrain {
    let mut terrain = Terrain::flat(65, 65, 1.0);
    for row in 0..65 {
        for column in 0..65 {
            let index = terrain.index(column, row);
            // Flat plateau on the left, a steep wall on the right.
            terrain.heights[index] = if column < 40 {
                1.0
            } else {
                1.0 + (column - 40) as f32 * 3.0
            };
        }
    }
    terrain
}

#[test]
fn scattering_is_deterministic_for_a_seed() {
    let terrain = Terrain::flat(65, 65, 1.0);
    let settings = ScatterSettings {
        min_height: -1.0,
        ..ScatterSettings::default()
    };
    let a = scatter_everywhere(&terrain, &settings, &[]);
    let b = scatter_everywhere(&terrain, &settings, &[]);
    assert_eq!(a, b);
    assert!(!a.is_empty());
    let other = ScatterSettings {
        seed: 99,
        ..settings
    };
    assert_ne!(a, scatter_everywhere(&terrain, &other, &[]));
}

#[test]
fn plants_respect_slope_height_and_spacing() {
    let terrain = hill();
    let settings = ScatterSettings {
        density: 4.0,
        max_slope_deg: 30.0,
        min_height: 0.5,
        ..ScatterSettings::default()
    };
    let plants = scatter_everywhere(&terrain, &settings, &[]);
    assert!(!plants.is_empty());
    for plant in &plants {
        let [x, y, z] = plant.position;
        assert!(slope_deg_at(&terrain, x, y) <= 30.0 + 1e-3);
        assert!((z - height_at(&terrain, x, y)).abs() < 1e-4);
        assert!((settings.scale_min..=settings.scale_max).contains(&plant.scale));
        assert!((0.0..360.0).contains(&plant.rotation_deg));
    }
    let gap = min_spacing(settings.density);
    for (i, a) in plants.iter().enumerate() {
        for b in &plants[i + 1..] {
            let d = ((a.position[0] - b.position[0]).powi(2)
                + (a.position[1] - b.position[1]).powi(2))
            .sqrt();
            assert!(d >= gap - 1e-4);
        }
    }
}

#[test]
fn nothing_grows_below_the_minimum_height() {
    let terrain = Terrain::flat(33, 33, 1.0);
    let settings = ScatterSettings {
        min_height: 5.0,
        ..ScatterSettings::default()
    };
    assert!(scatter_everywhere(&terrain, &settings, &[]).is_empty());
}

#[test]
fn circle_scatter_stays_inside_and_erase_removes() {
    let terrain = Terrain::flat(65, 65, 1.0);
    let settings = ScatterSettings {
        density: 3.0,
        min_height: -1.0,
        ..ScatterSettings::default()
    };
    let mut plants = scatter_in_circle(&terrain, &settings, &[], [5.0, 5.0], 6.0, 200, 1);
    assert!(!plants.is_empty());
    assert!(plants
        .iter()
        .all(|p| ((p.position[0] - 5.0).powi(2) + (p.position[1] - 5.0).powi(2)).sqrt() <= 6.001));
    let removed = erase_in_circle(&mut plants, [5.0, 5.0], 100.0);
    assert!(removed > 0 && plants.is_empty());
}

#[test]
fn instance_count_is_capped() {
    let terrain = Terrain::flat(65, 65, 1.0);
    let settings = ScatterSettings {
        min_height: -1.0,
        ..ScatterSettings::default()
    };
    let full = vec![Default::default(); MAX_INSTANCES];
    assert!(scatter_everywhere(&terrain, &settings, &full).is_empty());
}

#[test]
fn reseat_follows_the_terrain() {
    let mut terrain = Terrain::flat(9, 9, 1.0);
    let settings = ScatterSettings {
        min_height: -1.0,
        density: 2.0,
        ..ScatterSettings::default()
    };
    let mut plants = scatter_everywhere(&terrain, &settings, &[]);
    terrain.heights.iter_mut().for_each(|h| *h = 4.0);
    reseat(&terrain, &mut plants);
    assert!(plants.iter().all(|p| p.position[2] == 4.0));
}
