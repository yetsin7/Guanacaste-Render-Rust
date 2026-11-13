//! The pattern functions: each maps a point of the unit tile to an sRGB color and tiles seamlessly.

use super::noise::{fbm, hash, value_noise};

/// Linear interpolation of two colors.
fn mix(a: [f32; 3], b: [f32; 3], t: f32) -> [f32; 3] {
    let t = t.clamp(0.0, 1.0);
    [
        a[0] + (b[0] - a[0]) * t,
        a[1] + (b[1] - a[1]) * t,
        a[2] + (b[2] - a[2]) * t,
    ]
}

fn scaled(color: [f32; 3], factor: f32) -> [f32; 3] {
    [color[0] * factor, color[1] * factor, color[2] * factor]
}

fn smoothstep(edge0: f32, edge1: f32, x: f32) -> f32 {
    let t = ((x - edge0) / (edge1 - edge0)).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

/// Running-bond brick wall: `n * 8` rows of `n * 4` bricks per tile.
pub fn brick(u: f32, v: f32, n: i32, seed: u32) -> [f32; 3] {
    let (rows, columns) = (8 * n, 4 * n);
    let row = (v.rem_euclid(1.0) * rows as f32).floor() as i32;
    let shift = if row % 2 == 0 { 0.0 } else { 0.5 };
    let x = u.rem_euclid(1.0) * columns as f32 + shift;
    let id = (x.floor() as i32).rem_euclid(columns);
    let (fx, fy) = (x - x.floor(), (v.rem_euclid(1.0) * rows as f32).fract());
    let edge = (fx.min(1.0 - fx) * 2.0 / 0.16).min(fy.min(1.0 - fy) / 0.08);
    let body = smoothstep(0.0, 1.0, edge);
    let tone = 0.8 + 0.4 * hash(id, row, seed);
    let grain = 0.88 + 0.24 * fbm(u, v, 16 * n, 3, seed ^ 0x51);
    let brick = scaled([0.62, 0.27, 0.19], tone * grain);
    let mortar = scaled(
        [0.66, 0.64, 0.60],
        0.9 + 0.2 * fbm(u, v, 24 * n, 2, seed ^ 0x77),
    );
    mix(mortar, brick, body)
}

/// Wooden planks: `n * 4` planks across the tile with stretched grain.
pub fn wood(u: f32, v: f32, n: i32, seed: u32) -> [f32; 3] {
    let planks = 4 * n;
    let plank = (u.rem_euclid(1.0) * planks as f32).floor() as i32;
    let local = (u.rem_euclid(1.0) * planks as f32).fract();
    let warp = fbm(u, v, 2 * n, 3, seed);
    let rings = ((local * 5.0 + warp * 6.0 + hash(plank, 0, seed) * 9.0).fract() - 0.5).abs() * 2.0;
    let streaks = value_noise(u, v, 40 * n, 3, seed ^ 0xA1);
    let tone = 0.5 * streaks + 0.5 * rings;
    let base = mix([0.36, 0.21, 0.10], [0.66, 0.45, 0.26], tone);
    let plank_tint = 0.88 + 0.24 * hash(plank, 1, seed);
    let gap = smoothstep(0.0, 0.025, local.min(1.0 - local));
    scaled(base, plank_tint * (0.35 + 0.65 * gap))
}

/// Poured concrete: soft stains, fine grain and small dark pores.
pub fn concrete(u: f32, v: f32, n: i32, seed: u32) -> [f32; 3] {
    let stains = fbm(u, v, 3 * n, 4, seed);
    let grain = value_noise(u, v, 96 * n, 96 * n, seed ^ 0x33);
    let pore = value_noise(u, v, 64 * n, 64 * n, seed ^ 0x99);
    let level = 0.52 + (stains - 0.5) * 0.22 + (grain - 0.5) * 0.10;
    let pit = smoothstep(0.88, 0.97, pore) * 0.35;
    let v = (level - pit).clamp(0.0, 1.0);
    [v, v * 0.99, v * 0.96]
}

/// White marble with dark meandering veins.
pub fn marble(u: f32, v: f32, n: i32, seed: u32) -> [f32; 3] {
    let warp = fbm(u, v, 2 * n, 5, seed);
    let phase = (2 * n) as f32 * u + (3 * n) as f32 * v + warp * 3.5;
    let vein = 1.0 - (phase * std::f32::consts::PI).sin().abs();
    let sharp = vein.powi(10);
    let cloud = fbm(u, v, 4 * n, 3, seed ^ 0x5) * 0.06;
    let base = mix([0.90, 0.90, 0.88], [0.80, 0.81, 0.82], cloud * 6.0);
    mix(base, [0.30, 0.32, 0.36], sharp * 0.85)
}

/// Short grass: vertical blades over patches of lighter and darker turf.
pub fn grass(u: f32, v: f32, n: i32, seed: u32) -> [f32; 3] {
    let blades = value_noise(u, v, 110 * n, 18 * n, seed);
    let fine = value_noise(u, v, 200 * n, 40 * n, seed ^ 0x2);
    let patches = fbm(u, v, 3 * n, 3, seed ^ 0x9);
    let green = mix(
        [0.10, 0.25, 0.05],
        [0.36, 0.55, 0.14],
        blades * 0.7 + fine * 0.3,
    );
    mix(
        green,
        [0.45, 0.50, 0.17],
        smoothstep(0.55, 0.85, patches) * 0.5,
    )
}

/// Plain grayscale fractal noise.
pub fn noise(u: f32, v: f32, n: i32, seed: u32) -> [f32; 3] {
    let value = fbm(u, v, 4 * n, 5, seed);
    [value; 3]
}
