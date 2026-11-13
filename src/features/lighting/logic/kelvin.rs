//! Correlated color temperature to linear sRGB.
//!
//! The chromaticity of the Planckian locus comes from the cubic spline of Kim et al. (US patent 7,024,034, accurate to
//! about 1e-3 in CIE 1931 xy between 1667 K and 25000 K). It is converted to XYZ with Y = 1, then to linear sRGB with
//! the IEC 61966-2-1 matrix (D65 white), negative channels are clipped and the result is normalized so its largest
//! channel is 1.

use crate::features::lighting::models::Rgb;

/// Lowest temperature of the spline; colder requests are clamped.
pub const MIN_KELVIN: f32 = 1667.0;
/// Highest temperature of the spline; hotter requests are clamped.
pub const MAX_KELVIN: f32 = 25000.0;

/// CIE 1931 `(x, y)` chromaticity of a black body at `kelvin` (clamped to `MIN_KELVIN..=MAX_KELVIN`).
pub fn planckian_xy(kelvin: f32) -> (f64, f64) {
    let t = f64::from(kelvin.clamp(MIN_KELVIN, MAX_KELVIN));
    let (t2, t3) = (t * t, t * t * t);
    let x = if t <= 4000.0 {
        -0.266_123_9e9 / t3 - 0.234_358_9e6 / t2 + 0.877_695_6e3 / t + 0.179_910
    } else {
        -3.025_846_9e9 / t3 + 2.107_037_9e6 / t2 + 0.222_634_7e3 / t + 0.240_390
    };
    let (x2, x3) = (x * x, x * x * x);
    let y = if t <= 2222.0 {
        -1.106_381_4 * x3 - 1.348_110_20 * x2 + 2.185_558_32 * x - 0.202_196_83
    } else if t <= 4000.0 {
        -0.954_947_6 * x3 - 1.374_185_93 * x2 + 2.091_370_15 * x - 0.167_488_67
    } else {
        3.081_758_0 * x3 - 5.873_386_70 * x2 + 3.751_129_97 * x - 0.370_014_83
    };
    (x, y)
}

/// Linear sRGB of chromaticity `(x, y)` at luminance 1, unclipped.
pub fn xy_to_linear_srgb(x: f64, y: f64) -> [f64; 3] {
    let big_x = x / y;
    let big_z = (1.0 - x - y) / y;
    [
        3.240_454_2 * big_x - 1.537_138_5 - 0.498_531_4 * big_z,
        -0.969_266_0 * big_x + 1.876_010_8 + 0.041_556_0 * big_z,
        0.055_643_4 * big_x - 0.204_025_9 + 1.057_225_2 * big_z,
    ]
}

/// Linear sRGB color of a black body at `kelvin`, normalized so its largest channel is 1.
pub fn kelvin_to_rgb(kelvin: f32) -> Rgb {
    let (x, y) = planckian_xy(kelvin);
    let [r, g, b] = xy_to_linear_srgb(x, y).map(|c| c.max(0.0));
    let max = r.max(g).max(b);
    if max <= 0.0 {
        return Rgb::WHITE;
    }
    Rgb::new((r / max) as f32, (g / max) as f32, (b / max) as f32)
}
