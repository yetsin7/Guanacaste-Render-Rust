//! Height map to tangent-space normal map conversion.

use super::bitmap::Bitmap;

/// Converts the grayscale (luminance) of `height` to a normal map with +Y up, using central differences that wrap
/// around the borders so tiled textures stay seamless. `strength` scales the slope.
pub fn normal_from_height(height: &Bitmap, strength: f32) -> Bitmap {
    let (w, h) = (height.width, height.height);
    let value = |x: isize, y: isize| -> f32 {
        let p = height.at(
            x.rem_euclid(w as isize) as usize,
            y.rem_euclid(h as isize) as usize,
        );
        (0.2126 * p[0] as f32 + 0.7152 * p[1] as f32 + 0.0722 * p[2] as f32) / 255.0
    };
    let mut pixels = Vec::with_capacity(w * h);
    for y in 0..h as isize {
        for x in 0..w as isize {
            let dx = (value(x + 1, y) - value(x - 1, y)) * 0.5 * strength;
            // Image rows grow downward while +Y of the normal points up.
            let dy = (value(x, y - 1) - value(x, y + 1)) * 0.5 * strength;
            let length = (dx * dx + dy * dy + 1.0).sqrt();
            let encode = |n: f32| ((n / length * 0.5 + 0.5) * 255.0 + 0.5) as u8;
            pixels.push([encode(-dx), encode(-dy), encode(1.0), 255]);
        }
    }
    Bitmap {
        width: w,
        height: h,
        pixels,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_flat_height_gives_a_straight_up_normal() {
        let n = normal_from_height(&Bitmap::filled(4, 4, [90, 90, 90, 255]), 4.0);
        assert!(n
            .pixels
            .iter()
            .all(|p| p[0] == 128 && p[1] == 128 && p[2] == 255));
    }

    #[test]
    fn a_ramp_tilts_the_normal_against_the_slope() {
        let mut ramp = Bitmap::filled(8, 1, [0; 4]);
        for (x, pixel) in ramp.pixels.iter_mut().enumerate() {
            *pixel = [(x * 30) as u8, (x * 30) as u8, (x * 30) as u8, 255];
        }
        let n = normal_from_height(&ramp, 2.0);
        // Rising to the right: the normal leans left (red below 128).
        assert!(n.at(3, 0)[0] < 120);
    }
}
