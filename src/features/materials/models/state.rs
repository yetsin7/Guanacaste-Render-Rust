//! Material editor state.

use super::{MapKind, Material, MaterialLibrary};
use crate::features::materials::logic::preview::PreviewShape;
use crate::services::texture_cache::LoadError;

/// UI state of the material editor.
#[derive(Debug, Clone, PartialEq)]
pub struct MaterialState {
    /// The materials being edited: the open project's, or a session library with the presets when no project is open.
    pub library: MaterialLibrary,
    /// Index of the selected material.
    pub selected: usize,
    /// Name filter of the list.
    pub filter: String,
    /// Preview object.
    pub shape: PreviewShape,
    /// Preview rotation about the vertical axis, radians.
    pub yaw: f32,
    /// Preview rotation about the horizontal axis, radians.
    pub pitch: f32,
    /// The name being typed while renaming the selected material.
    pub renaming: Option<String>,
    /// Whether the delete button was pressed once and awaits confirmation.
    pub confirm_delete: bool,
    /// Textures of the selected material that failed to load, with the reason.
    pub failures: Vec<(MapKind, LoadError)>,
    /// Identifier of the project the library mirrors; `None` for the session library.
    pub project_id: Option<[u8; 16]>,
    /// Set by any edit; the screen stores the library into the project and clears it.
    pub changed: bool,
    /// Index of a material deleted this frame, so the screen can fix the meshes that used it.
    pub removed: Option<usize>,
}

impl Default for MaterialState {
    fn default() -> Self {
        Self {
            library: MaterialLibrary::with_presets(),
            selected: 0,
            filter: String::new(),
            shape: PreviewShape::Sphere,
            yaw: 0.5,
            pitch: 0.25,
            renaming: None,
            confirm_delete: false,
            failures: Vec::new(),
            project_id: None,
            changed: false,
            removed: None,
        }
    }
}

impl MaterialState {
    /// The selected material.
    pub fn current(&self) -> Option<&Material> {
        self.library.get(self.selected)
    }

    /// Mutable access to the selected material; the caller sets `changed` when it edits.
    pub fn current_mut(&mut self) -> Option<&mut Material> {
        self.library.get_mut(self.selected)
    }
}
