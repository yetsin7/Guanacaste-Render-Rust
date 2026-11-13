//! Brush strokes over the terrain: raise, lower, flatten, smooth, noise and soil-layer painting.

use super::noise::value_noise;
use crate::features::landscape::models::terrain::MAX_ABS_HEIGHT;
use crate::features::landscape::models::{Brush, SoilLayer, Terrain, Tool};

/// Meters per second a full-strength raise or lower dab moves the center of the brush.
pub const RAISE_RATE: f32 = 6.0;
/// Fraction per second a full-strength flatten, smooth or paint dab closes the gap to its target.
pub const BLEND_RATE: f32 = 6.0;

/// Brush weight (0..=1) at `distance` meters from the center: 1 inside the hard core, then a smooth
/// fall to 0 at the radius.
pub fn falloff_weight(brush: &Brush, distance: f32) -> f32 {
    let radius = brush.radius.max(f32::EPSILON);
    let d = distance / radius;
    if d >= 1.0 {
        return 0.0;
    }
    let soft = brush.falloff.clamp(0.0, 1.0);
    let core = 1.0 - soft;
    if d <= core || soft <= f32::EPSILON {
        return 1.0;
    }
    let t = (d - core) / soft;
    1.0 - t * t * (3.0 - 2.0 * t)
}

/// Vertices touched by a brush at world `(x, y)` with their weights: `(index, column, row, weight)`.
fn footprint(terrain: &Terrain, brush: &Brush, x: f32, y: f32) -> Vec<(usize, usize, usize, f32)> {
    let radius = brush.radius.max(0.0);
    let to_grid = |value: f32, origin: f32| (value - origin) / terrain.spacing;
    let span = |low: f32, high: f32, count: usize| {
        let first = low.ceil().max(0.0) as usize;
        let last = high.floor().min((count - 1) as f32);
        (first, last)
    };
    let (c_first, c_last) = span(
        to_grid(x - radius, terrain.origin[0]),
        to_grid(x + radius, terrain.origin[0]),
        terrain.columns,
    );
    let (r_first, r_last) = span(
        to_grid(y - radius, terrain.origin[1]),
        to_grid(y + radius, terrain.origin[1]),
        terrain.rows,
    );
    let mut found = Vec::new();
    if c_last < 0.0 || r_last < 0.0 {
        return found;
    }
    for row in r_first..=r_last as usize {
        for column in c_first..=c_last as usize {
            let [px, py, _] = terrain.position(column, row);
            let weight = falloff_weight(brush, ((px - x).powi(2) + (py - y).powi(2)).sqrt());
            if weight > 0.0 {
                found.push((terrain.index(column, row), column, row, weight));
            }
        }
    }
    found
}

/// Average height of the 3 × 3 neighborhood of a vertex.
fn neighborhood_mean(terrain: &Terrain, column: usize, row: usize) -> f32 {
    let mut sum = 0.0;
    let mut count = 0.0;
    for r in row.saturating_sub(1)..=(row + 1).min(terrain.rows - 1) {
        for c in column.saturating_sub(1)..=(column + 1).min(terrain.columns - 1) {
            sum += terrain.heights[terrain.index(c, r)];
            count += 1.0;
        }
    }
    sum / count
}

/// Applies one dab of a sculpting `tool` lasting `dt` seconds at world `(x, y)`.
/// `salt` varies the noise between strokes. Returns `true` when any height changed.
pub fn sculpt(
    terrain: &mut Terrain,
    tool: Tool,
    brush: &Brush,
    x: f32,
    y: f32,
    dt: f32,
    salt: u64,
) -> bool {
    let strength = brush.strength.clamp(0.0, 1.0) * dt.clamp(0.0, 1.0);
    if strength <= 0.0 || !tool.sculpts() {
        return false;
    }
    let touched = footprint(terrain, brush, x, y);
    let blend = |weight: f32| (weight * strength * BLEND_RATE).min(1.0);
    let targets: Vec<f32> = touched
        .iter()
        .map(|&(index, column, row, weight)| {
            let height = terrain.heights[index];
            match tool {
                Tool::Raise => height + weight * strength * RAISE_RATE,
                Tool::Lower => height - weight * strength * RAISE_RATE,
                Tool::Flatten => height + (brush.target_height - height) * blend(weight),
                Tool::Smooth => {
                    height + (neighborhood_mean(terrain, column, row) - height) * blend(weight)
                }
                _ => {
                    let [px, py, _] = terrain.position(column, row);
                    let scale = brush.noise_scale.max(0.1);
                    let noise = value_noise(px / scale, py / scale, brush.seed ^ salt);
                    height + noise * weight * strength * RAISE_RATE
                }
            }
        })
        .collect();
    let mut changed = false;
    for (&(index, ..), target) in touched.iter().zip(targets) {
        let target = target.clamp(-MAX_ABS_HEIGHT, MAX_ABS_HEIGHT);
        if target != terrain.heights[index] {
            terrain.heights[index] = target;
            changed = true;
        }
    }
    changed
}

/// Paints one dab of `layer` lasting `dt` seconds at world `(x, y)`. Weights of every vertex keep summing to 255.
/// Returns `true` when any weight changed.
pub fn paint(
    terrain: &mut Terrain,
    layer: SoilLayer,
    brush: &Brush,
    x: f32,
    y: f32,
    dt: f32,
) -> bool {
    let strength = brush.strength.clamp(0.0, 1.0) * dt.clamp(0.0, 1.0);
    if strength <= 0.0 {
        return false;
    }
    let channel = layer.index();
    let mut changed = false;
    for (index, _, _, weight) in footprint(terrain, brush, x, y) {
        let amount = (weight * strength * BLEND_RATE).min(1.0);
        let old = terrain.layers[index];
        let mut blended = [0.0f32; 4];
        for (slot, (value, target)) in blended.iter_mut().zip(old.iter().zip(0..4)) {
            let goal = if target == channel { 255.0 } else { 0.0 };
            let current = f32::from(*value);
            *slot = current + (goal - current) * amount;
        }
        let new = normalized_weights(blended, channel);
        if new != old {
            terrain.layers[index] = new;
            changed = true;
        }
    }
    changed
}

/// Rounds `weights` to bytes summing exactly to 255; the remainder goes to `favored`.
pub fn normalized_weights(weights: [f32; 4], favored: usize) -> [u8; 4] {
    let total: f32 = weights.iter().map(|w| w.max(0.0)).sum();
    if total <= f32::EPSILON {
        let mut out = [0; 4];
        out[favored.min(3)] = 255;
        return out;
    }
    let mut out = weights.map(|w| (w.max(0.0) / total * 255.0).floor() as u8);
    let sum: u32 = out.iter().map(|w| u32::from(*w)).sum();
    let favored = favored.min(3);
    out[favored] = out[favored].saturating_add((255 - sum.min(255)) as u8);
    out
}
