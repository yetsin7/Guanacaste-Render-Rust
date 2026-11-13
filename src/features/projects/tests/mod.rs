//! Unit tests of the projects domain: repository filters, the `.gr` container, saving, recovery and the
//! read-only Guanacaste 3D path.

mod container_tests;
mod legacy_tests;
mod store_tests;

use super::logic;
use super::models::{ProjectFilter, ProjectsState};
use crate::services::sample_data::projects;

#[test]
fn all_filter_shows_every_project() {
    let list = projects::projects();
    let state = ProjectsState::default();
    assert_eq!(logic::visible(&list, &state).len(), list.len());
}

#[test]
fn queue_filter_matches_the_queue_count() {
    let list = projects::projects();
    let state = ProjectsState {
        filter: ProjectFilter::RenderQueue,
        ..Default::default()
    };
    assert_eq!(
        logic::visible(&list, &state).len(),
        logic::queued_count(&list)
    );
    assert_eq!(logic::queued_count(&list), 2);
}

#[test]
fn search_is_case_insensitive_and_checks_ids() {
    let list = projects::projects();
    let state = ProjectsState {
        search: "kyo-312".into(),
        ..Default::default()
    };
    let found = logic::visible(&list, &state);
    assert_eq!(found.len(), 1);
    assert_eq!(found[0].id, "#KYO-312");
}

#[test]
fn nothing_matches_gibberish() {
    let list = projects::projects();
    let state = ProjectsState {
        search: "zzzz".into(),
        ..Default::default()
    };
    assert!(logic::visible(&list, &state).is_empty());
}

/// A fresh empty folder under the system temporary directory, unique per call.
pub(super) fn temp_folder(label: &str) -> std::path::PathBuf {
    let id = crate::features::projects::services::util::hex(
        &crate::features::projects::services::util::new_document_id(),
    );
    let folder = std::env::temp_dir().join(format!("gr-test-{label}-{id}"));
    std::fs::create_dir_all(&folder).expect("temp folder");
    folder
}
