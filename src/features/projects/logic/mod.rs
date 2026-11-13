//! Rules of the repository and of the project document: filtering, conversion and schema migrations.

pub mod convert;
pub mod describe;
pub mod migrations;
pub mod save_plan;

use super::models::{ProjectFilter, ProjectSummary, ProjectsState};

/// Whether `project` passes the selected filter tab.
pub fn matches_filter(project: &ProjectSummary, filter: ProjectFilter) -> bool {
    match filter {
        ProjectFilter::All => true,
        ProjectFilter::RenderQueue => project.queued,
        ProjectFilter::CloudSynced => project.cloud,
        ProjectFilter::Archived => project.archived,
    }
}

/// Whether the search text appears in the project's name, id, description or tag (case-insensitive).
pub fn matches_search(project: &ProjectSummary, search: &str) -> bool {
    let needle = search.trim().to_lowercase();
    needle.is_empty()
        || [project.title, project.id, project.description, project.tag]
            .iter()
            .any(|field| field.to_lowercase().contains(&needle))
}

/// The projects visible under the current filter and search.
pub fn visible<'a>(
    projects: &'a [ProjectSummary],
    state: &ProjectsState,
) -> Vec<&'a ProjectSummary> {
    projects
        .iter()
        .filter(|project| {
            matches_filter(project, state.filter) && matches_search(project, &state.search)
        })
        .collect()
}

/// Number of projects waiting in the render queue.
pub fn queued_count(projects: &[ProjectSummary]) -> usize {
    projects.iter().filter(|project| project.queued).count()
}
