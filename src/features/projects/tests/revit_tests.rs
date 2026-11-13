//! The sample `.gr` written by the Revit add-in's own C# writer (`plugins/revit/src/Format/GrFile.cs`, generated with
//! `dotnet run --project plugins/revit/tests/GrFormatCheck`), checked against the layout in
//! `docs/plugin-export-format.md` and opened through the normal `open()` path.

use crate::features::import_export::models::NodeKind;
use crate::features::projects::logic::convert;
use crate::features::projects::services::container::{Limits, Reader, KIND_MESH};
use crate::features::projects::services::open::open;
use std::path::PathBuf;

/// Exact JSON text the C# writer produces for the sample scene.
const JSON: &str = r#"{"meta":{"name":"Casa de prueba","created":1700000000,"modified":1700000000,"app_version":"revit-addin 1.0.0"},"origin":{"format":"Revit","version":"Revit 2026 (compilación 20260101)","application":"Autodesk Revit 2026","file_name":"casa.rvt","file_size":1234567,"length_unit":"ft","unit_scale":0.30480000000000002,"imported":1700000000},"origin_shift":[500.25,-300.5,0],"levels":[{"name":"Nivel 1","elevation":0,"node":1},{"name":"Nivel 2","elevation":3.048,"node":2}],"materials":[{"name":"Concreto","color":[0.5,0.5,0.5,1]},{"name":"Vidrio","color":[0.600000024,0.800000012,0.899999976,0.25]}],"nodes":[{"name":"Casa de prueba","class":"Revit","kind":"project"},{"name":"Nivel 1","class":"Level","kind":"level","parent":0,"level":0},{"name":"Nivel 2","class":"Level","kind":"level","parent":0,"level":1},{"name":"Muros","class":"Muros","kind":"group","parent":1,"level":0},{"name":"Muro básico [100]","class":"Muros","kind":"element","type_name":"Muro básico: Genérico - 200 mm","parent":3,"meshes":[0],"level":0},{"name":"Ventanas","class":"Ventanas","kind":"group","parent":2,"level":1},{"name":"Ventana \"ñ\" [200]\t","class":"Ventanas","kind":"element","parent":5,"meshes":[1],"level":1}],"meshes":[{"material":0,"vertices":4,"indices":6},{"material":1,"vertices":3,"indices":3}]}"#;

fn golden() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("plugins/revit/tests/fixtures/sample.gr")
}

/// `[nv, ni].pack("VV") + positions + indices`, assembled by hand.
fn mesh_block(positions: &[f32], indices: &[u32]) -> Vec<u8> {
    let mut out = Vec::new();
    out.extend_from_slice(&((positions.len() / 3) as u32).to_le_bytes());
    out.extend_from_slice(&(indices.len() as u32).to_le_bytes());
    positions
        .iter()
        .for_each(|v| out.extend_from_slice(&v.to_le_bytes()));
    indices
        .iter()
        .for_each(|i| out.extend_from_slice(&i.to_le_bytes()));
    out
}

#[test]
fn the_csharp_container_has_the_documented_layout() {
    let bytes = std::fs::read(golden()).expect("fixture");
    assert_eq!(&bytes[..8], b"GRRPROJ1");
    assert_eq!(&bytes[8..12], &[1, 0, 0, 0]);
    assert_eq!(&bytes[12..16], &[1, 0, 0, 0]);
    assert_eq!(&bytes[16..32], &(1u8..=16).collect::<Vec<_>>()[..]);
    let mut reader = Reader::open(&golden(), Limits::default()).expect("verified container");
    assert_eq!(reader.header.block_count, 2);
    assert_eq!(std::str::from_utf8(&reader.json).unwrap(), JSON);
    let kinds: Vec<u32> = reader.blocks.iter().map(|b| b.kind).collect();
    assert_eq!(kinds, [KIND_MESH; 2]);
    let wall = mesh_block(
        &[0., 0., 0., 1., 0., 0., 1., 1., 0., 0., 1., 0.],
        &[0, 1, 2, 0, 2, 3],
    );
    assert_eq!(reader.read_block(0).expect("block 0"), wall);
    let window = mesh_block(&[0.1, 0.2, 3.048, 2., 0., 3.048, 2., 0., 4.5], &[0, 1, 2]);
    assert_eq!(reader.read_block(1).expect("block 1"), window);
}

#[test]
fn the_csharp_project_opens_with_hierarchy_materials_and_levels() {
    let opened = open(&golden()).expect("opens");
    let document = &opened.project.document;
    assert_eq!(document.meta.name, "Casa de prueba");
    assert_eq!(document.origin_shift, [500.25, -300.5, 0.0]);
    let origin = document.origin.as_ref().expect("origin");
    assert_eq!(origin.format, "Revit");
    assert_eq!(origin.length_unit, "ft");
    assert_eq!(document.levels.len(), 2);
    assert_eq!(document.levels[1].name, "Nivel 2");
    assert_eq!(document.levels[1].elevation, 3.048);
    assert_eq!(document.materials[1].color, [0.6, 0.8, 0.9, 0.25]);
    assert_eq!(opened.project.meshes.len(), 2);
    assert_eq!(opened.project.meshes[0].material, Some(0));
    assert_eq!(opened.project.meshes[1].positions[0], [0.1, 0.2, 3.048]);

    let scene = convert::to_scene(document, &opened.project.meshes, 0);
    assert_eq!(scene.nodes[0].kind, NodeKind::Project);
    let wall = scene
        .nodes
        .iter()
        .find(|n| n.kind == NodeKind::Element && n.name.starts_with("Muro"))
        .expect("wall");
    assert_eq!(
        wall.type_name.as_deref(),
        Some("Muro básico: Genérico - 200 mm")
    );
    assert_eq!(wall.level, Some(0));
    let group = wall.parent.expect("category group");
    assert_eq!(scene.nodes[group].kind, NodeKind::Group);
    let level = scene.nodes[group].parent.expect("level node");
    assert_eq!(scene.nodes[level].kind, NodeKind::Level);
    let window = scene
        .nodes
        .iter()
        .find(|n| n.kind == NodeKind::Element && n.name.starts_with("Ventana"))
        .expect("window");
    assert_eq!(window.name, "Ventana \"ñ\" [200]\t");
    assert_eq!(scene.triangle_count(), 3);
}
