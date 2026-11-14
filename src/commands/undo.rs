//! Global undo and redo (Ctrl+Z / Ctrl+Y) for placed objects and the landscape, in the order the edits were made.
//! Objects keep their own command history; the landscape is restored from snapshots taken when an edit (a whole
//! brush stroke or a panel action) begins. Loading another project forgets everything.

use crate::features::landscape::models::Landscape;
use crate::models::AppState;
use crate::services::localization::tr;

/// Most landscape snapshots kept at once.
const LANDSCAPE_LIMIT: usize = 24;

/// One entry of the shared timeline.
enum Entry {
    /// An edit recorded in the objects' own history.
    Objects,
    /// The landscape as it was before (undo list) or after (redo list) an edit.
    Landscape(Box<Landscape>),
}

/// The shared undo and redo timeline.
#[derive(Default)]
pub struct UndoHub {
    undo: Vec<Entry>,
    redo: Vec<Entry>,
    /// The landscape as of the last settled moment (the state before the edit being made).
    settled: Option<Landscape>,
    seen_revision: u64,
    seen_objects: u64,
    in_stroke: bool,
}

impl UndoHub {
    /// An empty history that starts watching `app` from now (a project was opened or created).
    pub fn new(app: &AppState) -> Self {
        Self {
            seen_revision: app.landscape.revision,
            seen_objects: app.objects.history.recorded(),
            settled: Some(app.landscape.landscape.clone()),
            ..Self::default()
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

    fn push_undo(&mut self, entry: Entry) {
        self.redo.clear();
        self.undo.push(entry);
        let snapshots = self
            .undo
            .iter()
            .filter(|e| matches!(e, Entry::Landscape(_)))
            .count();
        if snapshots > LANDSCAPE_LIMIT {
            if let Some(oldest) = self
                .undo
                .iter()
                .position(|e| matches!(e, Entry::Landscape(_)))
            {
                self.undo.remove(oldest);
            }
        }
    }
}

/// Notices the edits of this frame. `editing` is whether the pointer is down (a brush stroke may be going on).
/// Call once per frame.
pub fn track(app: &mut AppState, editing: bool) {
    let revision = app.landscape.revision;
    let recorded = app.objects.history.recorded();
    let hub = &mut app.undo;
    if recorded != hub.seen_objects {
        hub.seen_objects = recorded;
        hub.push_undo(Entry::Objects);
    }
    if revision != hub.seen_revision {
        hub.seen_revision = revision;
        if !hub.in_stroke {
            if let Some(before) = hub.settled.clone() {
                hub.push_undo(Entry::Landscape(Box::new(before)));
            }
            hub.in_stroke = true;
        }
    }
    if !editing && (hub.in_stroke || hub.settled.is_none()) {
        hub.in_stroke = false;
        hub.settled = Some(app.landscape.landscape.clone());
    }
}

/// Reverts the latest edit; returns the localization key of what was done, or of why nothing was.
pub fn undo(app: &mut AppState) -> &'static str {
    // Entries whose objects were already undone from the objects panel are skipped.
    while matches!(app.undo.undo.last(), Some(Entry::Objects)) && !app.objects.history.can_undo() {
        app.undo.undo.pop();
    }
    let Some(entry) = app.undo.undo.pop() else {
        return "undo.nothing";
    };
    match entry {
        Entry::Objects => {
            app.objects.undo();
            app.undo.seen_objects = app.objects.history.recorded();
            app.undo.redo.push(Entry::Objects);
        }
        Entry::Landscape(before) => {
            let after = std::mem::replace(&mut app.landscape.landscape, *before);
            app.undo.redo.push(Entry::Landscape(Box::new(after)));
            settle(app);
        }
    }
    "undo.done"
}

/// Applies the latest undone edit again.
pub fn redo(app: &mut AppState) -> &'static str {
    while matches!(app.undo.redo.last(), Some(Entry::Objects)) && !app.objects.history.can_redo() {
        app.undo.redo.pop();
    }
    let Some(entry) = app.undo.redo.pop() else {
        return "redo.nothing";
    };
    match entry {
        Entry::Objects => {
            app.objects.redo();
            app.undo.seen_objects = app.objects.history.recorded();
            app.undo.undo.push(Entry::Objects);
        }
        Entry::Landscape(after) => {
            let before = std::mem::replace(&mut app.landscape.landscape, *after);
            app.undo.undo.push(Entry::Landscape(Box::new(before)));
            settle(app);
        }
    }
    "redo.done"
}

