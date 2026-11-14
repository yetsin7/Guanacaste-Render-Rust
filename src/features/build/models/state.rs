//! State of the objects panel: the placed instances, selection, history, tool options and the asset geometry cache.

use super::history::History;
use super::objects::{
    AxisLock, BrowserState, GizmoMode, MassOptions, ObjectsDoc, PlacedObject, PlacementOptions,
};
use crate::features::import_export::services::picker::FilePicker;
use crate::features::library::logic::procedural::Rng;
use crate::features::library::models::AssetRef;
use crate::features::library::services::{custom_asset, geometry_cache::GeometryCache};
use eframe::egui::TextureHandle;
use std::collections::{BTreeSet, HashMap};
use std::path::Path;
use std::sync::Arc;

use super::drag::Drag;

/// Height of the ground under a horizontal position `(x, y)`, in meters (the sculpted terrain).
pub type Ground = Arc<dyn Fn(f64, f64) -> f64 + Send + Sync>;

/// Everything the objects panel and the viewport interaction need.
pub struct ObjectsState {
    /// Serializable content: instances and imported assets.
    pub doc: ObjectsDoc,
    /// Ids of the selected instances.
    pub selection: BTreeSet<u64>,
    /// Undo and redo stacks.
    pub history: History,
    /// Shared geometry and thumbnails.
    pub cache: GeometryCache,
    /// Asset browser state.
    pub browser: BrowserState,
    /// What dragging does.
    pub mode: GizmoMode,
    /// Axis restriction.
    pub axis: AxisLock,
    /// Options for new instances.
    pub placement: PlacementOptions,
    /// Options of the mass tools.
    pub mass: MassOptions,
    /// Names of imported assets whose file could not be loaded when the project opened.
    pub missing: Vec<String>,
    /// Localization key of the last message (error or hint), if any.
    pub message: Option<&'static str>,
    /// True when the objects changed since they were loaded or saved.
    pub dirty: bool,
    /// The ground the objects rest on; `None` is the flat plane at Z = 0.
    pub ground: Option<Ground>,
    pub(crate) drag: Option<Drag>,
    pub(crate) picker: FilePicker,
    pub(crate) rng: Rng,
    pub(crate) textures: HashMap<(AssetRef, usize), TextureHandle>,
}

impl Default for ObjectsState {
    fn default() -> Self {
        Self::from_doc(ObjectsDoc::default())
    }
}

impl ObjectsState {
    /// Builds the state from a stored document, importing the models of the imported assets again from their
    /// source files; those that cannot be read are listed in `missing` and their instances are kept but not drawn.
    pub fn from_doc(doc: ObjectsDoc) -> Self {
        let mut cache = GeometryCache::default();
        let mut missing = Vec::new();
        for model in &doc.imported_assets {
            match custom_asset::import(Path::new(&model.source)) {
                Ok(scene) => cache.insert_imported(model.id, scene),
                Err(_) => missing.push(model.name.clone()),
            }
        }
        let mut state = Self {
            doc,
            selection: BTreeSet::new(),
            history: History::default(),
            cache,
            browser: BrowserState::default(),
            mode: GizmoMode::default(),
            axis: AxisLock::default(),
            placement: PlacementOptions::default(),
            mass: MassOptions::default(),
            missing,
            message: None,
            dirty: false,
            ground: None,
            drag: None,
            picker: FilePicker::default(),
            rng: Rng::new(0x6A09_E667),
            textures: HashMap::new(),
        };
        let floor = state.doc.instances.iter().map(|i| i.id + 1);
        let floor = floor.chain(state.doc.imported_assets.iter().map(|a| a.id + 1));
        state.doc.next_id = state.doc.next_id.max(floor.max().unwrap_or(1)).max(1);
        state
    }

    /// The document to store in the project.
    pub fn to_doc(&self) -> ObjectsDoc {
        self.doc.clone()
    }

    /// Replaces the content with `doc` (a project was opened or created).
    pub fn load(&mut self, doc: ObjectsDoc) {
        let browser = self.browser.clone();
        *self = Self::from_doc(doc);
        self.browser = browser;
    }

    /// Placed instances.
    pub fn instances(&self) -> &[PlacedObject] {
        &self.doc.instances
    }

    /// The instance with `id`.
    pub fn instance(&self, id: u64) -> Option<&PlacedObject> {
        self.doc.instances.iter().find(|i| i.id == id)
    }

    /// The only selected instance, when exactly one is selected.
    pub fn single(&self) -> Option<&PlacedObject> {
        match self.selection.len() {
            1 => self.instance(*self.selection.iter().next()?),
            _ => None,
        }
    }

    pub(crate) fn next_id(&mut self) -> u64 {
        let id = self.doc.next_id.max(1);
        self.doc.next_id = id + 1;
        id
    }
}
