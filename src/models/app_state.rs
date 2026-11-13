//! `AppState`: the one place for state shared by the shell and every workspace panel — the open project (scene),
//! the active mode and build category, the shared 3D view and its camera, the state of each domain (objects,
//! landscape, lighting, environment, materials, camera studio, movie, render studio), panel layout and one-shot
//! requests to the shell. Every panel receives `&mut AppState` (see `screens::workspace`).
//!
//! The domain states are the single live copy of their data; `commands::sync` copies them into the project
//! document when they change (so Save writes them) and loads them back when a project is opened.

use super::navigation::{BuildCategory, Mode, PhotoSection, Screen};
use super::render_studio::RenderStudio;
use crate::commands::rendering::RenderHub;
use crate::commands::scene::SceneComposer;
use crate::commands::sync::Baselines;
use crate::commands::thumbnails::Thumbnails;
use crate::commands::workspace::Workspace;
use crate::features::build::models::ObjectsState;
use crate::features::camera::models::state::CameraStudioState;
use crate::features::environment::EnvironmentSettings;
use crate::features::landscape::models::LandscapeState;
use crate::features::lighting::models::LightingState;
use crate::features::materials::components::session::Session;
use crate::features::materials::models::MaterialState;
use crate::features::movie::components::export_panel::ExportPanel;
use crate::features::movie::models::{Animation, MovieState};
use crate::features::projects::models::CameraDoc;
use crate::features::viewport::ui::ViewportState;
use crate::services::textures::Textures;
use std::path::PathBuf;

/// Which workspace panels are expanded.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PanelLayout {
    /// The bottom drawer (category tabs, camera strip, timeline or queue).
    pub bottom_open: bool,
    /// The right panel (category content, effects, camera inspector or output settings).
    pub side_open: bool,
    /// The shortcut hint strip.
    pub hints_open: bool,
}

impl Default for PanelLayout {
    fn default() -> Self {
        Self {
            bottom_open: true,
            side_open: true,
            hints_open: true,
        }
    }
}

/// One-shot requests a panel raises and the shell serves at the end of the frame.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct ShellRequests {
    /// Go to another screen.
    pub navigate: Option<Screen>,
}

/// State shared by the shell and every workspace panel.
pub struct AppState {
    /// The open project (the scene) and its files.
    pub workspace: Workspace,
    /// Active workspace mode (kept while visiting other screens).
    pub mode: Mode,
    /// Active build-mode category tab.
    pub category: BuildCategory,
    /// Active section of the photo mode's left rail.
    pub photo_section: PhotoSection,
    /// The "live style" switch of the photo bar (the real-time view keeps showing the photo filters).
    pub live_style: bool,
    /// Thumbnails of the saved cameras, captured from the 3D view.
    pub thumbs: Thumbnails,
    /// Render progress history and the completion notice.
    pub render_watch: crate::commands::render_watch::RenderWatch,
    /// Expanded panels.
    pub panels: PanelLayout,
    /// Requests for the shell.
    pub requests: ShellRequests,
    /// Placeholder image cache.
    pub textures: Textures,
    /// The shared 3D view: camera, saved slots and the scene sent to the GPU.
    pub viewport: ViewportState,
    /// Builds the scene shown in the 3D view from the project, the landscape and the placed objects.
    pub composer: SceneComposer,
    /// Placed objects, selection and history.
    pub objects: ObjectsState,
    /// Terrain, water and vegetation with the brush settings.
    pub landscape: LandscapeState,
    /// Lights, groups, emissive materials and lighting scenes.
    pub lighting: LightingState,
    /// Sun, sky and weather.
    pub environment: EnvironmentSettings,
    /// Material editing state.
    pub materials: MaterialState,
    /// Material editor previews.
    pub material_session: Session,
    /// Photographic camera and filters of the photo mode.
    pub camera_studio: CameraStudioState,
    /// Saved cameras of the photo strip, kept in the project as `CameraDoc`.
    pub cameras: Vec<CameraDoc>,
    /// Movie timeline state.
    pub movie: MovieState,
    /// The camera animation being edited.
    pub animation: Animation,
    /// Movie export dialog.
    pub movie_export: ExportPanel,
    /// Render studio (output settings, queue view, preview).
    pub studio: RenderStudio,
    /// The render queue.
    pub render: RenderHub,
    /// What each domain state looked like when it last matched the project document.
    pub baselines: Baselines,
}

impl AppState {
    /// State whose recent list and recovery copies live in `data_dir`.
    pub fn with_data_dir(data_dir: PathBuf) -> Self {
        Self {
            workspace: Workspace::new(data_dir),
            mode: Mode::default(),
            category: BuildCategory::default(),
            photo_section: PhotoSection::default(),
            live_style: true,
            thumbs: Thumbnails::default(),
            render_watch: Default::default(),
            panels: PanelLayout::default(),
            requests: ShellRequests::default(),
            textures: Textures::default(),
            viewport: ViewportState::new(),
            composer: SceneComposer::default(),
            objects: ObjectsState::default(),
            landscape: LandscapeState::default(),
            lighting: LightingState::default(),
            environment: EnvironmentSettings::default(),
            materials: MaterialState::default(),
            material_session: Session::default(),
            camera_studio: CameraStudioState::default(),
            cameras: Vec::new(),
            movie: MovieState::default(),
            animation: Animation::default(),
            movie_export: ExportPanel::default(),
            studio: RenderStudio::default(),
            render: RenderHub::default(),
            baselines: Baselines::default(),
        }
    }

    /// Asks the shell to show `screen` (switching the workspace mode when it is one).
    pub fn navigate(&mut self, screen: Screen) {
        self.requests.navigate = Some(screen);
    }
}

impl Default for AppState {
    fn default() -> Self {
        Self::with_data_dir(crate::features::projects::services::recovery::data_dir())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn navigation_is_a_request() {
        let folder = std::env::temp_dir().join("gr-state-nav");
        let mut state = AppState::with_data_dir(folder);
        state.navigate(Screen::Workspace(Mode::Photo));
        assert_eq!(
            state.requests.navigate,
            Some(Screen::Workspace(Mode::Photo))
        );
    }
}
