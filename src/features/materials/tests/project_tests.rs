//! Materials inside `.gr` projects: backward compatible documents, exact round trips and editor synchronization.

use crate::features::import_export::models::Mesh;
use crate::features::materials::logic::project_io::{
    docs_from_library, library_from_docs, remap_after_removal, store_library,
};
use crate::features::materials::logic::sync;
use crate::features::materials::models::{
    MapKind, Material, MaterialLibrary, MaterialState, Procedural, ProceduralKind, TextureRef, Wrap,
};
use crate::features::projects::logic::convert;
use crate::features::projects::models::{MaterialDoc, Project, ProjectDocument};
use crate::features::projects::services::container::Limits;
use crate::features::projects::services::store;
use crate::features::projects::services::util;

fn rich_library() -> MaterialLibrary {
    let mut wall = Material::new("Muro de ladrillo");
    wall.base_color = [0.123_456_79, 0.456_789_1, 0.987_654_3, 1.0];
    wall.roughness = 0.333_333_33;
    wall.ior = 1.457_891;
    wall.uv.scale = [2.5, 0.75];
    wall.uv.rotation = 12.345;
    wall.uv.wrap = Wrap::Mirror;
    wall.maps.albedo = Some(TextureRef::procedural(Procedural::new(
        ProceduralKind::Brick,
    )));
    let mut bump = TextureRef::file(r"C:\texturas\muro_relieve.png");
    bump.is_height = true;
    wall.maps.normal = Some(bump);
    let mut glass = Material::new("Vidrio");
    glass.transparency = 0.9;
    glass.ior = 1.52;
    let mut lamp = Material::new("Lámpara");
    lamp.emission = [1.0, 0.8, 0.5];
    lamp.emission_strength = 7.5;
    MaterialLibrary::from_materials(vec![wall, glass, lamp])
}

fn project_with(library: &MaterialLibrary) -> Project {
    let mut project = store::new_project("Casa");
    store_library(&mut project, library);
    project
}

fn temp_file(label: &str) -> std::path::PathBuf {
    let id = util::hex(&util::new_document_id());
    let folder = std::env::temp_dir().join(format!("gr-mat-{label}-{id}"));
    std::fs::create_dir_all(&folder).expect("temp folder");
    folder.join("proyecto.gr")
}

#[test]
fn docs_round_trip_through_json_without_losing_a_bit() {
    let library = rich_library();
    let json = serde_json::to_string(&docs_from_library(&library)).expect("serialize");
    let docs: Vec<MaterialDoc> = serde_json::from_str(&json).expect("deserialize");
    assert_eq!(library_from_docs(&docs), library);
}

#[test]
fn a_saved_gr_file_returns_the_same_materials() {
    let library = rich_library();
    let project = project_with(&library);
    let path = temp_file("roundtrip");
    store::save(&path, &project, None).expect("save");
    let loaded = store::load(&path, Limits::default()).expect("load");
    assert!(loaded.warnings.is_empty());
    assert_eq!(
        library_from_docs(&loaded.project.document.materials),
        library
    );
    assert_eq!(
        loaded.project.document.materials,
        project.document.materials
    );
}

#[test]
fn the_portable_summary_still_carries_name_and_color() {
    let docs = docs_from_library(&rich_library());
    let value = serde_json::to_value(&docs).expect("json");
    assert_eq!(value[0]["name"], "Muro de ladrillo");
    assert_eq!(value[0]["color"].as_array().map(Vec::len), Some(4));
    // Transparency is folded into the summary alpha so a reader that only knows colors still sees glass.
    assert!(docs[1].color[3] < 0.2);
    assert!(value[0].get("pbr").is_some());
}

#[test]
fn documents_written_before_pbr_existed_still_load() {
    let old = r#"{"meta":{"name":"Antiguo"},"materials":[{"name":"Muro","color":[0.8,0.2,0.1,1.0]},
        {"name":"Vidrio","color":[0.5,0.7,0.9,0.25]}]}"#;
    let document: ProjectDocument = serde_json::from_str(old).expect("old documents parse");
    assert!(document.materials.iter().all(|m| m.pbr.is_none()));
    let library = library_from_docs(&document.materials);
    assert_eq!(library.len(), 2);
    assert_eq!(library.get(0).unwrap().base_color, [0.8, 0.2, 0.1, 1.0]);
    assert!(library.get(1).unwrap().is_transparent());
    // Saving it again upgrades the entries without changing what they look like.
    let upgraded = docs_from_library(&library);
    assert_eq!(upgraded[0].color, [0.8, 0.2, 0.1, 1.0]);
    assert!(upgraded[0].pbr.is_some());
}

#[test]
fn a_pbr_entry_with_unknown_future_fields_still_loads() {
    let json =
        r#"[{"name":"X","color":[1,1,1,1],"pbr":{"name":"ignorado","roughness":0.3,"sheen":0.9}}]"#;
    let docs: Vec<MaterialDoc> = serde_json::from_str(json).expect("parse");
    let library = library_from_docs(&docs);
    assert_eq!(
        library.get(0).unwrap().name,
        "X",
        "the document's name wins"
    );
    assert_eq!(library.get(0).unwrap().roughness, 0.3);
}

