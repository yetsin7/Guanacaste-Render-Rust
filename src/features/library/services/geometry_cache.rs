//! Geometry of every asset in use, generated or imported once and shared by all its instances.

use super::thumbnail::{self, Thumbnail};
use crate::features::import_export::models::{Bounds, ImportedScene};
use crate::features::library::logic::procedural;
use crate::features::library::models::AssetRef;
use std::collections::HashMap;
use std::sync::Arc;

/// Meshes and thumbnails keyed by asset.
#[derive(Debug, Default, Clone)]
pub struct GeometryCache {
    scenes: HashMap<AssetRef, Arc<ImportedScene>>,
    thumbnails: HashMap<(AssetRef, usize), Arc<Thumbnail>>,
}

impl GeometryCache {
    /// Registers the geometry of an imported model.
    pub fn insert_imported(&mut self, id: u64, scene: ImportedScene) {
        let asset = AssetRef::Imported { id };
        self.thumbnails.retain(|(key, _), _| *key != asset);
        self.scenes.insert(asset, Arc::new(scene));
    }

    /// Forgets an imported model.
    pub fn remove_imported(&mut self, id: u64) {
        let asset = AssetRef::Imported { id };
        self.scenes.remove(&asset);
        self.thumbnails.retain(|(key, _), _| *key != asset);
    }

    /// The geometry of `asset`, generating procedural ones on first use; `None` for an imported model whose file
    /// could not be loaded.
    pub fn scene(&mut self, asset: AssetRef) -> Option<Arc<ImportedScene>> {
        if let Some(scene) = self.scenes.get(&asset) {
            return Some(Arc::clone(scene));
        }
        let AssetRef::Procedural { kind, seed } = asset else {
            return None;
        };
        let scene = Arc::new(procedural::generate(kind, seed));
        self.scenes.insert(asset, Arc::clone(&scene));
        Some(scene)
    }

    /// Local bounds of `asset` (meters, before the instance transform).
    pub fn bounds(&mut self, asset: AssetRef) -> Option<Bounds> {
        self.scene(asset)?.bounds()
    }

    /// A `size`×`size` thumbnail of `asset`, rendered on the CPU on first use.
    pub fn thumbnail(&mut self, asset: AssetRef, size: usize) -> Option<Arc<Thumbnail>> {
        if let Some(image) = self.thumbnails.get(&(asset, size)) {
            return Some(Arc::clone(image));
        }
        let scene = self.scene(asset)?;
        let image = Arc::new(thumbnail::render(&scene, size));
        self.thumbnails.insert((asset, size), Arc::clone(&image));
        Some(image)
    }
}
