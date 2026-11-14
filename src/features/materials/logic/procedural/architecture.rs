//! Architectural patterns: sand-finish stucco, clay barrel roof tile and cut stone blocks. They are pale and
//! almost neutral on purpose: the material's base color tints them, so one pattern serves every paint color.

use super::noise::{fbm, hash, value_noise};

fn smoothstep(edge0: f32, edge1: f32, x: f32) -> f32 {
    let t = ((x - edge0) / (edge1 - edge0)).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

/// Sand-finish stucco: fine grain, faint trowel mottling and a few darker pits.
pub fn stucco(u: f32, v: f32, n: i32, seed: u32) -> [f32; 3] {
    let mottling = fbm(u, v, 2 * n, 4, seed);
    let grain = value_noise(u, v, 96 * n, 96 * n, seed ^ 0x31);
    let fine = value_noise(u, v, 192 * n, 192 * n, seed ^ 0x32);
    let pit = smoothstep(0.86, 0.96, value_noise(u, v, 48 * n, 48 * n, seed ^ 0x33)) * 0.12;
    let level = 0.9 + (mottling - 0.5) * 0.14 + (grain - 0.5) * 0.10 + (fine - 0.5) * 0.06 - pit;
    let l = level.clamp(0.0, 1.0);
    [l, l * 0.99, l * 0.97]
}

/// Spanish barrel tile: `6 * n` rounded columns and `4 * n` overlapping rows per tile, each tile a little
/// different, the lower edge of every row lit and the upper end shaded by the row above.
pub fn roof_tile(u: f32, v: f32, n: i32, seed: u32) -> [f32; 3] {
    let (columns, rows) = (6 * n, 4 * n);
    let x = u.rem_euclid(1.0) * columns as f32;
    let y = v.rem_euclid(1.0) * rows as f32;
    let (column, row) = (x.floor() as i32, y.floor() as i32);
    let (fx, fy) = (x - x.floor(), y - y.floor());
    // Every other row is shifted half a tile, as laid on a real roof.
    let shifted = if row % 2 == 0 { fx } else { (fx + 0.5).fract() };
    let id = if row % 2 == 0 {
        column
    } else {
        ((x + 0.5).floor() as i32).rem_euclid(columns)
    };
    let round = (shifted * std::f32::consts::PI).sin();
    let tilt = (shifted * std::f32::consts::PI).cos();
    let across = 0.62 + 0.28 * round - 0.16 * tilt;
    let overlap = 0.55 + 0.45 * smoothstep(0.0, 0.42, fy);
    let lip = 1.0 + 0.12 * smoothstep(0.9, 0.99, fy);
    let gap = 0.7 + 0.3 * smoothstep(0.0, 0.08, shifted.min(1.0 - shifted));
    let tone = 0.86 + 0.28 * hash(id, row, seed);
    let weather = 0.9 + 0.2 * fbm(u, v, 5 * n, 4, seed ^ 0x61);
    let l = (across * overlap * lip * gap * tone * weather).clamp(0.0, 1.0);
    [l, l * 0.97, l * 0.94]
}

/// Cut stone blocks laid in staggered courses with dark joints: `5 * n` courses of `3 * n` blocks per tile.
pub fn stone_block(u: f32, v: f32, n: i32, seed: u32) -> [f32; 3] {
    let (courses, columns) = (5 * n, 3 * n);
    let y = v.rem_euclid(1.0) * courses as f32;
    let row = y.floor() as i32;
    let fy = y - y.floor();
    let shift = if row % 2 == 0 { 0.0 } else { 0.5 };
    let x = u.rem_euclid(1.0) * columns as f32 + shift;
    let id = (x.floor() as i32).rem_euclid(columns);
    let fx = x - x.floor();
    let joint =
        smoothstep(0.0, 0.05, fx.min(1.0 - fx) * 0.5).min(smoothstep(0.0, 0.06, fy.min(1.0 - fy)));
    let tone = 0.8 + 0.2 * hash(id, row, seed);
    let veins = fbm(u, v, 6 * n, 4, seed ^ 0x71);
    let pits = value_noise(u, v, 80 * n, 80 * n, seed ^ 0x72);
    let face = tone * (0.9 + 0.2 * veins) * (1.0 - 0.1 * smoothstep(0.8, 0.95, pits));
    let l = (face * (0.45 + 0.55 * joint)).clamp(0.0, 1.0);
    [l, l * 0.98, l * 0.94]
}

/// A calm, well-kept lawn: soft green with mowing patches and a few dry spots, much paler than a saturated grass.
pub fn lawn(u: f32, v: f32, n: i32, seed: u32) -> [f32; 3] {
    let blades = value_noise(u, v, 140 * n, 140 * n, seed);
    let fine = value_noise(u, v, 260 * n, 260 * n, seed ^ 0x2);
    let patches = fbm(u, v, 3 * n, 4, seed ^ 0x9);
    let dry = smoothstep(0.6, 0.85, fbm(u, v, 5 * n, 3, seed ^ 0x44));
    let green = [
        0.30 + 0.10 * blades + 0.04 * fine,
        0.39 + 0.09 * blades + 0.05 * fine,
        0.19 + 0.06 * blades,
    ];
    let shade = 0.78 + 0.44 * patches;
    let dried = [0.55, 0.55, 0.32];
    std::array::from_fn(|i| {
        ((green[i] + (dried[i] - green[i]) * dry * 0.65) * shade).clamp(0.0, 1.0)
    })
}
