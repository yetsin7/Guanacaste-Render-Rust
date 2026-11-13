//! Storing the landscape in a `.gr` project document. The `landscape` field is optional and additive: projects
//! written before it existed open with no landscape, and older readers ignore the field.

use crate::features::landscape::models::Landscape;
use crate::features::projects::models::ProjectDocument;

/// Writes `landscape` into `document`.
pub fn save_into(document: &mut ProjectDocument, landscape: &Landscape) {
    document.landscape = Some(landscape.clone());
}

/// The landscape of `document`, repaired so it is safe to use; `None` when the project has none.
pub fn load_from(document: &ProjectDocument) -> Option<Landscape> {
    let mut landscape = document.landscape.clone()?;
    landscape.repair();
    Some(landscape)
}
