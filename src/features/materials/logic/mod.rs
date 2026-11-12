//! Material editor rules.

use super::models::MaterialState;

/// Restores the default parameters, keeping the viewport tool and filter.
pub fn reset(state: &mut MaterialState) {
    *state = MaterialState {
        tool: state.tool,
        filter: std::mem::take(&mut state.filter),
        ..MaterialState::default()
    };
}

/// Whether a parameter named `name` is shown under the filter text (case-insensitive substring).
pub fn matches_filter(name: &str, filter: &str) -> bool {
    let needle = filter.trim().to_lowercase();
    needle.is_empty() || name.to_lowercase().contains(&needle)
}
