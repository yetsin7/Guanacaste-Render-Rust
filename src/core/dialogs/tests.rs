//! Tests of the dialog host: menu, shortcuts, requests and the project flow.

use super::*;
use crate::features::projects::models::Project;
use crate::features::projects::services::{store, util};

fn temp_host(label: &str) -> (DialogHost, PathBuf) {
    let folder = std::env::temp_dir().join(format!(
        "gr-host-{label}-{}",
        util::hex(&util::new_document_id())
    ));
    std::fs::create_dir_all(&folder).unwrap();
    (DialogHost::with_data_dir(folder.join("datos")), folder)
}

fn frame(host: &mut DialogHost, ctx: &Context) {
    host.on_change(ctx);
    host.after(ctx);
}

#[test]
fn every_menu_entry_has_a_dialog() {
    let dialogs: Vec<Dialog> = MENU.iter().map(|entry| entry.3).collect();
    assert!(dialogs.contains(&Dialog::OpenProject));
    assert!(dialogs.contains(&Dialog::SaveAs));
    assert!(dialogs.contains(&Dialog::About));
}

#[test]
fn shortcuts_open_distinct_dialogs() {
    let mut dialogs: Vec<String> = SHORTCUTS
        .iter()
        .map(|entry| format!("{:?}", entry.1))
        .collect();
    dialogs.sort();
    dialogs.dedup();
    assert_eq!(dialogs.len(), SHORTCUTS.len());
}

#[test]
fn a_request_opens_its_dialog() {
    let ctx = Context::default();
    let (mut host, _) = temp_host("request");
    commands::request_dialog(&ctx, Dialog::About);
    host.intake(&ctx);
    assert_eq!(host.active(), Some(Dialog::About));
}

#[test]
fn closing_with_unsaved_changes_asks_first() {
    let ctx = Context::default();
    let (mut host, _) = temp_host("close");
    assert!(
        !host.intercept_close(&ctx),
        "nothing open: the window may close"
    );
    host.workspace.start_new("Casa");
    assert!(host.intercept_close(&ctx));
    assert_eq!(host.active(), Some(Dialog::UnsavedChanges));
    host.settle(&ctx, UnsavedChoiceAlias::Cancel);
    assert_eq!(host.active(), None);
    assert!(host.intercept_close(&ctx));
    host.settle(&ctx, UnsavedChoiceAlias::Discard);
    assert!(host.workspace.close_confirmed && !host.intercept_close(&ctx));
}

use crate::features::projects::components::unsaved_dialog::UnsavedChoice as UnsavedChoiceAlias;

#[test]
fn save_writes_in_place_once_the_project_has_its_own_file() {
    let ctx = Context::default();
    let (mut host, folder) = temp_host("save");
    host.workspace.start_new("Casa");
    host.active = Some(Dialog::SaveProject);
    frame(&mut host, &ctx);
    assert_eq!(
        host.active(),
        Some(Dialog::SaveProject),
        "first save asks for a name"
    );
    assert_eq!(host.save.name, "Casa.gr");
    host.save.request = Some(folder.join("Casa"));
    frame(&mut host, &ctx);
    assert_eq!(host.active(), None);
    let saved = folder.join("Casa.gr");
    assert_eq!(&std::fs::read(&saved).unwrap()[..8], b"GRRPROJ1");
    assert!(!host.workspace.dirty());
    assert_eq!(host.workspace.recent[0].entry.path, saved);
    host.workspace.project.as_mut().unwrap().dirty = true;
    host.active = Some(Dialog::SaveProject);
    frame(&mut host, &ctx);
    assert_eq!(
        host.active(),
        None,
        "Ctrl+S writes straight to the same file"
    );
    assert!(!host.workspace.dirty() && saved.with_file_name("Casa.gr.bak").exists());
}

#[test]
fn save_on_a_guanacaste_3d_project_becomes_save_as_with_a_reason() {
    let ctx = Context::default();
    let (mut host, folder) = temp_host("legacy");
    let original = folder.join("Casa.gr");
    let mut project: Project = store::new_project("Casa");
    project.legacy_source = Some(original.clone());
    host.workspace.project = Some(project);
    host.active = Some(Dialog::SaveProject);
    frame(&mut host, &ctx);
    assert_eq!(host.active(), Some(Dialog::SaveProject));
    assert_eq!(host.save.target(), folder.join("Casa - render.gr"));
    assert_eq!(host.save.reason_key, Some("save.legacy_reason"));
}
