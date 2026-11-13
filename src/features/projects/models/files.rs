//! State of the open, save and new-scene dialogs. Each dialog only records what the user asked for
//! (`request`, `create`); the shell's workspace carries it out.

use crate::features::import_export::services::picker::FilePicker;
use eframe::egui::TextureHandle;
use std::collections::HashMap;
use std::path::PathBuf;

/// State of the open dialog.
#[derive(Default)]
pub struct OpenState {
    /// Selected recent file.
    pub selected: Option<usize>,
    /// Typed path.
    pub path: String,
    /// File the user asked to open.
    pub request: Option<PathBuf>,
    /// Native file dialog.
    pub picker: FilePicker,
    /// Thumbnails of recent files (`None` when the file has none or cannot be read).
    pub thumbnails: HashMap<PathBuf, Option<TextureHandle>>,
}

/// State of the save dialog.
pub struct SaveState {
    /// File name, without folder.
    pub name: String,
    /// Target folder.
    pub folder: String,
    /// Thumbnail switch.
    pub thumbnail: bool,
    /// Localization key of the reason why "Save" became "Save as", if any.
    pub reason_key: Option<&'static str>,
    /// Path the user asked to save to.
    pub request: Option<PathBuf>,
    /// Native file dialog.
    pub picker: FilePicker,
}

impl Default for SaveState {
    fn default() -> Self {
        Self {
            name: String::new(),
            folder: String::new(),
            thumbnail: true,
            reason_key: None,
            request: None,
            picker: FilePicker::default(),
        }
    }
}

impl SaveState {
    /// Fills name and folder from a proposed path.
    pub fn propose(&mut self, suggested: &std::path::Path, reason_key: Option<&'static str>) {
        self.name = suggested
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or_default()
            .to_owned();
        if let Some(folder) = suggested.parent().filter(|f| !f.as_os_str().is_empty()) {
            self.folder = folder.display().to_string();
        }
        self.reason_key = reason_key;
        self.request = None;
    }

    /// The path made of the folder and the name.
    pub fn target(&self) -> PathBuf {
        PathBuf::from(self.folder.trim()).join(self.name.trim())
    }
}

/// State of the new-scene dialog.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct NewSceneState {
    /// Scene name.
    pub name: String,
    /// Selected template.
    pub template: usize,
    /// Folder proposed when the scene is first saved.
    pub folder: String,
    /// The user pressed "Create".
    pub create: bool,
}
