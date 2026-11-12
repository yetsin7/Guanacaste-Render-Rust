//! Sanity checks of the placeholder data that feeds the screens.

use guanacaste_render_rust::services::sample_data::projects;
use std::collections::HashSet;

#[test]
fn projects_have_unique_ids_and_positive_frame_rates() {
    let list = projects::projects();
    assert!(!list.is_empty());
    let ids: HashSet<_> = list.iter().map(|project| project.id).collect();
    assert_eq!(ids.len(), list.len());
    assert!(list
        .iter()
        .all(|project| project.fps > 0 && !project.title.is_empty()));
}

#[test]
fn active_project_has_a_frame_time_series() {
    assert!(projects::active_project().frametime_series.len() >= 2);
}
