//! Random numbers, hemisphere sampling and the conversion from linear light to display pixels.

use super::vec3::Vec3;

/// A small PCG32 generator: deterministic, fast and dependency free.
#[derive(Debug, Clone)]
pub struct Rng {
    state: u64,
}

impl Rng {
    /// A generator for one pixel sample; equal inputs give equal sequences.
    pub fn for_sample(seed: u32, pixel: u32, sample: u32) -> Self {
        let mut rng = Self {
            state: (seed as u64) << 32 ^ (pixel as u64).wrapping_mul(0x9E37_79B9_7F4A_7C15),
        };
        rng.state = rng
            .state
            .wrapping_add((sample as u64).wrapping_mul(0xD1B5_4A32_D192_ED03));
        rng.next_u32();
        rng.next_u32();
        rng
    }

    /// The next 32 random bits.
    pub fn next_u32(&mut self) -> u32 {
        let old = self.state;
        self.state = old
            .wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(1_442_695_040_888_963_407);
        let xorshifted = (((old >> 18) ^ old) >> 27) as u32;
        xorshifted.rotate_right((old >> 59) as u32)
    }

    /// A uniform number in `[0, 1)`.
    pub fn next_f32(&mut self) -> f32 {
        (self.next_u32() >> 8) as f32 / 16_777_216.0
    }
}

/// Two perpendicular unit vectors that complete the unit vector `n` to a basis.
pub fn basis(n: Vec3) -> (Vec3, Vec3) {
    let helper = if n.x.abs() > 0.9 {
        Vec3::new(0.0, 1.0, 0.0)
    } else {
        Vec3::new(1.0, 0.0, 0.0)
    };
    let t = n.cross(helper).normalized();
    (t, n.cross(t))
}

/// A direction in the hemisphere around `n`, distributed proportionally to the cosine of the angle with `n`.
pub fn cosine_hemisphere(n: Vec3, rng: &mut Rng) -> Vec3 {
    let (u1, u2) = (rng.next_f32(), rng.next_f32());
    let radius = u1.sqrt();
    let phi = std::f32::consts::TAU * u2;
    let (t, b) = basis(n);
    (t * (radius * phi.cos()) + b * (radius * phi.sin()) + n * (1.0 - u1).max(0.0).sqrt())
        .normalized()
}

/// ACES filmic curve followed by the sRGB transfer function, per channel, 0..=1.
pub fn tone_map(linear: Vec3) -> [f32; 3] {
    let map = |x: f32| {
        let x = x.max(0.0);
        let aces = (x * (2.51 * x + 0.03)) / (x * (2.43 * x + 0.59) + 0.14);
        let v = aces.clamp(0.0, 1.0);
        if v <= 0.003_130_8 {
            v * 12.92
        } else {
            1.055 * v.powf(1.0 / 2.4) - 0.055
        }
    };
    [map(linear.x), map(linear.y), map(linear.z)]
}

/// Quantizes a 0..=1 value to a byte.
pub fn to_byte(value: f32) -> u8 {
    (value.clamp(0.0, 1.0) * 255.0 + 0.5) as u8
}

/// The ACES filmic curve on one linear channel; the result is display-linear, 0..=1.
pub fn aces(x: f32) -> f32 {
    let x = x.max(0.0);
    ((x * (2.51 * x + 0.03)) / (x * (2.43 * x + 0.59) + 0.14)).clamp(0.0, 1.0)
}
