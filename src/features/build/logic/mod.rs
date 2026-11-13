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
