//! Unit tests of the import domain. The IFC fixtures are synthetic: written by hand to imitate Revit, never
//! validated against a real Revit export (see `tests/real_files.rs` for that).

pub(crate) mod fixtures;
mod formats_tests;
mod ifc_tests;
mod robustness_tests;

use crate::features::import_export::models::{
    ImportOptions, ImportOutcome, ImportedScene, NodeKind, SceneNode,
};
use crate::features::import_export::services::{ifc, progress::Progress};

/// Imports IFC text with default options.
pub(crate) fn import_ifc(text: &str) -> ImportOutcome {
    ifc::import_reader(
        text.as_bytes(),
        text.len() as u64,
        &ImportOptions::default(),
        &Progress::default(),
    )
    .expect("fixture imports")
}

/// The element named `name`.
pub(super) fn element<'a>(scene: &'a ImportedScene, name: &str) -> &'a SceneNode {
    scene
        .nodes
        .iter()
        .find(|node| node.kind == NodeKind::Element && node.name == name)
        .unwrap_or_else(|| panic!("element {name} not found"))
}

/// World-space bounds (meters, shift restored) of a node's meshes.
pub(super) fn world_bounds(scene: &ImportedScene, node: &SceneNode) -> ([f64; 3], [f64; 3]) {
    let (mut min, mut max) = ([f64::INFINITY; 3], [f64::NEG_INFINITY; 3]);
    for mesh in node.meshes.iter().map(|i| &scene.meshes[*i]) {
        for p in &mesh.positions {
            for axis in 0..3 {
                let value = p[axis] as f64 + scene.origin_shift[axis];
                min[axis] = min[axis].min(value);
                max[axis] = max[axis].max(value);
            }
        }
    }
    (min, max)
}

/// Triangles of a node.
pub(super) fn triangles(scene: &ImportedScene, node: &SceneNode) -> usize {
    node.meshes
        .iter()
        .map(|i| scene.meshes[*i].indices.len() / 3)
        .sum()
}
