//! Storing objects in the `.gr` document, backward compatibly.

use crate::features::build::models::{ObjectsDoc, ObjectsState};
use crate::features::library::models::{AssetRef, ProceduralKind};
use crate::features::projects::models::ProjectDocument;

#[test]
fn old_documents_without_objects_still_open() {
    let document: ProjectDocument =
        serde_json::from_str(r#"{"meta":{"name":"Antiguo"},"vegetation":[]}"#).unwrap();
    assert!(document.objects.is_empty());
}

#[test]
fn empty_objects_are_not_written() {
    let text = serde_json::to_string(&ProjectDocument::default()).unwrap();
    assert!(!text.contains("objects"));
}

#[test]
fn unknown_future_fields_inside_objects_are_ignored() {
    let doc: ObjectsDoc = serde_json::from_str(
        r#"{"instances":[{"id":4,"asset":{"type":"procedural","kind":"tree","seed":9},
        "transform":{"position":[1,2,3]},"future":1}],"extra":true}"#,
    )
    .unwrap();
    assert_eq!(doc.instances[0].transform.scale, [1.0; 3]);
    let state = ObjectsState::from_doc(doc);
    assert!(state.doc.next_id > 4);
}

#[test]
fn objects_round_trip_through_the_project_document() {
    let mut state = ObjectsState::default();
    state.browser.active = Some(AssetRef::Procedural {
        kind: ProceduralKind::Car,
        seed: 0,
    });
    state.place_active([1.5, -2.25, 0.3]);
    state.place_active([3.0, 4.0, 0.0]);
    state.select_all();
    state.edit_selection(|_, t| t.rotation[2] = 33.3);
    let document = ProjectDocument {
        objects: state.to_doc(),
        ..ProjectDocument::default()
    };
    let text = serde_json::to_string(&document).unwrap();
    let back: ProjectDocument = serde_json::from_str(&text).unwrap();
    assert_eq!(back, document);
    let mut reopened = ObjectsState::from_doc(back.objects);
    assert_eq!(reopened.instances(), state.instances());
    assert!(!reopened.history.can_undo());
    reopened.browser.active = state.browser.active;
    let fresh = reopened.place_active([0.0; 3]).unwrap();
    assert!(state.instance(fresh).is_none(), "ids are not reused");
}

#[test]
fn imported_models_are_reloaded_from_their_files() {
    let dir = std::env::temp_dir().join(format!("gr-objects-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let file = dir.join("mesa.obj");
    std::fs::write(
        &file,
        "v 0 0 5\nv 2 0 5\nv 2 2 5\nv 0 2 5\nv 0 0 6\nf 1 2 3 4\nf 1 2 5\n",
    )
    .unwrap();
    let mut state = ObjectsState::default();
    let id = state.import_model(&file).expect("imports");
    state.place_active([10.0, 0.0, 0.0]);
    let doc = state.to_doc();
    let mut reopened = ObjectsState::from_doc(doc.clone());
    assert!(reopened.missing.is_empty());
    let scene = reopened
        .cache
        .scene(AssetRef::Imported { id })
        .expect("geometry");
    let bounds = scene.bounds().unwrap();
    assert_eq!(bounds.min[2], 0.0, "rests on the ground");
    std::fs::remove_dir_all(&dir).unwrap();
    let gone = ObjectsState::from_doc(doc);
    assert_eq!(gone.missing, vec!["mesa".to_owned()]);
    assert_eq!(gone.instances().len(), 1);
}
