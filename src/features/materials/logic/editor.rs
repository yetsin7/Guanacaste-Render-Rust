//! The editor's actions on its state: selecting, creating, duplicating, renaming, deleting and assigning textures.

use super::matches_filter;
use super::presets;
use crate::features::materials::models::{
    MapKind, Material, MaterialState, Procedural, TextureRef,
};
use crate::services::texture_cache::TextureCache;
use std::path::Path;

/// Indices of the materials shown under the list filter.
pub fn visible(state: &MaterialState) -> Vec<usize> {
    state
        .library
        .iter()
        .enumerate()
        .filter(|(_, material)| matches_filter(&material.name, &state.filter))
        .map(|(index, _)| index)
        .collect()
}

fn edited(state: &mut MaterialState) {
    state.changed = true;
    state.confirm_delete = false;
}

/// Selects the material at `index` (ignored when out of range); this cancels a rename or pending delete.
pub fn select(state: &mut MaterialState, index: usize) {
    if index < state.library.len() && index != state.selected {
        state.selected = index;
        state.renaming = None;
        state.confirm_delete = false;
        state.failures.clear();
    }
}

fn append(state: &mut MaterialState, material: Material) {
    let index = state.library.push(material);
    state.selected = index;
    state.renaming = None;
    state.failures.clear();
    edited(state);
}

/// Adds a new default material and selects it.
pub fn add_new(state: &mut MaterialState, name: &str) {
    append(state, Material::new(name));
}

/// Adds a copy of preset number `index` (see [`presets::all`]) and selects it.
pub fn add_preset(state: &mut MaterialState, index: usize) {
    if let Some(material) = presets::available().into_iter().nth(index) {
        append(state, material);
    }
}

/// Duplicates the selected material (the copy goes to the end of the list) and selects the copy.
pub fn duplicate_selected(state: &mut MaterialState) {
    if let Some(copy) = state.library.duplicate(state.selected) {
        state.selected = copy;
        state.renaming = None;
        edited(state);
    }
}

/// Starts renaming the selected material.
pub fn begin_rename(state: &mut MaterialState) {
    state.renaming = state.current().map(|m| m.name.clone());
}

/// Applies the typed name; a blank name keeps the old one.
pub fn commit_rename(state: &mut MaterialState) {
    if let Some(name) = state.renaming.take() {
        if state.library.rename(state.selected, &name) {
            edited(state);
        }
    }
}

/// Deletes the selected material and records its index in `state.removed`. The selection moves to the previous
/// material.
pub fn delete_selected(state: &mut MaterialState) {
    let index = state.selected;
    if state.library.remove(index).is_some() {
        state.removed = Some(index);
        state.selected = index
            .saturating_sub(1)
            .min(state.library.len().saturating_sub(1));
        state.renaming = None;
        state.failures.clear();
        edited(state);
    }
}

/// Assigns the image file at `path` to slot `kind` of the selected material, forgetting any cached copy so an
/// edited file is read again. An emission map on a black emission color switches the color to white, otherwise the
/// map would stay invisible.
pub fn assign_file(
    state: &mut MaterialState,
    cache: &mut TextureCache,
    kind: MapKind,
    path: &Path,
) {
    cache.invalidate(path);
    assign(
        state,
        kind,
        TextureRef::file(path.to_string_lossy().into_owned()),
    );
}

/// Assigns a procedural texture to slot `kind` of the selected material.
pub fn assign_procedural(state: &mut MaterialState, kind: MapKind, procedural: Procedural) {
    assign(state, kind, TextureRef::procedural(procedural));
}

fn assign(state: &mut MaterialState, kind: MapKind, texture: TextureRef) {
    let Some(material) = state.current_mut() else {
        return;
    };
    *material.maps.slot_mut(kind) = Some(texture);
    if kind == MapKind::Emission && material.emission == [0.0; 3] {
        material.emission = [1.0; 3];
    }
    edited(state);
}

/// Empties slot `kind` of the selected material.
pub fn clear_map(state: &mut MaterialState, kind: MapKind) {
    if let Some(material) = state.current_mut() {
        *material.maps.slot_mut(kind) = None;
        state.failures.retain(|(k, _)| *k != kind);
        edited(state);
    }
}

/// Restores the selected material's values to the defaults (name kept).
pub fn reset_selected(state: &mut MaterialState) {
    if let Some(material) = state.current_mut() {
        super::reset(material);
        state.failures.clear();
        edited(state);
    }
}

#[cfg(test)]
mod tests;
