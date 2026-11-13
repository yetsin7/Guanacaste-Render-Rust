//! Reading and writing the materials of a `.gr` project document.

use super::convert;
use crate::features::materials::models::MaterialLibrary;
use crate::features::projects::models::{MaterialDoc, Project};

/// The library described by a document's materials: the full PBR material when present, otherwise a diffuse
/// material built from the stored color (files written before the material editor existed).
pub fn library_from_docs(docs: &[MaterialDoc]) -> MaterialLibrary {
    MaterialLibrary::from_materials(
        docs.iter()
            .map(|doc| match &doc.pbr {
                Some(pbr) => {
                    let mut material = pbr.clone();
                    material.name = doc.name.clone();
                    material
                }
                None => crate::features::materials::models::Material::from_color(
                    doc.name.clone(),
                    doc.color,
                ),
            })
            .collect(),
    )
}

/// The document entries of `library`: the portable name and color plus the full PBR material.
pub fn docs_from_library(library: &MaterialLibrary) -> Vec<MaterialDoc> {
    library
        .iter()
        .map(|material| {
            let summary = convert::to_scene(material);
            MaterialDoc {
                name: summary.name,
                color: summary.color,
                pbr: Some(material.clone()),
            }
        })
        .collect()
}

/// Stores `library` in `project` and marks it as modified when something changed. Returns whether it changed.
pub fn store_library(project: &mut Project, library: &MaterialLibrary) -> bool {
    let docs = docs_from_library(library);
    if docs == project.document.materials {
        return false;
    }
    project.document.materials = docs;
    project.dirty = true;
    true
}

/// Fixes the meshes of `project` after the material at `removed` was deleted from its list: meshes that used it
/// become material-less and later indices shift down by one.
pub fn remap_after_removal(project: &mut Project, removed: usize) {
    let shift = |index: usize| match index.cmp(&removed) {
        std::cmp::Ordering::Less => Some(index),
        std::cmp::Ordering::Equal => None,
        std::cmp::Ordering::Greater => Some(index - 1),
    };
    for mesh in &mut project.meshes {
        mesh.material = mesh.material.and_then(shift);
    }
    for meta in &mut project.document.meshes {
        meta.material = meta
            .material
            .and_then(|index| shift(index as usize).map(|i| i as u32));
    }
    project.dirty = true;
}
