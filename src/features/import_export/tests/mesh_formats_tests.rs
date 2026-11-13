//! STL, PLY and 3DS readers on small files generated here.

use crate::features::import_export::models::{ImportError, ImportOptions};
use crate::features::import_export::services::{ply, progress::Progress, stl, tds};

fn z_up() -> ImportOptions {
    ImportOptions {
        y_up: false,
        unit_scale: 1.0,
    }
}

fn binary_stl(triangles: &[[[f32; 3]; 3]], declared: u32) -> Vec<u8> {
    let mut bytes = vec![0u8; 80];
    bytes.extend_from_slice(&declared.to_le_bytes());
    for triangle in triangles {
        bytes.extend_from_slice(&[0u8; 12]);
        for corner in triangle {
            for value in corner {
                bytes.extend_from_slice(&value.to_le_bytes());
            }
        }
        bytes.extend_from_slice(&[0u8; 2]);
    }
    bytes
}

const QUAD: [[[f32; 3]; 3]; 2] = [
    [[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [1.0, 1.0, 0.0]],
    [[0.0, 0.0, 0.0], [1.0, 1.0, 0.0], [0.0, 1.0, 2.0]],
];

#[test]
fn stl_binary_welds_vertices() {
    let bytes = binary_stl(&QUAD, 2);
    let scene = stl::import_bytes(&bytes, "placa", &z_up(), &Progress::default())
        .unwrap()
        .scene;
    assert_eq!(scene.triangle_count(), 2);
    assert_eq!(scene.vertex_count(), 4);
    assert_eq!(scene.nodes[1].name, "placa");
    assert_eq!(scene.bounds().unwrap().max, [1.0, 1.0, 2.0]);
}

#[test]
fn stl_binary_starting_with_solid_is_still_binary() {
    let mut bytes = binary_stl(&QUAD, 2);
    bytes[..5].copy_from_slice(b"solid");
    let outcome = stl::import_bytes(&bytes, "x", &z_up(), &Progress::default()).unwrap();
    assert_eq!(outcome.scene.triangle_count(), 2);
}

#[test]
fn stl_truncated_loads_what_is_there_and_warns() {
    let bytes = binary_stl(&QUAD[..1], 5);
    let outcome = stl::import_bytes(&bytes, "x", &z_up(), &Progress::default()).unwrap();
    assert_eq!(outcome.scene.triangle_count(), 1);
    assert!(outcome
        .report
        .entries
        .iter()
        .any(|e| e.key == "stl.truncated"));
}

#[test]
fn stl_ascii_with_two_solids_and_y_up() {
    let text = "solid uno\nfacet normal 0 0 1\nouter loop\nvertex 0 0 0\nvertex 1 0 0\nvertex 0 0 -1\nendloop\nendfacet\nendsolid uno\nsolid dos\nfacet normal 0 0 1\nouter loop\nvertex 0 0 0\nvertex 0 1 0\nvertex 1 1 0\nendloop\nendfacet\nendsolid dos\n";
    let options = ImportOptions::default();
    let scene = stl::import_bytes(text.as_bytes(), "x", &options, &Progress::default())
        .unwrap()
        .scene;
    assert_eq!(scene.element_count(), 2);
    assert_eq!(scene.nodes[1].name, "uno");
    // Y-up: the point (0, 0, -1) becomes Y = 1.
    assert!(scene.meshes[0].positions.iter().any(|p| p[1] == 1.0));
}

#[test]
fn stl_rejects_garbage_without_panicking() {
    for bytes in [
        vec![],
        vec![1, 2, 3],
        vec![0u8; 84],
        b"solid vacio\nendsolid\n".to_vec(),
    ] {
        let result = stl::import_bytes(&bytes, "x", &z_up(), &Progress::default());
        assert!(matches!(result, Err(ImportError::Invalid(_))), "{bytes:?}");
    }
}

#[test]
fn stl_skips_non_finite_triangles() {
    let mut triangles = QUAD.to_vec();
    triangles[1][0][0] = f32::NAN;
    let bytes = binary_stl(&triangles, 2);
    let outcome = stl::import_bytes(&bytes, "x", &z_up(), &Progress::default()).unwrap();
    assert_eq!(outcome.scene.triangle_count(), 1);
    assert!(outcome
        .report
        .entries
        .iter()
        .any(|e| e.key == "mesh.nonfinite"));
}

const PLY_BODY: &str = "0 0 0 255 0 0\n1 0 0 255 0 0\n1 1 0 255 0 0\n0 1 0 255 0 0\n4 0 1 2 3\n";

fn ascii_ply() -> String {
    format!("ply\nformat ascii 1.0\ncomment TextureFile pared.png\nelement vertex 4\nproperty float x\nproperty float y\nproperty float z\nproperty uchar red\nproperty uchar green\nproperty uchar blue\nelement face 1\nproperty list uchar int vertex_indices\nend_header\n{PLY_BODY}")
}

#[test]
fn ply_ascii_reads_quad_color_and_texture_name() {
    let text = ascii_ply();
    let outcome =
        ply::import_bytes(text.as_bytes(), "p", None, &z_up(), &Progress::default()).unwrap();
    assert_eq!(outcome.scene.triangle_count(), 2);
    let material = &outcome.scene.materials[0];
    assert_eq!(material.color, [1.0, 0.0, 0.0, 1.0]);
    assert!(material.texture.is_none(), "no UVs: no texture binding");
}

#[test]
fn ply_binary_big_endian_with_uvs() {
    let mut bytes = b"ply\nformat binary_big_endian 1.0\ncomment TextureFile t.png\nelement vertex 3\nproperty float x\nproperty float y\nproperty float z\nproperty float s\nproperty float t\nelement face 1\nproperty list uchar uint vertex_indices\nend_header\n".to_vec();
    for v in [
        [0.0f32, 0.0, 0.0, 0.0, 0.0],
        [2.0, 0.0, 0.0, 1.0, 0.0],
        [0.0, 2.0, 0.0, 0.0, 1.0],
    ] {
        for value in v {
            bytes.extend_from_slice(&value.to_be_bytes());
        }
    }
    bytes.push(3);
    for i in [0u32, 1, 2] {
        bytes.extend_from_slice(&i.to_be_bytes());
    }
    let outcome = ply::import_bytes(&bytes, "p", None, &z_up(), &Progress::default()).unwrap();
    let mesh = &outcome.scene.meshes[0];
    assert_eq!(mesh.uvs.len(), 3);
    assert_eq!(mesh.uvs[1], [1.0, 0.0]);
    assert_eq!(mesh.positions[2], [0.0, 2.0, 0.0]);
    assert_eq!(outcome.scene.materials[0].texture.as_deref(), Some("t.png"));
}

#[test]
fn ply_damaged_files_error_cleanly() {
    let text = ascii_ply();
    let cut = &text.as_bytes()[..text.len() - 30];
    assert!(ply::import_bytes(cut, "p", None, &z_up(), &Progress::default()).is_err());
    let huge =
        "ply\nformat ascii 1.0\nelement vertex 4000000000\nproperty float x\nend_header\n1 2\n";
    assert!(ply::import_bytes(huge.as_bytes(), "p", None, &z_up(), &Progress::default()).is_err());
    for junk in [
        &b""[..],
        b"ply\n",
        b"nope",
        b"ply\nformat ascii 1.0\nend_header\n",
    ] {
        assert!(ply::import_bytes(junk, "p", None, &z_up(), &Progress::default()).is_err());
    }
}

fn chunk(id: u16, body: &[u8]) -> Vec<u8> {
    let mut out = id.to_le_bytes().to_vec();
    out.extend_from_slice(&(body.len() as u32 + 6).to_le_bytes());
    out.extend_from_slice(body);
    out
}

fn tds_file() -> Vec<u8> {
    let mut verts = 4u16.to_le_bytes().to_vec();
    for p in [
        [0.0f32, 0.0, 0.0],
        [1.0, 0.0, 0.0],
        [1.0, 1.0, 0.0],
        [0.0, 1.0, 0.0],
    ] {
        p.iter()
            .for_each(|v| verts.extend_from_slice(&v.to_le_bytes()));
    }
    let mut group = b"Rojo\0".to_vec();
    group.extend_from_slice(&2u16.to_le_bytes());
    group.extend_from_slice(&[0, 0, 1, 0]);
    let mut faces = 2u16.to_le_bytes().to_vec();
    for f in [[0u16, 1, 2, 0], [0, 2, 3, 0]] {
        f.iter()
            .for_each(|v| faces.extend_from_slice(&v.to_le_bytes()));
    }
    faces.extend(chunk(0x4130, &group));
    let mut mesh = chunk(0x4110, &verts);
    mesh.extend(chunk(0x4120, &faces));
    let mut object = b"Piso\0".to_vec();
    object.extend(chunk(0x4100, &mesh));
    let mut color = Vec::new();
    [200u8, 20, 10].iter().for_each(|v| color.push(*v));
    let mut material = chunk(0xA000, b"Rojo\0");
    material.extend(chunk(0xA020, &chunk(0x0011, &color)));
    let mut editor = chunk(0xAFFF, &material);
    editor.extend(chunk(0x4000, &object));
    chunk(0x4D4D, &chunk(0x3D3D, &editor))
}

#[test]
fn tds_reads_objects_and_materials() {
    let bytes = tds_file();
    let scene = tds::import_bytes(&bytes, "m", None, &z_up(), &Progress::default())
        .unwrap()
        .scene;
    assert_eq!(scene.nodes[1].name, "Piso");
    assert_eq!(scene.triangle_count(), 2);
    assert_eq!(scene.materials[0].name, "Rojo");
    assert!((scene.materials[0].color[0] - 200.0 / 255.0).abs() < 1e-6);
    assert_eq!(
        scene
            .meshes
            .iter()
            .filter(|m| m.material == Some(0))
            .count(),
        1
    );
}

#[test]
fn tds_truncation_never_panics() {
    let bytes = tds_file();
    for cut in 0..bytes.len() {
        let _ = tds::import_bytes(&bytes[..cut], "m", None, &z_up(), &Progress::default());
    }
    let mut hostile = bytes.clone();
    hostile[2..6].copy_from_slice(&u32::MAX.to_le_bytes());
    let _ = tds::import_bytes(&hostile, "m", None, &z_up(), &Progress::default());
}
