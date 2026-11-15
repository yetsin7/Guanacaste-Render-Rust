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

/// The AgX filmic curve followed by the sRGB transfer function, 0..=1 (the live preview's finish, so it matches
/// the final picture).
pub fn tone_map(linear: Vec3) -> [f32; 3] {
    let encode = |v: f32| {
        if v <= 0.003_130_8 {
            v * 12.92
        } else {
            1.055 * v.powf(1.0 / 2.4) - 0.055
        }
    };
    agx([linear.x, linear.y, linear.z]).map(encode)
}

/// The AgX view transform (Troy Sobotka's design with Wrensch's polynomial fit), as the original Guanacaste 3D
/// renderer used it: light is first mixed a little toward the other primaries so over-bright colors bleach to white
/// instead of clipping one channel, the log-encoded result is shaped by a gentle filmic curve and mixed back. Shadows
/// stay open and saturated colors stay calm, which is what makes a picture read as a photograph rather than a
/// graphic. The "punchy" finish (30 % more color, a 1.25 power) gives back the depth the plain curve lifts, and a
/// gain of 2 puts mid grey where the ACES curve put it, so exposure keeps its calibration. Returns display-linear
/// light, 0..=1, ready for the sRGB encoding.
pub fn agx(color: [f32; 3]) -> [f32; 3] {
    const MIN_EV: f32 = -12.473_93;
    const MAX_EV: f32 = 4.026_069;
    const PUNCH: f32 = 2.2 * 1.25;
    let c = color.map(|v| (v * 2.0).max(0.0));
    let mixed = [
        0.842_479_06 * c[0] + 0.078_433_6 * c[1] + 0.079_223_745 * c[2],
        0.042_328_242 * c[0] + 0.878_468_64 * c[1] + 0.079_166_13 * c[2],
        0.042_375_655 * c[0] + 0.078_433_6 * c[1] + 0.879_142_97 * c[2],
    ];
    let curve = |value: f32| {
        let log = value.max(1e-10).log2().clamp(MIN_EV, MAX_EV);
        let x = (log - MIN_EV) / (MAX_EV - MIN_EV);
        let x2 = x * x;
        let x4 = x2 * x2;
        15.5 * x4 * x2 - 40.14 * x4 * x + 31.96 * x4 - 6.868 * x2 * x + 0.4298 * x2 + 0.1191 * x
            - 0.00232
    };
    let shaped = mixed.map(curve);
    let grey = 0.2126 * shaped[0] + 0.7152 * shaped[1] + 0.0722 * shaped[2];
    let shaped = shaped.map(|v| (grey + (v - grey) * 1.3).max(0.0));
    let display = [
        1.196_879 * shaped[0] - 0.098_020_88 * shaped[1] - 0.099_029_74 * shaped[2],
        -0.052_896_85 * shaped[0] + 1.151_903_1 * shaped[1] - 0.098_961_18 * shaped[2],
        -0.052_971_635 * shaped[0] - 0.098_043_45 * shaped[1] + 1.151_073_7 * shaped[2],
    ];
    grade(display.map(|v| v.clamp(0.0, 1.0).powf(PUNCH)))
}

/// Highlight and shadow recovery of the original Guanacaste 3D "natural" look (+22 % shadows, −22 % highlights),
/// on display values and weighted by luminance as a photographer expects: shadows open up under trees and
/// carports, bright walls keep their shading instead of burning to white.
fn grade(color: [f32; 3]) -> [f32; 3] {
    const SHADOWS: f32 = 0.22;
    const HIGHLIGHTS: f32 = -0.22;
    let luminance = (0.2126 * color[0] + 0.7152 * color[1] + 0.0722 * color[2]).clamp(0.0, 1.0);
    let dark = 1.0 - luminance;
    let gain = (1.0 + HIGHLIGHTS * 0.45 * luminance * luminance)
        * (1.0 + SHADOWS * 0.9 * dark * dark * dark);
    color.map(|v| (v * gain).clamp(0.0, 1.0))
}

/// Quantizes a 0..=1 value to a byte.
pub fn to_byte(value: f32) -> u8 {
    (value.clamp(0.0, 1.0) * 255.0 + 0.5) as u8
}
