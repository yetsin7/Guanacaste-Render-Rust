//! Guanacaste 3D projects (`G3DNATV1`): read-only import, detection by signature and the rule that they are
//! never overwritten. The fixture is synthetic, written with the layout documented by Guanacaste 3D.

use super::temp_folder;
use crate::features::import_export::models::{ImportError, ImportOptions, Severity};
use crate::features::import_export::services::{job, legacy, progress::Progress};
use crate::features::projects::logic::save_plan::{save_target, SaveTarget};
use crate::features::projects::services::open::{open, OpenError};
use crate::features::projects::services::store::{self, StoreError};
use sha2::{Digest, Sha256};
use std::io::Write;
use std::path::Path;

const HOUSE: &str = r##"{"SchemaVersion":15,"Id":"6f1c","Name":"Casa demo",
"Levels":[{"Id":"L2","Name":"Nivel 2","Elevation":3.0},{"Id":"L1","Name":"Nivel 1","Elevation":0.0}],
"WallTypes":[{"Id":"T1","Name":"Basic 200 mm","Function":0,"Thickness":0.2,"MaterialName":"Concrete masonry","SectionFillColor":"#87928B"}],
"Walls":[{"Id":"W1","LevelId":"L1","Start":{"X":0,"Y":0},"End":{"X":6,"Y":0},"Thickness":0.2,"Height":2.7,"WallTypeId":"T1","TopLevelId":"L2","TopOffset":0.0},
{"Id":"W2","LevelId":"L1","Start":{"X":6,"Y":0},"End":{"X":6,"Y":4},"Thickness":0.2,"Height":2.7,"WallTypeId":"T1"},
{"Id":"W3","LevelId":"L1","Start":{"X":1,"Y":1},"End":{"X":1,"Y":1},"Thickness":0.2,"Height":2.7}],
"Openings":[{"Id":"O1","WallId":"W1","Kind":0,"Offset":1,"Width":0.9,"Sill":0,"Height":2.1}],
"Rooms":[{"Id":"R1","LevelId":"L1","Number":"1","Name":"Sala","FirstCorner":{"X":0,"Y":0},"SecondCorner":{"X":6,"Y":4}}],
"Floors":[{"Id":"F1","LevelId":"L1","Name":"Losa","Boundary":[{"X":0,"Y":0},{"X":6,"Y":0},{"X":6,"Y":4},{"X":0,"Y":4}],"Thickness":0.15,"Kind":0,"ElevationOffset":0}],
"Columns":[{"Id":"C1","LevelId":"L1","Position":{"X":3,"Y":2},"Width":0.3,"Depth":0.3,"Height":2.7}],
"StructuralElements":[{"Id":"S1","LevelId":"L2","Kind":0,"Name":"Viga","Start":{"X":0,"Y":2},"End":{"X":6,"Y":2},"Width":0.2,"Height":0.4,"Offset":-0.4}]}"##;

/// Encodes `json` as a Guanacaste 3D container of document `kind`.
fn g3d_bytes(json: &str, kind: u8) -> Vec<u8> {
    let mut gz = flate2::write::GzEncoder::new(Vec::new(), flate2::Compression::default());
    gz.write_all(json.as_bytes()).unwrap();
    let gz = gz.finish().unwrap();
    let mut out = b"G3DNATV1".to_vec();
    out.extend_from_slice(&1i32.to_le_bytes());
    out.push(kind);
    out.extend_from_slice(&15i32.to_le_bytes());
    out.extend_from_slice(&[9u8; 16]);
    out.extend_from_slice(&(json.len() as i64).to_le_bytes());
    out.extend_from_slice(&(gz.len() as i64).to_le_bytes());
    out.extend_from_slice(&Sha256::digest(json.as_bytes()));
    out.extend_from_slice(&gz);
    out
}

fn write_house(path: &Path) -> Vec<u8> {
    let bytes = g3d_bytes(HOUSE, 1);
    std::fs::write(path, &bytes).unwrap();
    bytes
}

