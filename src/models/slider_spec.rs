//! How a slider's normalized value maps to a displayed measure.

use crate::utils::format::with_thousands;

/// Range, unit and precision of a slider.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SliderSpec {
    /// Value at the left end.
    pub min: f32,
    /// Value at the right end.
    pub max: f32,
    /// Unit suffix, e.g. `" m"`.
    pub unit: &'static str,
    /// Digits after the decimal point.
    pub decimals: usize,
}

impl SliderSpec {
    /// Real value at normalized position `t`.
    pub fn value_of(&self, t: f32) -> f32 {
        self.min + (self.max - self.min) * t.clamp(0.0, 1.0)
    }

    /// Displayed text at normalized position `t`, e.g. `"1,250 cd"`.
    pub fn format(&self, t: f32) -> String {
        format!(
            "{}{}",
            with_thousands(self.value_of(t), self.decimals),
            self.unit
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn values_follow_the_range() {
        let spec = SliderSpec {
            min: 0.0,
            max: 1923.0,
            unit: " cd",
            decimals: 0,
        };
        assert_eq!(spec.value_of(0.0), 0.0);
        assert_eq!(spec.value_of(1.5), 1923.0);
        assert_eq!(spec.format(0.65), "1,250 cd");
    }
}
