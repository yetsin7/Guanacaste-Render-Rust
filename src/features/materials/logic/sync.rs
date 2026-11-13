//! Keeping the editor's library and the open project's materials in step.

use super::project_io::{library_from_docs, remap_after_removal, store_library};
use crate::features::materials::models::{MaterialLibrary, MaterialState};
use crate::features::projects::models::Project;

/// Points the editor at `project` (or back at the session library when there is none) if it is not already
/// showing it. Returns whether the library was replaced.
pub fn adopt(state: &mut MaterialState, project: Option<&Project>) -> bool {
    let wanted = project.map(|p| p.id);
    if state.project_id == wanted {
        return false;
    }
    state.library = match project {
        Some(p) => library_from_docs(&p.document.materials),
        None => MaterialLibrary::with_presets(),
    };
    state.project_id = wanted;
    state.selected = 0;
    state.renaming = None;
    state.confirm_delete = false;
    state.failures.clear();
    state.changed = false;
    state.removed = None;
    true
}

/// Writes pending edits of the library into `project` (marking it modified) and clears the edit flags. A project
/// opened read-only is never modified: its edits stay in the session.
pub fn flush(state: &mut MaterialState, project: Option<&mut Project>) {
    if let Some(project) = project.filter(|p| !p.read_only) {
        if let Some(removed) = state.removed {
            remap_after_removal(project, removed);
        }
        if state.changed {
            store_library(project, &state.library);
        }
    }
    state.changed = false;
    state.removed = None;
}
