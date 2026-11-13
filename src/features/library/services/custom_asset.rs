//! Turning a model of the user (IFC, OBJ, glTF, FBX...) into a library asset, through the existing importers.

use crate::features::import_export::models::{ImportError, ImportOptions, ImportedScene};
use crate::features::import_export::services::{job, progress::Progress};
use std::path::Path;

/// Moves `scene` so its footprint is centered on the origin and its lowest point rests on Z = 0, which is how every
/// library asset is placed.
pub fn normalize(scene: &mut ImportedScene) {
    let Some(bounds) = scene.bounds() else {
        return;
    };
    let shift = [
        (bounds.min[0] + bounds.max[0]) * 0.5,
        (bounds.min[1] + bounds.max[1]) * 0.5,
        bounds.min[2],
    ];
    for mesh in &mut scene.meshes {
        for point in &mut mesh.positions {
            for axis in 0..3 {
                point[axis] -= shift[axis];
            }
        }
    }
    for axis in 0..3 {
        scene.origin_shift[axis] += f64::from(shift[axis]);
    }
}

/// Imports `path` with the regular importers and normalizes it as an asset. Runs on the calling thread.
pub fn import(path: &Path) -> Result<ImportedScene, ImportError> {
    let outcome = job::run(path, &ImportOptions::default(), &Progress::default())?;
    let mut scene = outcome.scene;
    normalize(&mut scene);
    Ok(scene)
}

/// Display name of an imported asset: the file stem, or the whole file name.
pub fn display_name(path: &Path) -> String {
    path.file_stem()
        .or_else(|| path.file_name())
        .map(|name| name.to_string_lossy().into_owned())
        .unwrap_or_default()
}