/// After the landscape was swapped: refresh what depends on it and make it the new settled state.
fn settle(app: &mut AppState) {
    app.landscape.touch();
    app.undo.seen_revision = app.landscape.revision;
    app.undo.settled = Some(app.landscape.landscape.clone());
    app.undo.in_stroke = false;
}

/// Undo or redo with a notice for the user.
pub fn run(app: &mut AppState, redo_it: bool) {
    let key = if redo_it { redo(app) } else { undo(app) };
    app.workspace.notify(tr(key).to_owned(), false);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::features::build::models::{PlacedObject, Transform};
    use crate::features::landscape::logic::actions;
    use crate::features::library::models::{AssetRef, ProceduralKind};

    fn app() -> AppState {
        let folder = std::env::temp_dir().join("gr-undo-hub");
        let mut app = AppState::with_data_dir(folder);
        app.undo = UndoHub::new(&app);
        app
    }

    #[test]
    fn landscape_edits_and_objects_undo_in_the_order_they_were_made() {
        let mut app = app();
        let flat = app.landscape.landscape.terrain.heights.clone();
        // A relief generated from a panel button.
        actions::new_flat(&mut app.landscape, 21, 21, 4.0);
        app.landscape
            .landscape
            .terrain
            .heights
            .iter_mut()
            .for_each(|h| *h = 2.0);
        app.landscape.touch();
        track(&mut app, false);
        // Then an object.
        app.objects.add_objects(vec![(
            AssetRef::Procedural {
                kind: ProceduralKind::Tree,
                seed: 1,
            },
            Transform::at([1.0, 1.0, 0.0]),
        )]);
        track(&mut app, false);
        assert!(app.undo.can_undo() && !app.undo.can_redo());
        undo(&mut app);
        assert!(app.objects.instances().is_empty(), "the object goes first");
        assert_eq!(app.landscape.landscape.terrain.heights[0], 2.0);
        undo(&mut app);
        assert_eq!(app.landscape.landscape.terrain.heights.len(), flat.len());
        assert_eq!(app.landscape.landscape.terrain.heights[0], 0.0);
        assert_eq!(undo(&mut app), "undo.nothing");
        redo(&mut app);
        assert_eq!(app.landscape.landscape.terrain.heights[0], 2.0);
        redo(&mut app);
        assert_eq!(app.objects.instances().len(), 1);
        assert_eq!(redo(&mut app), "redo.nothing");
    }

    #[test]
    fn a_whole_brush_stroke_is_one_undo_step() {
        let mut app = app();
        for dab in 0..5 {
            app.landscape.landscape.terrain.heights[0] = dab as f32;
            app.landscape.touch();
            track(&mut app, true);
        }
        track(&mut app, false);
        assert_eq!(app.undo.undo.len(), 1);
        undo(&mut app);
        assert_eq!(app.landscape.landscape.terrain.heights[0], 0.0);
    }

    #[test]
    fn a_new_edit_clears_the_redo_list() {
        let mut app = app();
        app.objects.doc.instances.push(PlacedObject {
            id: 1,
            asset: AssetRef::Procedural {
                kind: ProceduralKind::Bush,
                seed: 1,
            },
            transform: Transform::default(),
        });
        app.landscape.touch();
        track(&mut app, false);
        undo(&mut app);
        assert!(app.undo.can_redo());
        app.landscape.touch();
        track(&mut app, false);
        assert!(!app.undo.can_redo());
    }
}
