//! Project entities and the screen's UI state.

use crate::models::scene_art::SceneArt;

/// A project listed in the repository.
#[derive(Debug, Clone, PartialEq)]
pub struct ProjectSummary {
    /// Short identifier, e.g. `#KEN-094`.
    pub id: &'static str,
    /// Display name.
    pub title: &'static str,
    /// One or two sentences describing the scene.
    pub description: &'static str,
    /// Size on disk, e.g. `2.8 GB`.
    pub size: &'static str,
    /// Lighting preset shown over the thumbnail.
    pub tag: &'static str,
    /// Render engine name.
    pub engine: &'static str,
    /// Frame rate measured for the scene.
    pub fps: u32,
    /// Placeholder thumbnail.
    pub art: SceneArt,
    /// True while the project waits in the render queue.
    pub queued: bool,
    /// True when the project is synced to the cloud.
    pub cloud: bool,
    /// True when the project is archived.
    pub archived: bool,
}

/// The project open in the workspace, shown in the hero panel.
#[derive(Debug, Clone, PartialEq)]
pub struct ActiveProject {
    /// Display name.
    pub title: &'static str,
    /// Description paragraph.
    pub description: &'static str,
    /// Relative modification time.
    pub modified: &'static str,
    /// Render quality label shown over the picture.
    pub quality: &'static str,
    /// Frame-rate readout.
    pub fps: &'static str,
    /// Camera name.
    pub camera: &'static str,
    /// Lens readout.
    pub lens: &'static str,
    /// Exposure readout.
    pub exposure: &'static str,
    /// Output resolution label.
    pub resolution: &'static str,
    /// Engine state value.
    pub engine_state: &'static str,
    /// File size value.
    pub file_size: &'static str,
    /// Geometry value.
    pub geometry: &'static str,
    /// Lighting cache value.
    pub lighting: &'static str,
    /// Average frame time readout.
    pub frametime: &'static str,
    /// Frame-time series for the sparkline.
    pub frametime_series: Vec<f32>,
}

/// Filter tabs of the repository.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ProjectFilter {
    /// Every project.
    #[default]
    All,
    /// Projects waiting in the render queue.
    RenderQueue,
    /// Projects synced to the cloud.
    CloudSynced,
    /// Archived projects.
    Archived,
}

/// How the repository lays projects out.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ProjectLayout {
    /// Cards in a grid.
    #[default]
    Grid,
    /// Compact rows.
    List,
}

/// UI state of the projects screen.
#[derive(Debug, Default)]
pub struct ProjectsState {
    /// Text typed in the search box.
    pub search: String,
    /// Selected filter tab.
    pub filter: ProjectFilter,
    /// Selected layout.
    pub layout: ProjectLayout,
}
