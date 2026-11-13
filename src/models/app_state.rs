//! `AppState`: the one place for state shared by the shell and every workspace panel — the open project
//! (scene), the active mode and build category, the camera and its saved views, photo effects, panel layout,
//! one-shot requests to the shell, and the UI state of each domain panel. Every panel receives
//! `&mut AppState` (see `screens::workspace::slots`).

use super::camera::{CameraPose, CameraStrip};
use super::navigation::{BuildCategory, Mode, Screen};
use crate::commands::workspace::Workspace;
use crate::features::build::models::BuildState;
use crate::features::materials::models::MaterialState;
use crate::features::movie::models::MovieState;
use crate::features::rendering::models::RenderStudioState;
use crate::services::sample_data::rendering as render_sample;
use crate::services::textures::Textures;
use std::path::PathBuf;

/// One adjustable effect of photo mode.
#[derive(Debug, Clone, PartialEq)]
pub struct PhotoEffect {
    /// Localization key of the effect name.
    pub key: &'static str,
    /// Whether the effect is applied.
    pub enabled: bool,
    /// Strength, normalized 0..1.
    pub amount: f32,
}

/// The effects every new scene starts with.
pub fn default_effects() -> Vec<PhotoEffect> {
    let effect = |key, enabled, amount| PhotoEffect {
        key,
        enabled,
        amount,
    };
    vec![
        effect("effect.exposure", true, 0.5),
        effect("effect.sun", true, 0.6),
        effect("effect.shadows", true, 0.7),
        effect("effect.depth", false, 0.3),
        effect("effect.color", true, 0.5),
        effect("effect.vignette", false, 0.25),
    ]
}

/// Which workspace panels are expanded.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PanelLayout {
    /// The bottom panel (category content, camera strip, timeline or queue).
    pub bottom_open: bool,
    /// The right panel (properties, effects, camera or output settings).
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
    /// Frame the selection (or the whole scene) in the 3D view (the `F` key); the 3D view clears it.
    pub frame_view: bool,
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
    /// Current camera of the 3D view.
    pub view: CameraPose,
    /// Saved camera views of the photo strip.
    pub cameras: CameraStrip,
    /// Photo-mode effects, in panel order.
    pub effects: Vec<PhotoEffect>,
    /// Expanded panels.
    pub panels: PanelLayout,
    /// Requests for the shell.
    pub requests: ShellRequests,
    /// Placeholder image cache.
    pub textures: Textures,
    /// Build-mode tools state (objects, terrain, weather, properties).
    pub build: BuildState,
    /// Material editing state.
    pub materials: MaterialState,
    /// Movie timeline and camera state.
    pub movie: MovieState,
    /// Output settings and render queue.
    pub render: RenderStudioState,
}

impl AppState {
    /// State whose recent list and recovery copies live in `data_dir`.
    pub fn with_data_dir(data_dir: PathBuf) -> Self {
        Self {
            workspace: Workspace::new(data_dir),
            mode: Mode::default(),
            category: BuildCategory::default(),
            view: CameraPose::default(),
            cameras: CameraStrip::default(),
            effects: default_effects(),
            panels: PanelLayout::default(),
            requests: ShellRequests::default(),
            textures: Textures::default(),
            build: BuildState::default(),
            materials: MaterialState::default(),
            movie: MovieState::default(),
            render: RenderStudioState {
                tab: 0,
                jobs: render_sample::jobs(),
                resolution: 2,
                samples: 0.25,
                denoise: true,
                output: "C:/Proyectos/Guanacaste/Renders".into(),
            },
        }
    }

    /// Saves the current view as a new photo-strip camera named `Cámara N`; returns its index when it fit.
    pub fn save_current_view(&mut self) -> Option<usize> {
        let name = crate::services::localization::trf(
            "camera.default_name",
            &[&(self.cameras.saved.len() + 1).to_string()],
        );
        self.cameras.save(name, self.view)
    }

    /// Moves the 3D view to saved camera `index`.
    pub fn recall_camera(&mut self, index: usize) {
        if let Some(pose) = self.cameras.recall(index) {
            self.view = pose;
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

    fn temp_state() -> AppState {
        let folder = std::env::temp_dir().join(format!(
            "gr-state-{}",
            crate::features::projects::services::util::hex(
                &crate::features::projects::services::util::new_document_id()
            )
        ));
        AppState::with_data_dir(folder)
    }

    #[test]
    fn saving_and_recalling_views_moves_the_camera() {
        let mut state = temp_state();
        assert_eq!(state.save_current_view(), Some(0));
        assert_eq!(state.cameras.saved[0].name, "Cámara 1");
        let home = state.view;
        state.view.position = [9.0, 9.0, 9.0];
        state.recall_camera(0);
        assert_eq!(state.view, home);
    }

    #[test]
    fn navigation_is_a_request() {
        let mut state = temp_state();
        state.navigate(Screen::Workspace(Mode::Photo));
        assert_eq!(
            state.requests.navigate,
            Some(Screen::Workspace(Mode::Photo))
        );
    }
}
