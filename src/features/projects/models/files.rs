//! Project files offered by the open, save and new-scene dialogs.

/// A recent project file.
#[derive(Debug, Clone, PartialEq)]
pub struct RecentFile {
    /// File name.
    pub name: &'static str,
    /// Folder.
    pub folder: &'static str,
    /// Size on disk.
    pub size: &'static str,
    /// Relative modification time.
    pub modified: &'static str,
}

/// State of the open dialog.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct OpenState {
    /// Selected recent file.
    pub selected: Option<usize>,
    /// Typed path.
    pub path: String,
}

/// State of the save dialog.
#[derive(Debug, Clone, PartialEq)]
pub struct SaveState {
    /// File name.
    pub name: String,
    /// Target folder.
    pub folder: String,
    /// Compress switch.
    pub compress: bool,
    /// Thumbnail switch.
    pub thumbnail: bool,
}

impl Default for SaveState {
    fn default() -> Self {
        Self {
            name: "Villa Moderna L26".into(),
            folder: "C:/Proyectos/Guanacaste".into(),
            compress: true,
            thumbnail: true,
        }
    }
}

/// State of the new-scene dialog.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct NewSceneState {
    /// Scene name.
    pub name: String,
    /// Selected template.
    pub template: usize,
    /// Target folder.
    pub folder: String,
}