#[test]
fn a_guanacaste_3d_project_becomes_a_scene() {
    let path = temp_folder("g3d").join("casa.gr");
    write_house(&path);
    let outcome = legacy::import(&path, &Progress::default()).expect("import");
    let scene = &outcome.scene;
    assert_eq!(
        scene
            .levels
            .iter()
            .map(|l| l.name.as_str())
            .collect::<Vec<_>>(),
        ["Nivel 1", "Nivel 2"]
    );
    assert_eq!(outcome.report.elements_total, 6);
    assert_eq!(
        outcome.report.elements_with_geometry, 5,
        "the zero-length wall is skipped"
    );
    let wall = scene.nodes.iter().find(|n| n.class == "Wall").unwrap();
    let top = scene.meshes[wall.meshes[0]]
        .positions
        .iter()
        .map(|p| p[2])
        .fold(f32::MIN, f32::max);
    assert!(
        (top - 3.0).abs() < 1e-5,
        "the wall reaches the level above, got {top}"
    );
    let beam = scene.nodes.iter().find(|n| n.name == "Viga").unwrap();
    assert_eq!(beam.level, Some(1));
    let keys: Vec<_> = outcome
        .report
        .entries
        .iter()
        .map(|e| (e.severity, e.key))
        .collect();
    assert!(keys.contains(&(Severity::Warning, "legacy.openings_not_cut")));
    assert!(keys.contains(&(Severity::Warning, "legacy.degenerate")));
    assert_eq!(outcome.report.skipped_types.get("Rooms"), Some(&1));
}

#[test]
fn damaged_or_non_project_containers_are_refused() {
    let mut bytes = g3d_bytes(HOUSE, 1);
    let last = bytes.len() - 20;
    bytes[last] ^= 0x55;
    assert!(legacy::read_container(&bytes).is_err());
    assert!(
        legacy::read_container(&g3d_bytes(HOUSE, 2)).is_err(),
        "a family is not a project"
    );
    assert!(legacy::read_container(&g3d_bytes(HOUSE, 1)).is_ok());
}

#[test]
fn the_signature_decides_never_the_extension() {
    let folder = temp_folder("signature");
    let legacy_as_g3d = folder.join("viejo.g3d");
    write_house(&legacy_as_g3d);
    assert!(open(&legacy_as_g3d)
        .expect("legacy by signature")
        .project
        .legacy_source
        .is_some());
    let unknown = folder.join("raro.gr");
    std::fs::write(&unknown, b"PK\x03\x04 no es un proyecto").unwrap();
    assert_eq!(open(&unknown).err(), Some(OpenError::UnknownSignature));
    let render = folder.join("salida.gr");
    let project = open(&legacy_as_g3d).unwrap().project;
    store::save(&render, &project, None).expect("save");
    let reopened = open(&render).expect("render by signature");
    assert!(reopened.project.legacy_source.is_none() && reopened.report.is_none());
    let imported = job::run(&render, &ImportOptions::default(), &Progress::default());
    assert!(
        matches!(imported, Err(ImportError::Invalid(_))),
        "the importer refuses Guanacaste Render projects"
    );
}

#[test]
fn a_guanacaste_3d_file_is_never_overwritten() {
    let folder = temp_folder("protect");
    let original = folder.join("Casa.gr");
    let bytes = write_house(&original);
    let opened = open(&original).expect("open");
    assert!(opened.project.path.is_none());
    let SaveTarget::AskAs {
        suggested,
        reason_key,
    } = save_target(&opened.project)
    else {
        panic!("Save must behave as Save as for a Guanacaste 3D project");
    };
    assert_eq!(suggested, folder.join("Casa - render.gr"));
    assert_eq!(reason_key, Some("save.legacy_reason"));
    assert_eq!(
        store::save(&original, &opened.project, None),
        Err(StoreError::LegacyTarget)
    );
    assert_eq!(
        std::fs::read(&original).unwrap(),
        bytes,
        "the Guanacaste 3D file is byte-for-byte intact"
    );
    assert!(!folder.join("Casa.gr.bak").exists() && !folder.join("Casa.gr.tmp").exists());
    store::save(&suggested, &opened.project, None).expect("save as");
    assert_eq!(&std::fs::read(&suggested).unwrap()[..8], b"GRRPROJ1");
    assert_eq!(std::fs::read(&original).unwrap(), bytes);
}
