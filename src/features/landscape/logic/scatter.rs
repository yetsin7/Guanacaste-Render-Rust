//! Vegetation scattering: seeded placement with density, random scale and rotation, honoring slope and height.

use super::noise::{derive_seed, Rng};
use super::sample::{height_at, slope_deg_at};
use crate::features::landscape::models::{
    ScatterSettings, Terrain, VegetationInstance, MAX_INSTANCES,
};
use std::f32::consts::TAU;

/// Smallest distance in meters kept between plants for a density of `density` plants per 100 m².
pub fn min_spacing(density: f32) -> f32 {
    if density <= 0.0 {
        return f32::INFINITY;
    }
    (100.0 / density).sqrt() * 0.5
}

/// Whether a plant may grow at world `(x, y)`.
pub fn accepts(terrain: &Terrain, settings: &ScatterSettings, x: f32, y: f32) -> bool {
    terrain.contains(x, y)
        && height_at(terrain, x, y) >= settings.min_height
        && slope_deg_at(terrain, x, y) <= settings.max_slope_deg
}

fn too_close(
    existing: &[VegetationInstance],
    added: &[VegetationInstance],
    x: f32,
    y: f32,
    spacing: f32,
) -> bool {
    let limit = spacing * spacing;
    existing.iter().chain(added).any(|plant| {
        let (dx, dy) = (plant.position[0] - x, plant.position[1] - y);
        dx * dx + dy * dy < limit
    })
}

/// Tries `attempts` random spots in a disk of `radius` meters around `(x, y)` and returns the accepted plants.
/// The result depends only on the inputs and `seed`. Plants keep [`min_spacing`] from `existing` and each other,
/// and the total never passes [`MAX_INSTANCES`].
pub fn scatter_in_circle(
    terrain: &Terrain,
    settings: &ScatterSettings,
    existing: &[VegetationInstance],
    center: [f32; 2],
    radius: f32,
    attempts: usize,
    seed: u64,
) -> Vec<VegetationInstance> {
    let mut rng = Rng::new(derive_seed(settings.seed, seed));
    let spacing = min_spacing(settings.density);
    let room = MAX_INSTANCES.saturating_sub(existing.len());
    let mut added = Vec::new();
    for _ in 0..attempts {
        if added.len() >= room {
            break;
        }
        let distance = radius.max(0.0) * rng.next_f32().sqrt();
        let angle = rng.next_f32() * TAU;
        let rotation_deg = rng.range(0.0, 360.0);
        let scale = rng.range(
            settings.scale_min.min(settings.scale_max),
            settings.scale_max.max(settings.scale_min),
        );
        let (x, y) = (
            center[0] + distance * angle.cos(),
            center[1] + distance * angle.sin(),
        );
        if !accepts(terrain, settings, x, y) || too_close(existing, &added, x, y, spacing) {
            continue;
        }
        added.push(VegetationInstance {
            asset: settings.asset.clone(),
            position: [x, y, height_at(terrain, x, y)],
            rotation_deg,
            scale: scale.max(0.01),
        });
    }
    added
}

/// Number of attempts that fill an area of `area` m² at `density` plants per 100 m².
pub fn attempts_for(area: f32, density: f32) -> usize {
    let wanted = (area.max(0.0) * density.max(0.0) / 100.0).round();
    (wanted.min(MAX_INSTANCES as f32)) as usize
}

/// Scatters plants over the whole terrain at the configured density, adding to `existing`.
pub fn scatter_everywhere(
    terrain: &Terrain,
    settings: &ScatterSettings,
    existing: &[VegetationInstance],
) -> Vec<VegetationInstance> {
    let [width, depth] = terrain.size();
    let mut rng = Rng::new(derive_seed(settings.seed, 0xA11));
    let spacing = min_spacing(settings.density);
    let room = MAX_INSTANCES.saturating_sub(existing.len());
    let mut added: Vec<VegetationInstance> = Vec::new();
    for _ in 0..attempts_for(width * depth, settings.density) {
        if added.len() >= room {
            break;
        }
        let x = terrain.origin[0] + rng.next_f32() * width;
        let y = terrain.origin[1] + rng.next_f32() * depth;
        let rotation_deg = rng.range(0.0, 360.0);
        let scale = rng.range(
            settings.scale_min.min(settings.scale_max),
            settings.scale_max.max(settings.scale_min),
        );
        if !accepts(terrain, settings, x, y) || too_close(existing, &added, x, y, spacing) {
            continue;
        }
        added.push(VegetationInstance {
            asset: settings.asset.clone(),
            position: [x, y, height_at(terrain, x, y)],
            rotation_deg,
            scale: scale.max(0.01),
        });
    }
    added
}

/// Removes the plants within `radius` meters of `center`; returns how many were removed.
pub fn erase_in_circle(
    plants: &mut Vec<VegetationInstance>,
    center: [f32; 2],
    radius: f32,
) -> usize {
    let before = plants.len();
    let limit = radius * radius;
    plants.retain(|plant| {
        let (dx, dy) = (plant.position[0] - center[0], plant.position[1] - center[1]);
        dx * dx + dy * dy > limit
    });
    before - plants.len()
}

/// Puts every plant back on the ground after the terrain changed.
pub fn reseat(terrain: &Terrain, plants: &mut [VegetationInstance]) {
    for plant in plants {
        plant.position[2] = height_at(terrain, plant.position[0], plant.position[1]);
    }
}
