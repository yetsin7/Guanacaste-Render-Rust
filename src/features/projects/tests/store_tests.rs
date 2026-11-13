//! Saving and loading: IFC round trip, backups, newer schemas, recovery copies and recent files.

use super::temp_folder;
use crate::features::import_export::models::ImportedScene;
use crate::features::import_export::tests::fixtures::IFC2X3_MM;
use crate::features::projects::logic::convert;
use crate::features::projects::models::Project;
use crate::features::projects::services::container::{self, BlockSource, Limits};
use crate::features::projects::services::store::{self, LoadWarning, StoreError};
use crate::features::projects::services::{recent, recovery, util};

fn ifc_project() -> (ImportedScene, Project) {
    let outcome = crate::features::import_export::tests::import_ifc(IFC2X3_MM);
    let (document, meshes) =
        convert::from_scene(&outcome.scene, "Casa IFC", "casa.ifc", 1_700_000_000);
    let project = Project {
        id: util::new_document_id(),
        document,
        meshes,
        path: None,
        legacy_source: None,
        read_only: false,
        dirty: true,
    };
    (outcome.scene, project)
}

fn normalized(mut scene: ImportedScene) -> ImportedScene {
    for node in &mut scene.nodes {
        node.children.sort_unstable();
    }
    scene
}

#[test]
fn an_ifc_import_survives_save_and_load_unchanged() {
    let (scene, project) = ifc_project();
    let path = temp_folder("roundtrip").join("casa.gr");
    let stats = store::save(&path, &project, Some(b"\x89PNG fake")).expect("save");
    assert!(stats.bytes > 0 && !stats.backed_up);
    let loaded = store::load(&path, Limits::default()).expect("load");
    assert!(loaded.warnings.is_empty() && !loaded.project.read_only);
    assert_eq!(loaded.project.id, project.id);
    assert_eq!(loaded.project.document, project.document);
    assert_eq!(loaded.project.meshes, project.meshes);
    assert_eq!(loaded.thumbnail.as_deref(), Some(&b"\x89PNG fake"[..]));
    let rebuilt = convert::to_scene(&loaded.project.document, &loaded.project.meshes, 0);
    assert_eq!(normalized(rebuilt), normalized(scene));
    assert_eq!(store::peek(&path).expect("peek").name, "Casa IFC");
}

#[test]
fn saving_again_keeps_the_previous_version_as_bak() {
    let (_, mut project) = ifc_project();
    let path = temp_folder("bak").join("casa.gr");
    store::save(&path, &project, None).expect("first save");
    let first = std::fs::read(&path).unwrap();
    project.document.meta.name = "Casa v2".into();
    assert!(
        store::save(&path, &project, None)
            .expect("second save")
            .backed_up
    );
    assert_eq!(
        std::fs::read(path.with_file_name("casa.gr.bak")).unwrap(),
        first
    );
    assert_eq!(store::peek(&path).unwrap().name, "Casa v2");
    assert!(!path.with_file_name("casa.gr.tmp").exists());
}

#[test]
fn a_failed_save_leaves_no_partial_file() {
    let (_, project) = ifc_project();
    let path = temp_folder("fail").join("no-existe").join("casa.gr");
    assert!(store::save(&path, &project, None).is_err());
    assert!(!path.exists());
}

#[test]
fn a_newer_schema_opens_read_only_and_refuses_to_save() {
    let path = temp_folder("newer").join("futuro.gr");
    let mut file = std::fs::File::create(&path).unwrap();
    let json = br#"{"meta":{"name":"Futuro"},"campo_nuevo":true}"#;
    container::write(
        &mut file,
        99,
        [1u8; 16],
        json,
        &mut std::iter::empty::<BlockSource>(),
    )
    .unwrap();
    drop(file);
    let loaded = store::load(&path, Limits::default()).expect("load");
    assert!(loaded.project.read_only);
    assert_eq!(
        loaded.warnings,
        vec![LoadWarning::NewerSchema {
            found: 99,
            supported: 1
        }]
    );
    let before = std::fs::read(&path).unwrap();
    assert_eq!(
        store::save(&path, &loaded.project, None),
        Err(StoreError::ReadOnly)
    );
    assert_eq!(std::fs::read(&path).unwrap(), before);
}

#[test]
fn recovery_copies_are_listed_and_discarded() {
    let data = temp_folder("recovery");
    let (_, project) = ifc_project();
    recovery::write(&data, &project).expect("autosave");
    let found = recovery::list(&data);
    assert_eq!(found.len(), 1);
    assert_eq!(found[0].name, "Casa IFC");
    assert!(found[0].path.extension().is_some_and(|e| e == "gr"));
    recovery::discard(&data, &project.id);
    assert!(recovery::list(&data).is_empty());
}

#[test]
fn recent_files_move_to_the_front_and_vanished_ones_are_pruned() {
    let data = temp_folder("recent");
    let (a, b) = (data.join("a.gr"), data.join("b.gr"));
    std::fs::write(&a, b"x").unwrap();
    std::fs::write(&b, b"yy").unwrap();
    recent::remember(
        &data,
        recent::RecentEntry {
            path: a.clone(),
            name: "A".into(),
        },
    )
    .unwrap();
    recent::remember(
        &data,
        recent::RecentEntry {
            path: b.clone(),
            name: "B".into(),
        },
    )
    .unwrap();
    let list = recent::remember(
        &data,
        recent::RecentEntry {
            path: a.clone(),
            name: "A".into(),
        },
    )
    .unwrap();
    assert_eq!(
        list.iter().map(|e| e.name.as_str()).collect::<Vec<_>>(),
        ["A", "B"]
    );
    assert_eq!(recent::describe(&list)[1].size, 2);
    std::fs::remove_file(&b).unwrap();
    assert_eq!(recent::prune(&data).len(), 1);
}
