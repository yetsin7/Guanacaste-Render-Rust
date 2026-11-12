//! Help rules: tutorial navigation and shortcut search.

use super::models::{HelpState, Shortcut};

/// Moves to the next tutorial step, if any.
pub fn next_step(state: &mut HelpState, total: usize) {
    if state.step + 1 < total {
        state.step += 1;
    }
}

/// Moves to the previous tutorial step, if any.
pub fn previous_step(state: &mut HelpState) {
    state.step = state.step.saturating_sub(1);
}

/// The shortcuts whose action or keys contain the search text (case-insensitive).
pub fn filter_shortcuts<'a>(shortcuts: &'a [Shortcut], search: &str) -> Vec<&'a Shortcut> {
    let needle = search.trim().to_lowercase();
    shortcuts
        .iter()
        .filter(|shortcut| {
            needle.is_empty()
                || shortcut.action.to_lowercase().contains(&needle)
                || shortcut.keys.to_lowercase().contains(&needle)
        })
        .collect()
}
