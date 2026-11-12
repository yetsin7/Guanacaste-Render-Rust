//! Pure grid math shared by card layouts.

/// Number of columns that fit in `width` when each column needs at least `min_column` points.
pub fn column_count(width: f32, min_column: f32, gap: f32, max_columns: usize) -> usize {
    let fit = ((width + gap) / (min_column + gap)).floor() as usize;
    fit.clamp(1, max_columns.max(1))
}

/// Width of one column when `columns` share `width` separated by `gap`.
pub fn column_width(width: f32, columns: usize, gap: f32) -> f32 {
    ((width - gap * (columns as f32 - 1.0)) / columns as f32).floor()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn counts_columns_that_fit() {
        assert_eq!(column_count(1200.0, 280.0, 12.0, 4), 4);
        assert_eq!(column_count(700.0, 280.0, 12.0, 4), 2);
        assert_eq!(column_count(100.0, 280.0, 12.0, 4), 1);
    }

    #[test]
    fn column_widths_fill_the_row() {
        let width = column_width(1000.0, 4, 12.0);
        assert!(width * 4.0 + 36.0 <= 1000.0);
        assert!(width * 4.0 + 36.0 > 995.0);
    }
}
