//! The undo/redo command stack of object edits.

use super::objects::{PlacedObject, Transform};

/// Maximum commands kept for undo.
pub const HISTORY_LIMIT: usize = 200;

/// One reversible edit of the placed objects.
#[derive(Debug, Clone, PartialEq)]
pub enum Command {
    /// Instances were added.
    Add(Vec<PlacedObject>),
    /// Instances were removed; each keeps the index it had.
    Remove(Vec<(usize, PlacedObject)>),
    /// Transforms changed: `(id, before, after)`.
    Transform(Vec<(u64, Transform, Transform)>),
}

impl Command {
    /// Applies the command to `instances`.
    pub fn apply(&self, instances: &mut Vec<PlacedObject>) {
        match self {
            Command::Add(added) => instances.extend(added.iter().cloned()),
            Command::Remove(removed) => {
                instances.retain(|i| !removed.iter().any(|(_, r)| r.id == i.id))
            }
            Command::Transform(changes) => set_transforms(instances, changes, true),
        }
    }

    /// Reverts the command on `instances`.
    pub fn revert(&self, instances: &mut Vec<PlacedObject>) {
        match self {
            Command::Add(added) => instances.retain(|i| !added.iter().any(|a| a.id == i.id)),
            Command::Remove(removed) => {
                let mut sorted: Vec<&(usize, PlacedObject)> = removed.iter().collect();
                sorted.sort_by_key(|(index, _)| *index);
                for (index, object) in sorted {
                    instances.insert((*index).min(instances.len()), object.clone());
                }
            }
            Command::Transform(changes) => set_transforms(instances, changes, false),
        }
    }

    /// Ids of the instances the command touches.
    pub fn ids(&self) -> Vec<u64> {
        match self {
            Command::Add(items) => items.iter().map(|i| i.id).collect(),
            Command::Remove(items) => items.iter().map(|(_, i)| i.id).collect(),
            Command::Transform(items) => items.iter().map(|(id, _, _)| *id).collect(),
        }
    }
}

fn set_transforms(
    instances: &mut [PlacedObject],
    changes: &[(u64, Transform, Transform)],
    after: bool,
) {
    for (id, before, next) in changes {
        if let Some(object) = instances.iter_mut().find(|i| i.id == *id) {
            object.transform = if after { *next } else { *before };
        }
    }
}

/// Undo and redo stacks.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct History {
    undo: Vec<Command>,
    redo: Vec<Command>,
}

impl History {
    /// Records a command that was just applied; the redo stack is cleared.
    pub fn record(&mut self, command: Command) {
        self.redo.clear();
        self.undo.push(command);
        if self.undo.len() > HISTORY_LIMIT {
            self.undo.remove(0);
        }
    }

    /// Whether there is something to undo.
    pub fn can_undo(&self) -> bool {
        !self.undo.is_empty()
    }

    /// Whether there is something to redo.
    pub fn can_redo(&self) -> bool {
        !self.redo.is_empty()
    }

    /// Pops the latest command to revert it (it moves to the redo stack).
    pub fn take_undo(&mut self) -> Option<Command> {
        let command = self.undo.pop()?;
        self.redo.push(command.clone());
        Some(command)
    }

    /// Pops the latest undone command to apply it again.
    pub fn take_redo(&mut self) -> Option<Command> {
        let command = self.redo.pop()?;
        self.undo.push(command.clone());
        Some(command)
    }

    /// Forgets everything (a new project was opened).
    pub fn clear(&mut self) {
        self.undo.clear();
        self.redo.clear();
    }
}
