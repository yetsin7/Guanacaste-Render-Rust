//! The Guanacaste 3D reader against corrupt, truncated and hostile containers.

use crate::features::import_export::services::legacy::{build, read_container};
use sha2::{Digest, Sha256};
use std::io::Write;

fn container(json: &str) -> Vec<u8> {
    let mut gz = flate2::write::GzEncoder::new(Vec::new(), flate2::Compression::default());
    gz.write_all(json.as_bytes()).unwrap();
    let gz = gz.finish().unwrap();
    let mut out = b"G3DNATV1".to_vec();
    out.extend_from_slice(&1i32.to_le_bytes());
    out.push(1);
    out.extend_from_slice(&15i32.to_le_bytes());
    out.extend_from_slice(&[7u8; 16]);
    out.extend_from_slice(&(json.len() as i64).to_le_bytes());
    out.extend_from_slice(&(gz.len() as i64).to_le_bytes());
    out.extend_from_slice(&Sha256::digest(json.as_bytes()));
    out.extend_from_slice(&gz);
    out
}

const DOC: &str = r#"{"Name":"T","Levels":[{"Id":"L","Name":"N1","Elevation":0}],
"Walls":[{"LevelId":"L","Start":{"X":0,"Y":0},"End":{"X":4,"Y":0},"Thickness":0.2,"Height":3}],
"StructuralElements":[{"LevelId":"L","Start":{"X":0,"Y":0},"End":{"X":1,"Y":1},"Width":1,"Height":0}]}"#;

#[test]
fn every_truncation_is_an_error_not_a_panic() {
    let bytes = container(DOC);
    assert!(read_container(&bytes).is_ok());
    for cut in 0..bytes.len() {
        assert!(read_container(&bytes[..cut]).is_err(), "cut at {cut}");
    }
}

#[test]
fn flipped_bytes_and_hostile_lengths_are_rejected() {
    let bytes = container(DOC);
    for at in (0..bytes.len()).step_by(3) {
        let mut damaged = bytes.clone();
        damaged[at] ^= 0xFF;
        let _ = read_container(&damaged);
    }
    let mut huge = bytes.clone();
    huge[41..49].copy_from_slice(&i64::MAX.to_le_bytes());
    assert!(read_container(&huge).is_err());
    let mut negative = bytes.clone();
    negative[33..41].copy_from_slice(&(-5i64).to_le_bytes());
    assert!(read_container(&negative).is_err());
    let mut liar = bytes;
    liar[33..41].copy_from_slice(&(200i64 << 20).to_le_bytes());
    assert!(read_container(&liar).is_err());
}

#[test]
fn truncation_message_is_actionable() {
    let bytes = container(DOC);
    let error = read_container(&bytes[..bytes.len() - 4]).unwrap_err();
    assert!(error.detail().contains("truncado"), "{}", error.detail());
}

#[test]
fn odd_documents_build_without_panicking() {
    let weird = [
        "[]",
        "null",
        r#"{"Walls":"no"}"#,
        r#"{"Walls":[{}],"Floors":[{"Boundary":[{"X":1e308,"Y":-1e308},{"X":0,"Y":1e308},{"X":5,"Y":5}]}]}"#,
        r#"{"Columns":[{"Position":{"X":1,"Y":1},"Width":-1,"Depth":0,"Height":1e300}]}"#,
    ];
    for json in weird {
        let (_, document) = read_container(&container(json)).unwrap();
        let (scene, _) = build(&document, 15, 0);
        assert!(scene
            .meshes
            .iter()
            .flat_map(|m| &m.positions)
            .all(|p| p.iter().all(|v| v.is_finite())));
    }
}

#[test]
fn voids_without_height_are_skipped_not_failed() {
    let (_, document) = read_container(&container(DOC)).unwrap();
    let (scene, report) = build(&document, 15, 0);
    assert_eq!(scene.element_count(), 1);
    assert_eq!(report.elements_failed, 0);
    assert_eq!(report.skipped_types.values().sum::<usize>(), 1);
    assert_eq!(scene.levels.len(), 1);
    assert_eq!(scene.materials.len(), 1);
}
