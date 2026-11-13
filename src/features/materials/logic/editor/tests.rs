//! Editor actions: list management, renaming, deleting and texture assignment.

use super::*;
use crate::features::materials::models::{MaterialLibrary, ProceduralKind};

fn state(names: &[&str]) -> MaterialState {
    MaterialState {
        library: MaterialLibrary::from_materials(names.iter().map(|n| Material::new(*n)).collect()),
        ..MaterialState::default()
    }
}

#[test]
fn the_session_library_starts_with_the_presets() {
    let state = MaterialState::default();
    assert!(state.library.len() >= 20);
    assert!(state.current().is_some());
    assert!(!state.changed);
}

#[test]
fn the_filter_hides_non_matching_names() {
    let mut s = state(&["Concreto", "Vidrio claro", "Vidrio tintado"]);
    s.filter = "vidrio".into();
    assert_eq!(visible(&s), [1, 2]);
    s.filter = "  ".into();
    assert_eq!(visible(&s), [0, 1, 2]);
    s.filter = "zzz".into();
    assert!(visible(&s).is_empty());
}

#[test]
fn select_ignores_bad_indices_and_cancels_rename() {
    let mut s = state(&["A", "B"]);
    begin_rename(&mut s);
    select(&mut s, 7);
    assert_eq!(s.selected, 0);
    assert!(s.renaming.is_some());
    select(&mut s, 1);
    assert_eq!((s.selected, s.renaming.clone()), (1, None));
}

#[test]
fn adding_selects_the_new_material_and_flags_a_change() {
    let mut s = state(&["A"]);
    add_new(&mut s, "Material nuevo");
    assert_eq!((s.library.len(), s.selected), (2, 1));
    assert!(s.changed);
    s.changed = false;
    add_preset(&mut s, 0);
    assert_eq!(s.library.len(), 3);
    assert_eq!(s.current().unwrap().name, "Concreto visto");
    assert!(s.changed);
    add_preset(&mut s, 999);
    assert_eq!(s.library.len(), 3, "an unknown preset adds nothing");
}

#[test]
fn adding_the_same_preset_twice_keeps_names_unique() {
    let mut s = state(&[]);
    add_preset(&mut s, 0);
    add_preset(&mut s, 0);
    assert_ne!(
        s.library.get(0).unwrap().name,
        s.library.get(1).unwrap().name
    );
}

#[test]
fn duplicate_copies_values_and_selects_the_copy() {
    let mut s = state(&["A", "B"]);
    s.current_mut().unwrap().metallic = 0.7;
    duplicate_selected(&mut s);
    assert_eq!(s.selected, 2);
    assert_eq!(s.current().unwrap().metallic, 0.7);
    assert_eq!(s.current().unwrap().name, "A (copia)");
    assert!(s.changed);
}

#[test]
fn rename_commits_a_unique_name_and_ignores_blank_input() {
    let mut s = state(&["A", "B"]);
    select(&mut s, 1);
    begin_rename(&mut s);
    assert_eq!(s.renaming.as_deref(), Some("B"));
    s.renaming = Some("A".into());
    commit_rename(&mut s);
    assert_eq!(s.current().unwrap().name, "A 2");
    assert!(s.renaming.is_none() && s.changed);
    s.changed = false;
    begin_rename(&mut s);
    s.renaming = Some("   ".into());
    commit_rename(&mut s);
    assert_eq!(s.current().unwrap().name, "A 2");
    assert!(!s.changed);
}

#[test]
fn delete_removes_records_the_index_and_moves_the_selection() {
    let mut s = state(&["A", "B", "C"]);
    select(&mut s, 1);
    delete_selected(&mut s);
    assert_eq!(s.removed, Some(1));
    assert_eq!(s.library.len(), 2);
    assert_eq!(s.selected, 0);
    assert_eq!(s.current().unwrap().name, "A");
    delete_selected(&mut s);
    delete_selected(&mut s);
    assert!(s.library.is_empty() && s.current().is_none());
    assert_eq!(s.selected, 0);
    delete_selected(&mut s); // nothing left: harmless
}

#[test]
fn assigning_a_file_sets_the_slot_and_refreshes_the_cache() {
    let mut s = state(&["A"]);
    let mut cache = TextureCache::default();
    let path = std::env::temp_dir().join("gr-editor-nonexistent.png");
    // Remember a failure, then assign: the cache must forget it so the file is retried.
    let _ = cache.get_file(&path, crate::services::texture_cache::Usage::Color);
    assign_file(&mut s, &mut cache, MapKind::Roughness, &path);
    let slot = s.current().unwrap().maps.roughness.clone().unwrap();
    assert_eq!(slot, TextureRef::file(path.to_string_lossy().into_owned()));
    assert!(s.changed);
}

#[test]
fn an_emission_map_turns_black_emission_white() {
    let mut s = state(&["Lámpara"]);
    assign_procedural(
        &mut s,
        MapKind::Emission,
        Procedural::new(ProceduralKind::Noise),
    );
    assert_eq!(s.current().unwrap().emission, [1.0; 3]);
    s.current_mut().unwrap().emission = [0.2, 0.3, 0.4];
    assign_procedural(
        &mut s,
        MapKind::Emission,
        Procedural::new(ProceduralKind::Brick),
    );
    assert_eq!(s.current().unwrap().emission, [0.2, 0.3, 0.4]);
}

#[test]
fn clearing_a_map_also_clears_its_error() {
    let mut s = state(&["A"]);
    assign_procedural(
        &mut s,
        MapKind::Albedo,
        Procedural::new(ProceduralKind::Wood),
    );
    s.failures.push((
        MapKind::Albedo,
        crate::services::texture_cache::LoadError::NotFound,
    ));
    s.changed = false;
    clear_map(&mut s, MapKind::Albedo);
    assert!(s.current().unwrap().maps.albedo.is_none());
    assert!(s.failures.is_empty() && s.changed);
}

#[test]
fn reset_keeps_the_name_and_drops_everything_else() {
    let mut s = state(&["Mi muro"]);
    s.current_mut().unwrap().metallic = 1.0;
    assign_procedural(
        &mut s,
        MapKind::Albedo,
        Procedural::new(ProceduralKind::Brick),
    );
    reset_selected(&mut s);
    let m = s.current().unwrap();
    assert_eq!(m.name, "Mi muro");
    assert_eq!(*m, Material::new("Mi muro"));
}

#[test]
fn any_edit_cancels_a_pending_delete_confirmation() {
    let mut s = state(&["A"]);
    s.confirm_delete = true;
    add_new(&mut s, "B");
    assert!(!s.confirm_delete);
}
