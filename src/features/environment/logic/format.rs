//! Text formatting of environment values.

/// Formats a clock time in hours (0..24) as `HH:MM`.
pub fn format_clock(hour: f32) -> String {
    let minutes = ((hour.rem_euclid(24.0) * 60.0).round() as u32) % (24 * 60);
    format!("{:02}:{:02}", minutes / 60, minutes % 60)
}
