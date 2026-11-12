//! Inspector rules: wind scale names.

/// Beaufort-style label for a normalized wind value: calm, breeze, gale or storm.
pub fn beaufort_label(wind: f32) -> &'static str {
    match wind {
        w if w < 0.2 => "inspector.wind_calm",
        w if w < 0.5 => "inspector.wind_breeze",
        w if w < 0.8 => "inspector.wind_gale",
        _ => "inspector.wind_storm",
    }
}

/// Beaufort number (0..=12) for a normalized wind value.
pub fn beaufort_number(wind: f32) -> u8 {
    (wind.clamp(0.0, 1.0) * 12.0).round() as u8
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wind_labels_follow_the_scale() {
        assert_eq!(beaufort_label(0.0), "inspector.wind_calm");
        assert_eq!(beaufort_label(0.65), "inspector.wind_gale");
        assert_eq!(beaufort_label(1.0), "inspector.wind_storm");
    }

    #[test]
    fn beaufort_numbers_scale_to_twelve() {
        assert_eq!(beaufort_number(1.0), 12);
        assert_eq!(beaufort_number(0.65), 8);
    }
}
