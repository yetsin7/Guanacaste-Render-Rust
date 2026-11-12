//! Number formatting helpers.

/// Formats `value` with thousands separators and the given decimals, e.g. `1250.0` -> `"1,250"`.
pub fn with_thousands(value: f32, decimals: usize) -> String {
    let text = format!("{:.*}", decimals, value.abs());
    let (whole, fraction) = text
        .split_once('.')
        .map_or((text.as_str(), None), |(w, f)| (w, Some(f)));
    let mut grouped = String::new();
    for (index, digit) in whole.chars().enumerate() {
        if index > 0 && (whole.len() - index) % 3 == 0 {
            grouped.push(',');
        }
        grouped.push(digit);
    }
    if let Some(fraction) = fraction {
        grouped.push('.');
        grouped.push_str(fraction);
    }
    if value < 0.0 {
        grouped.insert(0, '-');
    }
    grouped
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn thousands_separators_group_digits() {
        assert_eq!(with_thousands(1250.0, 0), "1,250");
        assert_eq!(with_thousands(999.0, 0), "999");
        assert_eq!(with_thousands(1234567.0, 0), "1,234,567");
        assert_eq!(with_thousands(-18.8, 2), "-18.80");
    }
}
