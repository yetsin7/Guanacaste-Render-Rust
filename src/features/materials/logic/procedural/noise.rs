//! Seamlessly tiling value noise: every function takes coordinates in the unit tile and integer lattice periods, so
//! the result at `u` and `u + 1` is identical.

/// A deterministic hash of lattice point `(x, y)` and `seed` into 0..1.
pub fn hash(x: i32, y: i32, seed: u32) -> f32 {
    let mut h = (x as u32).wrapping_mul(0x9E37_79B1)
        ^ (y as u32).wrapping_mul(0x85EB_CA77)
        ^ seed.wrapping_mul(0xC2B2_AE3D);
    h ^= h >> 15;
    h = h.wrapping_mul(0x2C1B_3C6D);
    h ^= h >> 12;
    h = h.wrapping_mul(0x297A_2D39);
    h ^= h >> 15;
    (h >> 8) as f32 / 16_777_216.0
}

fn smooth(t: f32) -> f32 {
    t * t * (3.0 - 2.0 * t)
}

/// Value noise in 0..1 with `px` x `py` lattice cells across the unit tile.
pub fn value_noise(u: f32, v: f32, px: i32, py: i32, seed: u32) -> f32 {
    let (px, py) = (px.max(1), py.max(1));
    let x = u.rem_euclid(1.0) * px as f32;
    let y = v.rem_euclid(1.0) * py as f32;
    let (x0, y0) = (x.floor(), y.floor());
    let (fx, fy) = (smooth(x - x0), smooth(y - y0));
    let (ix, iy) = (x0 as i32, y0 as i32);
    let at = |dx: i32, dy: i32| hash((ix + dx).rem_euclid(px), (iy + dy).rem_euclid(py), seed);
    let top = at(0, 0) + (at(1, 0) - at(0, 0)) * fx;
    let bottom = at(0, 1) + (at(1, 1) - at(0, 1)) * fx;
    top + (bottom - top) * fy
}

/// Fractal noise in 0..1: `octaves` layers, each with twice the lattice cells and half the weight.
pub fn fbm(u: f32, v: f32, cells: i32, octaves: u32, seed: u32) -> f32 {
    let (mut sum, mut weight, mut total) = (0.0, 0.5, 0.0);
    let mut period = cells.max(1);
    for octave in 0..octaves.max(1) {
        sum += weight * value_noise(u, v, period, period, seed.wrapping_add(octave * 7919));
        total += weight;
        weight *= 0.5;
        period = period.saturating_mul(2).min(1 << 14);
    }
    sum / total
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hash_is_deterministic_and_in_range() {
        for i in 0..200 {
            let h = hash(i, -i, 5);
            assert!((0.0..1.0).contains(&h));
            assert_eq!(h, hash(i, -i, 5));
        }
        assert_ne!(hash(1, 2, 0), hash(1, 2, 1));
    }

    #[test]
    fn noise_tiles_in_both_axes() {
        for step in 0..50 {
            let t = step as f32 * 0.0173;
            let a = value_noise(t, 0.3, 7, 5, 3);
            let b = value_noise(t + 1.0, 0.3, 7, 5, 3);
            assert!((a - b).abs() < 1e-4, "{a} {b}");
            assert!((fbm(0.2, t, 4, 4, 1) - fbm(0.2, t + 1.0, 4, 4, 1)).abs() < 1e-4);
            assert!((fbm(t, 0.0, 4, 4, 1) - fbm(t, 1.0, 4, 4, 1)).abs() < 1e-4);
        }
    }

    #[test]
    fn noise_is_continuous_and_bounded() {
        let mut last = value_noise(0.0, 0.5, 8, 8, 0);
        for step in 1..400 {
            let now = value_noise(step as f32 / 400.0, 0.5, 8, 8, 0);
            assert!((0.0..=1.0).contains(&now));
            assert!((now - last).abs() < 0.1, "jump at {step}");
            last = now;
        }
    }

    #[test]
    fn noise_varies() {
        let values: Vec<f32> = (0..64)
            .map(|i| fbm(i as f32 / 64.0, 0.4, 4, 3, 2))
            .collect();
        let min = values.iter().cloned().fold(f32::MAX, f32::min);
        let max = values.iter().cloned().fold(f32::MIN, f32::max);
        assert!(max - min > 0.1);
    }
}
