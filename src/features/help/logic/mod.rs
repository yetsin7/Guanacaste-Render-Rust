//! Help rules: the real controls and the shortcut search.

pub mod controls;

use super::models::Shortcut;

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
