//! Human-readable sizes and ages of project files.

/// `"812 KB"`, `"13,9 MB"`, `"1,2 GB"` (Spanish decimal comma).
pub fn size_label(bytes: u64) -> String {
    const KB: f64 = 1024.0;
    let value = bytes as f64;
    let text = if value < KB * KB {
        format!("{} KB", (value / KB).ceil() as u64)
    } else if value < KB * KB * KB {
        format!("{:.1} MB", value / (KB * KB))
    } else {
        format!("{:.1} GB", value / (KB * KB * KB))
    };
    text.replace('.', ",")
}

/// `"hace un momento"`, `"hace 5 min"`, `"hace 3 h"`, `"hace 2 días"`, from Unix seconds.
pub fn age_label(now: u64, then: u64) -> String {
    let seconds = now.saturating_sub(then);
    match seconds {
        0..=59 => "hace un momento".into(),
        60..=3_599 => format!("hace {} min", seconds / 60),
        3_600..=86_399 => format!("hace {} h", seconds / 3_600),
        86_400..=172_799 => "hace 1 día".into(),
        _ => format!("hace {} días", seconds / 86_400),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sizes_use_a_decimal_comma() {
        assert_eq!(size_label(1), "1 KB");
        assert_eq!(size_label(14_574_000), "13,9 MB");
        assert_eq!(size_label(3 << 30), "3,0 GB");
    }

    #[test]
    fn ages_are_relative() {
        assert_eq!(age_label(100, 90), "hace un momento");
        assert_eq!(age_label(1_000, 400), "hace 10 min");
        assert_eq!(age_label(10_000, 0), "hace 2 h");
        assert_eq!(age_label(100_000, 0), "hace 1 día");
        assert_eq!(age_label(1_000_000, 0), "hace 11 días");
        assert_eq!(age_label(0, 50), "hace un momento");
    }
}
