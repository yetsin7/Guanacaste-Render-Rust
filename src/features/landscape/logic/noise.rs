//! Deterministic randomness: a seeded generator, 2D value noise and fractal Brownian motion (fBm).

/// A small seeded pseudo-random generator (SplitMix64). The same seed always yields the same sequence.
#[derive(Debug, Clone)]
pub struct Rng(u64);

impl Rng {
    /// A generator started from `seed`.
    pub fn new(seed: u64) -> Self {
        Self(seed)
    }

    /// Next 64 random bits.
    pub fn next_u64(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9E37_79B9_7F4A_7C15);
        mix(self.0)
    }

    /// Uniform value in `0.0..1.0`.
    pub fn next_f32(&mut self) -> f32 {
        (self.next_u64() >> 40) as f32 / (1u64 << 24) as f32
    }

    /// Uniform value in `low..high` (returns `low` when the range is empty).
    pub fn range(&mut self, low: f32, high: f32) -> f32 {
        if high <= low {
            return low;
        }
        low + (high - low) * self.next_f32()
    }
}

fn mix(mut z: u64) -> u64 {
    z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    z ^ (z >> 31)
}

/// Combines a seed with another value into a new seed.
pub fn derive_seed(seed: u64, salt: u64) -> u64 {
    mix(seed ^ mix(salt.wrapping_add(0x632B_E59B_D9B4_E019)))
}

/// Random value in `-1.0..=1.0` attached to the lattice point `(x, y)`.
fn lattice(x: i64, y: i64, seed: u64) -> f32 {
    let hash = mix(seed ^ mix((x as u64).wrapping_mul(0x8CB9_2BA7_2F3D_8DD7) ^ (y as u64)));
    (hash >> 40) as f32 / (1u64 << 23) as f32 - 1.0
}

fn fade(t: f32) -> f32 {
    t * t * t * (t * (t * 6.0 - 15.0) + 10.0)
}

/// Smooth value noise in `-1.0..=1.0` with features about one unit wide.
pub fn value_noise(x: f32, y: f32, seed: u64) -> f32 {
    let (fx, fy) = (x.floor(), y.floor());
    let (ix, iy) = (fx as i64, fy as i64);
    let (u, v) = (fade(x - fx), fade(y - fy));
    let a = lattice(ix, iy, seed);
    let b = lattice(ix + 1, iy, seed);
    let c = lattice(ix, iy + 1, seed);
    let d = lattice(ix + 1, iy + 1, seed);
    let bottom = a + (b - a) * u;
    let top = c + (d - c) * u;
    bottom + (top - bottom) * v
}

/// Fractal Brownian motion in `-1.0..=1.0`: `octaves` layers of value noise, each `lacunarity` times finer and
/// `gain` times weaker than the previous one.
pub fn fbm(x: f32, y: f32, seed: u64, octaves: u32, lacunarity: f32, gain: f32) -> f32 {
    let mut total = 0.0;
    let mut norm = 0.0;
    let mut amplitude = 1.0;
    let mut frequency = 1.0;
    for octave in 0..octaves.clamp(1, 10) {
        let octave_seed = derive_seed(seed, u64::from(octave));
        total += amplitude * value_noise(x * frequency, y * frequency, octave_seed);
        norm += amplitude;
        amplitude *= gain;
        frequency *= lacunarity;
    }
    if norm > 0.0 {
        (total / norm).clamp(-1.0, 1.0)
    } else {
        0.0
    }
}
