//! Color space helpers shared by sampling, presets and the preview.

/// sRGB-encoded value (0..1) to linear light.
pub fn srgb_to_linear(value: f32) -> f32 {
    let c = value.clamp(0.0, 1.0);
    if c <= 0.04045 {
        c / 12.92
    } else {
        ((c + 0.055) / 1.055).powf(2.4)
    }
}

/// Linear light to an sRGB-encoded value (0..1).
pub fn linear_to_srgb(value: f32) -> f32 {
    let c = value.clamp(0.0, 1.0);
    if c <= 0.003_130_8 {
        c * 12.92
    } else {
        1.055 * c.powf(1.0 / 2.4) - 0.055
    }
}

/// Lookup table of [`srgb_to_linear`] for every 8-bit value.
pub fn srgb_table() -> &'static [f32; 256] {
    static TABLE: std::sync::OnceLock<[f32; 256]> = std::sync::OnceLock::new();
    TABLE.get_or_init(|| std::array::from_fn(|i| srgb_to_linear(i as f32 / 255.0)))
}

/// Rec. 709 luminance of a linear or gamma RGB triple.
pub fn luminance(rgb: [f32; 3]) -> f32 {
    0.2126 * rgb[0] + 0.7152 * rgb[1] + 0.0722 * rgb[2]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn conversions_are_inverse_and_monotonic() {
        let mut previous = -1.0;
        for step in 0..=100 {
            let v = step as f32 / 100.0;
            let linear = srgb_to_linear(v);
            assert!(linear >= previous);
            previous = linear;
            assert!((linear_to_srgb(linear) - v).abs() < 1e-4);
        }
        assert_eq!(srgb_to_linear(0.0), 0.0);
        assert!((srgb_to_linear(1.0) - 1.0).abs() < 1e-6);
        assert!((srgb_to_linear(0.5) - 0.2140).abs() < 1e-3);
    }

    #[test]
    fn the_table_matches_the_function() {
        assert_eq!(srgb_table()[128], srgb_to_linear(128.0 / 255.0));
        assert_eq!(srgb_table()[255], 1.0);
    }

    #[test]
    fn luminance_weights_green_most() {
        assert!(luminance([0.0, 1.0, 0.0]) > luminance([1.0, 0.0, 0.0]));
        assert!((luminance([1.0; 3]) - 1.0).abs() < 1e-6);
    }
}
