//! Help entities and UI state.

/// A shortcut row: action and key combination (both already localized).
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
    /// Search text.
    pub search: String,
}

/// Facts about the running session shown on the help screen.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Diagnostics {
    /// The 3D view is drawn by the graphics card.
    pub gpu: bool,
    /// Threads the renderer can use.
    pub threads: usize,
    /// Name of the open project.
    pub project: Option<String>,
    /// Meshes of the open project.
    pub meshes: usize,
    /// Triangles of the open project.
    pub triangles: usize,
    /// Placed objects.
    pub objects: usize,
    /// Scattered plants.
    pub plants: usize,
    /// Saved cameras.
    pub cameras: usize,
}
