//! Conversion between an imported scene and the project document + mesh blocks, in both directions.

use crate::features::import_export::models::{
    ImportedScene, Level, Material, Mesh, NodeKind, SceneNode, SourceFormat, SourceInfo,
};
use crate::features::projects::models::{
    ImportOrigin, LevelDoc, MaterialDoc, MeshMeta, NodeDoc, ProjectDocument,
};

fn kind_name(kind: NodeKind) -> &'static str {
    match kind {
        NodeKind::Project => "project",
        NodeKind::Site => "site",
        NodeKind::Building => "building",
        NodeKind::Level => "level",
        NodeKind::Element => "element",
        NodeKind::Group => "group",
    }
}

fn kind_from(name: &str) -> NodeKind {
    match name {
        "project" => NodeKind::Project,
        "site" => NodeKind::Site,
        "building" => NodeKind::Building,
        "level" => NodeKind::Level,
        "element" => NodeKind::Element,
        _ => NodeKind::Group,
    }
}

/// Fills `document` (hierarchy, levels, materials, mesh metadata, origin) from `scene` and returns its meshes.
/// `file_name` is the original file name and `now` the current time in Unix seconds.
pub fn from_scene(
    scene: &ImportedScene,
    name: &str,
    file_name: &str,
    now: u64,
) -> (ProjectDocument, Vec<Mesh>) {
    let mut document = ProjectDocument::default();
    document.meta.name = name.to_owned();
    document.meta.created = now;
    document.meta.modified = now;
    document.origin = Some(ImportOrigin {
        format: scene.source.format.name().to_owned(),
        version: scene.source.version.clone(),
        application: scene.source.application.clone(),
        file_name: file_name.to_owned(),
        file_size: scene.source.file_size,
        length_unit: scene.source.length_unit.clone(),
        unit_scale: scene.source.unit_scale,
        imported: now,
    });
    document.origin_shift = scene.origin_shift;
    document.levels = scene
        .levels
        .iter()
        .map(|l| LevelDoc {
            name: l.name.clone(),
            elevation: l.elevation,
            node: l.node as u32,
        })
        .collect();
    document.materials = scene
        .materials
        .iter()
        .map(|m| MaterialDoc {
            name: m.name.clone(),
            color: m.color,
        })
        .collect();
    document.nodes = scene
        .nodes
        .iter()
        .map(|n| NodeDoc {
            name: n.name.clone(),
            class: n.class.clone(),
            kind: kind_name(n.kind).to_owned(),
            type_name: n.type_name.clone(),
            parent: n.parent.map(|p| p as u32),
            meshes: n.meshes.iter().map(|m| *m as u32).collect(),
            level: n.level.map(|l| l as u32),
        })
        .collect();
    document.meshes = scene
        .meshes
        .iter()
        .map(|m| MeshMeta {
            material: m.material.map(|i| i as u32),
            vertices: m.positions.len() as u32,
            indices: m.indices.len() as u32,
        })
        .collect();
    (document, scene.meshes.clone())
}

/// Rebuilds the scene of a project; children lists are derived from the parent links.
pub fn to_scene(
    document: &ProjectDocument,
    meshes: &[Mesh],
    file_size_fallback: u64,
) -> ImportedScene {
    let mut nodes: Vec<SceneNode> = document
        .nodes
        .iter()
        .map(|n| SceneNode {
            name: n.name.clone(),
            class: n.class.clone(),
            kind: kind_from(&n.kind),
            type_name: n.type_name.clone(),
            parent: n.parent.map(|p| p as usize),
            children: Vec::new(),
            meshes: n.meshes.iter().map(|m| *m as usize).collect(),
            level: n.level.map(|l| l as usize),
        })
        .collect();
    for index in 0..nodes.len() {
        if let Some(parent) = nodes[index].parent.filter(|p| *p < nodes.len()) {
            nodes[parent].children.push(index);
        }
    }
    let origin = document.origin.as_ref();
    let format = match origin.map(|o| o.format.as_str()) {
        Some("OBJ") => SourceFormat::Obj,
        Some("glTF") => SourceFormat::Gltf,
        Some("FBX") => SourceFormat::Fbx,
        Some("Guanacaste 3D") => SourceFormat::Legacy,
        _ => SourceFormat::Ifc,
    };
    ImportedScene {
        source: SourceInfo {
            format,
            version: origin.map(|o| o.version.clone()).unwrap_or_default(),
            application: origin.map(|o| o.application.clone()).unwrap_or_default(),
            length_unit: origin.map_or_else(|| "m".into(), |o| o.length_unit.clone()),
            unit_scale: origin.map_or(1.0, |o| o.unit_scale),
            file_size: origin.map_or(file_size_fallback, |o| o.file_size),
        },
        nodes,
        meshes: meshes.to_vec(),
        materials: document
            .materials
            .iter()
            .map(|m| Material {
                name: m.name.clone(),
                color: m.color,
            })
            .collect(),
        levels: document
            .levels
            .iter()
            .map(|l| Level {
                name: l.name.clone(),
                elevation: l.elevation,
                node: l.node as usize,
            })
            .collect(),
        origin_shift: document.origin_shift,
    }
}
