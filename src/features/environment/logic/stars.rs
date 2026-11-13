//! Night sky: a deterministic star field and the moon disc.

use super::clouds::hash;
use super::math::{dot, smoothstep, Vec3};
use crate::features::environment::models::Rgb;

/// Cells per unit of direction; one cell spans about 0.8 degrees.
const GRID: f32 = 70.0;
/// Fraction of cells holding a star.
const STAR_DENSITY: f32 = 0.05;
/// Angular radius of the moon disc in radians (a little larger than the real 0.0045 so it reads in previews).
pub const MOON_RADIUS: f32 = 0.008;

/// Brightness (0..1) and tint of the star at `direction`, if one is there.
pub fn star_at(direction: Vec3) -> Option<(f32, Rgb)> {
    let p = [
        direction[0] * GRID,
        direction[1] * GRID,
        direction[2] * GRID,
    ];
    let cell = [
        p[0].floor() as i32,
        p[1].floor() as i32,
        p[2].floor() as i32,
    ];
    if hash(cell[0], cell[1], cell[2]) > STAR_DENSITY {
        return None;
    }
    let jitter = |salt: i32| 0.3 + 0.4 * hash(cell[0] + salt, cell[1] - salt, cell[2] + 3 * salt);
    let center = [
        cell[0] as f32 + jitter(11),
        cell[1] as f32 + jitter(23),
        cell[2] as f32 + jitter(37),
    ];
    let offset = [p[0] - center[0], p[1] - center[1], p[2] - center[2]];
    let distance = dot(offset, offset).sqrt();
    let falloff = 1.0 - smoothstep(0.0, 0.3, distance);
    if falloff <= 0.0 {
        return None;
    }
    let magnitude = hash(cell[0] + 5, cell[1] + 9, cell[2] + 13);
    let warmth = hash(cell[0] - 3, cell[1] + 1, cell[2] - 7);
    let tint = Rgb::new(0.85 + 0.15 * warmth, 0.92, 1.0 - 0.15 * warmth);
    Some((falloff * (0.25 + 0.75 * magnitude * magnitude), tint))
}

/// Brightness of the moon disc at angular distance `angle` (radians) from its center: 1 inside, soft edge.
pub fn moon_disc(angle: f32) -> f32 {
    1.0 - smoothstep(MOON_RADIUS * 0.85, MOON_RADIUS, angle)
}
