//! An ordered, switchable chain of filters.

use super::filter::{Filter, FilterKind};
use super::image_buffer::ImageBuffer;
use serde::{Deserialize, Serialize};

/// A filter plus its on/off switch.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FilterEntry {
    /// Disabled entries are skipped but keep their settings.
    pub enabled: bool,
    /// The filter.
    pub filter: Filter,
}

/// Filters applied from first to last; order matters (put tone mapping last).
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct FilterStack {
    /// The entries, first applied first.
    pub entries: Vec<FilterEntry>,
}

impl FilterStack {
    /// An empty stack, which leaves images untouched.
    pub fn new() -> Self {
        Self::default()
    }

    /// Number of entries, enabled or not.
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// True when there are no entries.
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Number of enabled entries.
    pub fn active_count(&self) -> usize {
        self.entries.iter().filter(|entry| entry.enabled).count()
    }

    /// Appends an enabled filter of `kind` with default parameters and returns its index.
    pub fn push(&mut self, kind: FilterKind) -> usize {
        self.entries.push(FilterEntry {
            enabled: true,
            filter: Filter::new(kind),
        });
        self.entries.len() - 1
    }

    /// Appends an enabled filter built by the caller and returns its index.
    pub fn push_filter(&mut self, filter: Filter) -> usize {
        self.entries.push(FilterEntry {
            enabled: true,
            filter: filter.normalized(),
        });
        self.entries.len() - 1
    }

    /// Removes the entry at `index`; returns false when it does not exist.
    pub fn remove(&mut self, index: usize) -> bool {
        let found = index < self.entries.len();
        if found {
            self.entries.remove(index);
        }
        found
    }

    /// Moves the entry at `from` so it ends up at `to`; returns false when either index is out of range.
    pub fn move_entry(&mut self, from: usize, to: usize) -> bool {
        let valid = from < self.entries.len() && to < self.entries.len();
        if valid {
            let entry = self.entries.remove(from);
            self.entries.insert(to, entry);
        }
        valid
    }

    /// Switches the entry at `index` on or off; returns false when it does not exist.
    pub fn set_enabled(&mut self, index: usize, enabled: bool) -> bool {
        self.entries
            .get_mut(index)
            .map(|entry| entry.enabled = enabled)
            .is_some()
    }

    /// Applies every enabled filter, in order, to `image` in place.
    pub fn apply_in_place(&self, image: &mut ImageBuffer) {
        for entry in self.entries.iter().filter(|entry| entry.enabled) {
            entry.filter.apply(image);
        }
    }

    /// A filtered copy of `image`; the original is untouched.
    pub fn apply(&self, image: &ImageBuffer) -> ImageBuffer {
        let mut out = image.clone();
        self.apply_in_place(&mut out);
        out
    }

    /// Serializes to JSON.
    pub fn to_json(&self) -> String {
        serde_json::to_string(self).unwrap_or_default()
    }

    /// Reads a stack from JSON, repairing out-of-range or missing parameters; `None` when the text is not a stack.
    pub fn from_json(text: &str) -> Option<Self> {
        let mut stack: Self = serde_json::from_str(text).ok()?;
        for entry in &mut stack.entries {
            entry.filter = std::mem::replace(&mut entry.filter, Filter::new(FilterKind::Exposure))
                .normalized();
        }
        Some(stack)
    }
}
