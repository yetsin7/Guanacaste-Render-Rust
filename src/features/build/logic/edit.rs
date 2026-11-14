//! Editing the placed objects: every change goes through a command so it can be undone and redone.

use crate::features::build::models::{Command, ObjectsState, PlacedObject, Transform};
use crate::features::import_export::models::ImportError;
use crate::features::library::models::AssetRef;
use crate::features::library::models::ImportedAssetDoc;
use crate::features::library::services::custom_asset;
use std::path::Path;

impl ObjectsState {
    /// Applies `command`, records it for undo and marks the project as changed.
    pub fn commit(&mut self, command: Command) {
        command.apply(&mut self.doc.instances);
        self.history.record(command);
        self.dirty = true;
    }

    /// Creates the instances for `objects` (ids are assigned here), selects them and records one command.
    pub fn add_objects(&mut self, objects: Vec<(AssetRef, Transform)>) -> Vec<u64> {
        let mut added = Vec::with_capacity(objects.len());
        for (asset, transform) in objects {
            let id = self.next_id();
            added.push(PlacedObject {
                id,
                asset,
                transform,
            });
        }
        let ids: Vec<u64> = added.iter().map(|o| o.id).collect();
        if !added.is_empty() {
            self.commit(Command::Add(added));
            self.selection = ids.iter().copied().collect();
        }
        ids
    }

    /// Changes the ground the objects rest on. Objects keep their height above it: when a ground was already set
    /// (the terrain was sculpted) every instance follows the change; the first ground only records itself.
    pub fn set_ground(&mut self, ground: Option<super::super::models::Ground>) {
        let old = self.ground.take();
        let moved: Vec<(usize, f64)> = match (&old, &ground) {
            (Some(before), Some(after)) => self
                .doc
                .instances
                .iter()
                .enumerate()
                .map(|(index, i)| {
                    let [x, y, _] = i.transform.position;
                    (index, after(x, y) - before(x, y))
                })
                .filter(|(_, delta)| delta.abs() > 1e-6)
                .collect(),
            _ => Vec::new(),
        };
        for (index, delta) in &moved {
            self.doc.instances[*index].transform.position[2] += delta;
        }
        if !moved.is_empty() {
            self.dirty = true;
        }
        self.ground = ground;
    }

    /// Deletes the selected instances.
    pub fn delete_selected(&mut self) {
        let removed: Vec<(usize, PlacedObject)> = self
            .doc
            .instances
            .iter()
            .enumerate()
            .filter(|(_, i)| self.selection.contains(&i.id))
            .map(|(index, i)| (index, i.clone()))
            .collect();
        if !removed.is_empty() {
            self.commit(Command::Remove(removed));
        }
        self.selection.clear();
    }

    /// Changes the transform of every selected instance with `change`, as one undoable step.
    pub fn edit_selection(&mut self, mut change: impl FnMut(&PlacedObject, &mut Transform)) {
        let changes: Vec<_> = self
            .doc
            .instances
            .iter()
            .filter(|i| self.selection.contains(&i.id))
            .filter_map(|i| {
                let mut after = i.transform;
                change(i, &mut after);
                (after != i.transform).then_some((i.id, i.transform, after))
            })
            .collect();
        if !changes.is_empty() {
            self.commit(Command::Transform(changes));
        }
    }

    /// Reverts the latest edit; the affected instances become the selection.
    pub fn undo(&mut self) {
        if let Some(command) = self.history.take_undo() {
            command.revert(&mut self.doc.instances);
            self.after_history(&command);
        }
    }

    /// Applies the latest undone edit again.
    pub fn redo(&mut self) {
        if let Some(command) = self.history.take_redo() {
            command.apply(&mut self.doc.instances);
            self.after_history(&command);
        }
    }

    fn after_history(&mut self, command: &Command) {
        self.dirty = true;
        self.selection = command
            .ids()
            .into_iter()
            .filter(|id| self.instance(*id).is_some())
            .collect();
    }

    /// Selects every instance.
    pub fn select_all(&mut self) {
        self.selection = self.doc.instances.iter().map(|i| i.id).collect();
    }

    /// Clears the selection.
    pub fn clear_selection(&mut self) {
        self.selection.clear();
    }

    /// Selects `id`, or toggles it when `additive`.
    pub fn select(&mut self, id: u64, additive: bool) {
        if !additive {
            self.selection.clear();
            self.selection.insert(id);
        } else if !self.selection.remove(&id) {
            self.selection.insert(id);
        }
    }

    /// Imports a model file as a library asset and makes it the active asset.
    pub fn import_model(&mut self, path: &Path) -> Result<u64, ImportError> {
        let scene = custom_asset::import(path)?;
        let id = self.next_id();
        self.doc.imported_assets.push(ImportedAssetDoc {
            id,
            name: custom_asset::display_name(path),
            source: path.to_string_lossy().into_owned(),
        });
        self.cache.insert_imported(id, scene);
        self.browser.active = Some(AssetRef::Imported { id });
        self.dirty = true;
        Ok(id)
    }
}