#[test]
fn damaged_values_in_a_document_are_clamped_on_load() {
    let json =
        r#"[{"name":"X","color":[1,1,1,1],"pbr":{"roughness":7.0,"ior":-3.0,"metallic":1e30}}]"#;
    let docs: Vec<MaterialDoc> = serde_json::from_str(json).expect("parse");
    let m = library_from_docs(&docs).get(0).cloned().unwrap();
    assert_eq!((m.roughness, m.ior, m.metallic), (1.0, 1.0, 1.0));
}

#[test]
fn imported_scene_materials_become_pbr_and_come_back_unchanged() {
    use crate::features::import_export::tests::{fixtures::IFC2X3_MM, import_ifc};
    let outcome = import_ifc(IFC2X3_MM);
    let (document, _) = convert::from_scene(&outcome.scene, "Casa", "casa.ifc", 1);
    let library = library_from_docs(&document.materials);
    assert_eq!(library.len(), outcome.scene.materials.len());
    let rebuilt = docs_from_library(&library);
    for (doc, original) in rebuilt.iter().zip(&outcome.scene.materials) {
        assert_eq!(doc.name, original.name);
        assert_eq!(doc.color, original.color);
    }
}

#[test]
fn removing_a_material_remaps_the_meshes() {
    let mut project = project_with(&rich_library());
    project.meshes = [Some(0), Some(1), Some(2), None]
        .into_iter()
        .map(|material| Mesh {
            material,
            ..Mesh::default()
        })
        .collect();
    project.document.meshes = project
        .meshes
        .iter()
        .map(|m| crate::features::projects::models::MeshMeta {
            material: m.material.map(|i| i as u32),
            vertices: 0,
            indices: 0,
        })
        .collect();
    project.dirty = false;
    remap_after_removal(&mut project, 1);
    let used: Vec<_> = project.meshes.iter().map(|m| m.material).collect();
    assert_eq!(used, [Some(0), None, Some(1), None]);
    let meta: Vec<_> = project.document.meshes.iter().map(|m| m.material).collect();
    assert_eq!(meta, [Some(0), None, Some(1), None]);
    assert!(project.dirty);
}

#[test]
fn storing_an_unchanged_library_does_not_dirty_the_project() {
    let library = rich_library();
    let mut project = project_with(&library);
    project.dirty = false;
    assert!(!store_library(&mut project, &library));
    assert!(!project.dirty);
    let mut edited = library.clone();
    edited.get_mut(0).unwrap().roughness = 0.9;
    assert!(store_library(&mut project, &edited));
    assert!(project.dirty);
}

#[test]
fn the_editor_adopts_a_project_once_and_writes_edits_back() {
    let mut project = project_with(&rich_library());
    project.dirty = false;
    let mut state = MaterialState::default();
    assert!(sync::adopt(&mut state, Some(&project)));
    assert_eq!(state.library, rich_library());
    assert!(
        !sync::adopt(&mut state, Some(&project)),
        "same project: keep the edits"
    );

    state.current_mut().unwrap().metallic = 1.0;
    state.changed = true;
    sync::flush(&mut state, Some(&mut project));
    assert!(project.dirty && !state.changed);
    assert_eq!(
        library_from_docs(&project.document.materials)
            .get(0)
            .unwrap()
            .metallic,
        1.0
    );
}

#[test]
fn deleting_in_the_editor_fixes_the_project_meshes() {
    let mut project = project_with(&rich_library());
    project.meshes = vec![Mesh {
        material: Some(2),
        ..Mesh::default()
    }];
    let mut state = MaterialState::default();
    sync::adopt(&mut state, Some(&project));
    crate::features::materials::logic::editor::select(&mut state, 0);
    crate::features::materials::logic::editor::delete_selected(&mut state);
    sync::flush(&mut state, Some(&mut project));
    assert_eq!(project.document.materials.len(), 2);
    assert_eq!(project.meshes[0].material, Some(1));
}

#[test]
fn read_only_projects_are_never_modified() {
    let mut project = project_with(&rich_library());
    project.read_only = true;
    project.dirty = false;
    let before = project.document.materials.clone();
    let mut state = MaterialState::default();
    sync::adopt(&mut state, Some(&project));
    state.current_mut().unwrap().roughness = 0.01;
    state.changed = true;
    sync::flush(&mut state, Some(&mut project));
    assert_eq!(project.document.materials, before);
    assert!(!project.dirty);
}

#[test]
fn closing_the_project_returns_to_the_session_presets() {
    let project = project_with(&rich_library());
    let mut state = MaterialState::default();
    sync::adopt(&mut state, Some(&project));
    assert_eq!(state.library.len(), 3);
    assert!(sync::adopt(&mut state, None));
    assert!(state.library.len() > 20 && state.project_id.is_none());
    assert!(!sync::adopt(&mut state, None));
}

#[test]
fn texture_references_survive_with_their_flags() {
    let library = rich_library();
    let docs = docs_from_library(&library);
    let back = library_from_docs(&docs);
    let normal = back.get(0).unwrap().maps.get(MapKind::Normal).unwrap();
    assert!(normal.is_height);
    assert_eq!(normal.source.file_name(), Some("muro_relieve.png"));
}
