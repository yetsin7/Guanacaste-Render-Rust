//! Help entities and UI state.

/// How a span of tutorial text is emphasized.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Emphasis {
    /// Normal text.
    Plain,
    /// A key or control name (primary color).
    Key,
    /// A strong term (bright text).
    Strong,
}

/// One step of the tutorial.
#[derive(Debug, Clone, PartialEq)]
pub struct TutorialStep {
    /// Short topic shown next to the step counter.
    pub topic: &'static str,
    /// Step title.
    pub title: &'static str,
    /// Body text as emphasized spans.
    pub body: Vec<(&'static str, Emphasis)>,
    /// Caption over the illustration.
    pub caption: &'static str,
    /// Four `(title, note)` tiles under the illustration.
    pub tiles: [(&'static str, &'static str); 4],
}

/// A shortcut row: action and key combination.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Shortcut {
    /// What the shortcut does.
    pub action: &'static str,
    /// Key combination.
    pub keys: &'static str,
}

/// UI state of the help screen.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct HelpState {
    /// Current tutorial step, 0-based.
    pub step: usize,
    /// Search text.
    pub search: String,
    /// Key highlighted in the flight cluster.
    pub highlighted: Option<char>,
}
