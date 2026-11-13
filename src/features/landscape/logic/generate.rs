//! Procedural terrain: fractal (fBm) heights and automatic soil painting by height and slope.

use super::noise::fbm;
use super::sample::slope_deg_at;
use super::sculpt::normalized_weights;
use crate::features::landscape::models::{FbmSettings, SoilLayer, Terrain};

/// Replaces every height with fBm noise: heights span `base_height ..= base_height + amplitude`.
pub fn generate_fbm(terrain: &mut Terrain, settings: &FbmSettings) {
    let scale = if settings.scale.is_finite() {
        settings.scale.max(1.0)
    } else {
        100.0
    };
    let amplitude = if settings.amplitude.is_finite() {
        settings.amplitude.clamp(0.0, 5000.0)
    } else {
        0.0
    };
    let base = if settings.base_height.is_finite() {
        settings.base_height
    } else {
        0.0
    };
    let lacunarity = settings.lacunarity.clamp(1.1, 4.0);
    let gain = settings.gain.clamp(0.05, 0.95);
    for row in 0..terrain.rows {
        for column in 0..terrain.columns {
            let [x, y, _] = terrain.position(column, row);
            let noise = fbm(
                x / scale,
                y / scale,
                settings.seed,
                settings.octaves,
                lacunarity,
                gain,
            );
            let index = terrain.index(column, row);
            terrain.heights[index] = base + amplitude * (noise * 0.5 + 0.5);
        }
    }
}

/// Repaints every vertex: sand up to `shore_height` meters, rock on slopes steeper than `rock_slope_deg`,
/// dirt on the steep-ish band below that, grass elsewhere. Transitions are blended.
pub fn auto_paint(terrain: &mut Terrain, shore_height: f32, rock_slope_deg: f32) {
    let rock_slope = rock_slope_deg.clamp(1.0, 89.0);
    let dirt_slope = rock_slope * 0.6;
    for row in 0..terrain.rows {
        for column in 0..terrain.columns {
            let [x, y, height] = terrain.position(column, row);
            let slope = slope_deg_at(terrain, x, y);
            let ramp = |value: f32, start: f32, end: f32| {
                ((value - start) / (end - start)).clamp(0.0, 1.0)
            };
            let rock = ramp(slope, rock_slope - 5.0, rock_slope + 5.0);
            let sand = (1.0 - ramp(height, shore_height - 0.5, shore_height + 0.5)) * (1.0 - rock);
            let dirt =
                ramp(slope, dirt_slope - 4.0, dirt_slope + 4.0) * (1.0 - rock) * (1.0 - sand);
            let grass = (1.0 - rock - sand - dirt).max(0.0);
            let weights = [grass, dirt, sand, rock];
            let favored =
                weights
                    .iter()
                    .enumerate()
                    .fold(0, |best, (i, w)| if *w > weights[best] { i } else { best });
            let index = terrain.index(column, row);
            terrain.layers[index] = normalized_weights(weights.map(|w| w * 255.0), favored);
        }
    }
}

/// Layer with the largest weight at a vertex.
pub fn dominant_layer(weights: [u8; 4]) -> SoilLayer {
    let mut best = 0;
    for (index, weight) in weights.iter().enumerate() {
        if *weight > weights[best] {
            best = index;
        }
    }
    SoilLayer::ALL[best]
}
