//! Gives the materials of an imported scene their realistic PBR version, element by element: the same flat style
//! can be a wall render on a wall and a ceiling on a ceiling, so a style used by several kinds of surface is split.

use super::{classify, derive, SurfaceInfo, SurfaceKind};
use crate::features::import_export::models::{ImportedScene, Material, SourceFormat};
use crate::features::materials::logic::convert;
use crate::features::materials::logic::procedural::color_at;
use crate::features::materials::models::{Material as Pbr, TextureSource};
use crate::services::localization::trf;
use std::collections::HashMap;

/// Whether `scene` came from a format whose materials are only flat colors that should be upgraded.
pub fn needs_enrichment(scene: &ImportedScene) -> bool {
    scene.source.format == SourceFormat::Ifc && scene.materials.iter().all(|m| m.pbr.is_none())
}

/// The flat color that stands for `pbr` where textures are not drawn (the 3D view): the base color times the average
/// of its albedo pattern, with the opacity as alpha.
fn flat_color(pbr: &Pbr) -> [f32; 4] {
    let mut color = convert::to_scene(pbr).color;
    let pattern = pbr.maps.albedo.as_ref().and_then(|t| match &t.source {
        TextureSource::Procedural(p) => Some(*p),
        TextureSource::File { .. } => None,
    });
    if let Some(pattern) = pattern {
        let mut sum = [0.0f32; 3];
        const GRID: usize = 12;
        for i in 0..GRID * GRID {
            let (u, v) = (
                (i % GRID) as f32 / GRID as f32,
                (i / GRID) as f32 / GRID as f32,
            );
            let c = color_at(&pattern, u, v);
            for k in 0..3 {
                sum[k] += c[k];
            }
        }
        for k in 0..3 {
            color[k] *= sum[k] / (GRID * GRID) as f32;
        }
    }
    color
}

/// Replaces the flat materials of `scene` by PBR ones derived from their names and the element they belong to.
/// Meshes of one style that need different families (a wall and a roof) get their own material; the first family
/// keeps the original slot. Surfaces that are hidden by default become fully transparent. Does nothing when the
/// scene already carries PBR materials.
pub fn enrich_scene(scene: &mut ImportedScene) {
    if !needs_enrichment(scene) {
        return;
    }
    let mut owner = vec![None; scene.meshes.len()];
    for (index, node) in scene.nodes.iter().enumerate() {
        for &mesh in &node.meshes {
            if let Some(slot) = owner.get_mut(mesh) {
                *slot = Some(index);
            }
        }
    }
    let originals = scene.materials.clone();
    let mut used = vec![false; originals.len()];
    let mut variants: HashMap<(usize, String, SurfaceKind), usize> = HashMap::new();
    for (mesh_index, mesh) in scene.meshes.iter_mut().enumerate() {
        let Some(old) = mesh.material.filter(|m| *m < originals.len()) else {
            continue;
        };
        let node = owner[mesh_index].map(|n| &scene.nodes[n]);
        let class = node.map_or("", |n| n.class.as_str());
        let source = &originals[old];
        let info = SurfaceInfo {
            name: &source.name,
            type_name: node.and_then(|n| n.type_name.as_deref()),
            element: node.map(|n| n.name.as_str()),
            class,
            color: source.color,
        };
        let kind = classify(&info);
        let key = (old, class.to_owned(), kind);
        let index = *variants.entry(key).or_insert_with(|| {
            let pbr = derive(&info);
            let mut material = Material {
                color: flat_color(&pbr),
                texture: None,
                ..convert::to_scene(&pbr)
            };
            material.pbr = Some(Box::new(pbr));
            if !used[old] {
                used[old] = true;
                scene.materials[old] = material;
                old
            } else {
                material.name = trf(
                    "material.derived_name",
                    &[
                        &source.name,
                        crate::services::localization::tr(kind.label_key()),
                    ],
                );
                if let Some(pbr) = material.pbr.as_mut() {
                    pbr.name = material.name.clone();
                }
                scene.materials.push(material);
                scene.materials.len() - 1
            }
        });
        mesh.material = Some(index);
    }
}
