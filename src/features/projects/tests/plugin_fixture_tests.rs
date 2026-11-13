//! A `.gr` built byte by byte exactly as the SketchUp plugin (Ruby `pack`) writes it, see
//! `docs/plugin-export-format.md`. The writer here is independent from the Rust container writer on purpose.

use super::temp_folder;
use crate::features::import_export::models::NodeKind;
use crate::features::projects::logic::convert;
use crate::features::projects::services::open::open;
use sha2::{Digest, Sha256};
use std::io::Write;

const JSON: &str = r#"{"meta":{"name":"Casa SketchUp","created":1700000000,"modified":1700000000,"app_version":"0.1.0"},
"origin":{"format":"SketchUp","version":"SketchUp 2024","application":"SketchUp","file_name":"casa.skp","file_size":0,
"length_unit":"in","unit_scale":0.0254,"imported":1700000000},
"origin_shift":[10.5,-2.25,0.0],
"levels":[],
"materials":[{"name":"Madera","color":[0.5,0.25,0.125,1.0]},{"name":"Vidrio","color":[0.6,0.8,0.9,0.4]}],
"nodes":[{"name":"Casa SketchUp","class":"","kind":"project","type_name":null,"parent":null,"meshes":[],"level":null},
{"name":"Mesa","class":"Muebles","kind":"element","type_name":"Mesa 120","parent":0,"meshes":[0],"level":null},
{"name":"Ventana","class":"Layer0","kind":"element","type_name":null,"parent":1,"meshes":[1],"level":null}],
"meshes":[{"material":0,"vertices":4,"indices":6},{"material":1,"vertices":3,"indices":3}]}"#;

fn gzip(data: &[u8]) -> Vec<u8> {
    let mut gz = flate2::write::GzEncoder::new(Vec::new(), flate2::Compression::default());
    gz.write_all(data).unwrap();
    gz.finish().unwrap()
}

/// `[nv, ni].pack("VV") + positions.pack("e*") + indices.pack("V*")`.
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

fn plugin_bytes() -> Vec<u8> {
    let blocks = [
        mesh_block(
            &[0., 0., 0., 1., 0., 0., 1., 1., 0., 0., 1., 0.],
            &[0, 1, 2, 0, 2, 3],
        ),
        mesh_block(&[0., 0., 1., 0., 1., 1., 0., 1., 2.], &[0, 1, 2]),
    ];
    let json_gz = gzip(JSON.as_bytes());
    let mut body = Vec::new();
    let mut index = Vec::new();
    for (id, raw) in blocks.iter().enumerate() {
        let gz = gzip(raw);
        index.extend_from_slice(&1u32.to_le_bytes());
        index.extend_from_slice(&(id as u32).to_le_bytes());
        index.extend_from_slice(&((96 + json_gz.len() + body.len()) as u64).to_le_bytes());
        index.extend_from_slice(&(gz.len() as u64).to_le_bytes());
        index.extend_from_slice(&(raw.len() as u64).to_le_bytes());
        index.extend_from_slice(&Sha256::digest(raw));
        body.extend_from_slice(&gz);
    }
    let mut out = b"GRRPROJ1".to_vec();
    out.extend_from_slice(&1u16.to_le_bytes());
    out.extend_from_slice(&0u16.to_le_bytes());
    out.extend_from_slice(&1u32.to_le_bytes());
    out.extend_from_slice(&[0x5Au8; 16]);
    out.extend_from_slice(&(JSON.len() as u64).to_le_bytes());
    out.extend_from_slice(&(json_gz.len() as u64).to_le_bytes());
    out.extend_from_slice(&2u32.to_le_bytes());
    out.extend_from_slice(&0u32.to_le_bytes());
    out.extend_from_slice(&Sha256::digest(JSON.as_bytes()));
    out.extend_from_slice(&((96 + json_gz.len() + body.len()) as u64).to_le_bytes());
    out.extend_from_slice(&json_gz);
    out.extend_from_slice(&body);
    out.extend_from_slice(&index);
    out
}

#[test]
fn a_plugin_written_file_opens_with_hierarchy_materials_and_meshes() {
    let path = temp_folder("plugin").join("casa.gr");
    std::fs::write(&path, plugin_bytes()).unwrap();
    let opened = open(&path).expect("the plugin file opens");
    assert!(opened.report.is_none() && opened.warnings.is_empty());
    let project = opened.project;
    assert_eq!(project.id, [0x5A; 16]);
    assert_eq!(project.document.meta.name, "Casa SketchUp");
    assert_eq!(project.document.origin_shift, [10.5, -2.25, 0.0]);
    assert_eq!(project.meshes.len(), 2);
    assert_eq!(project.meshes[0].indices, vec![0, 1, 2, 0, 2, 3]);
    assert_eq!(project.meshes[0].material, Some(0));
    assert_eq!(project.meshes[1].positions[2], [0.0, 1.0, 2.0]);
    assert_eq!(project.meshes[1].material, Some(1));
    let scene = convert::to_scene(&project.document, &project.meshes, 0);
    assert_eq!(scene.materials[1].color, [0.6, 0.8, 0.9, 0.4]);
    assert_eq!(scene.nodes[0].kind, NodeKind::Project);
    assert_eq!(scene.nodes[1].type_name.as_deref(), Some("Mesa 120"));
    assert_eq!(scene.nodes[1].children, vec![2]);
    assert_eq!(scene.nodes[2].parent, Some(1));
    assert_eq!(scene.triangle_count(), 3);
}

#[test]
fn a_plugin_file_with_a_wrong_block_checksum_is_refused() {
    let path = temp_folder("plugin-bad").join("casa.gr");
    let mut bytes = plugin_bytes();
    let last = bytes.len() - 1;
    bytes[last] ^= 0xFF;
    std::fs::write(&path, bytes).unwrap();
    assert!(open(&path).is_err());
}
