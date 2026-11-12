//! Build-mode rules: guide progress.

use super::models::{BuildGuide, BuildState};

/// Registers one more placed instance; at the end of the batch the guide moves to the next step.
pub fn place_next(state: &mut BuildState, guide: &BuildGuide) {
    if state.placed + 1 >= guide.instances_total {
        state.placed = 0;
        state.step = if state.step >= guide.step_total {
            1
        } else {
            state.step + 1
        };
    } else {
        state.placed += 1;
    }
}

/// Formats a normalized time of day (06:00 at 0, 20:00 at 1) as `HH:MM`.
pub fn format_hour(t: f32) -> String {
    let minutes = (6.0 * 60.0 + t.clamp(0.0, 1.0) * 14.0 * 60.0).round() as u32;
    format!("{:02}:{:02}", minutes / 60, minutes % 60)
}
