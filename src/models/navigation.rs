//! Navigation vocabulary shared by the shell and every screen: the start screen, the scene workspace with its
//! four modes, the build categories, and the secondary screens reached from the mode bar.

/// The four modes of the scene workspace, always reachable from the mode bar.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum Mode {
    /// Place nature, water, objects, lights and materials; set the sky.
    #[default]
    Build,
    /// Save camera views and adjust photo effects.
    Photo,
    /// Arrange camera clips on a timeline.
    Movie,
    /// Output settings and the render queue.
    Export,
}

impl Mode {
    /// Every mode, in mode-bar order.
    pub const ALL: [Mode; 4] = [Mode::Build, Mode::Photo, Mode::Movie, Mode::Export];

    /// Position in the mode bar (also the number key that selects it, minus one).
    pub fn index(self) -> usize {
        Mode::ALL.iter().position(|mode| *mode == self).unwrap_or(0)
    }
}

/// The sections of the left rail of photo mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PhotoSection {
    /// Sun, sky, clouds and weather.
    #[default]
    Environment,
    /// The filter stack.
    Image,
    /// Lens, framing and focus.
    Camera,
    /// Material editing.
    Material,
    /// Output quality.
    Quality,
    /// AI render (not available yet).
    AiRender,
}

impl PhotoSection {
    /// Every section, in rail order.
    pub const ALL: [PhotoSection; 6] = [
        PhotoSection::Environment,
        PhotoSection::Image,
        PhotoSection::Camera,
        PhotoSection::Material,
        PhotoSection::Quality,
        PhotoSection::AiRender,
    ];
}

/// The category tabs at the bottom of build mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum BuildCategory {
    /// Trees, plants and grass.
    #[default]
    Nature,
    /// Terrain sculpting and water bodies.
    WaterTerrain,
    /// Furniture, people, vehicles and other objects.
    Objects,
    /// Artificial lights.
    Lights,
    /// Material assignment.
    Materials,
    /// Sun, sky and weather.
    Sky,
}

impl BuildCategory {
    /// Every category, in tab order.
    pub const ALL: [BuildCategory; 6] = [
        BuildCategory::Nature,
        BuildCategory::WaterTerrain,
        BuildCategory::Objects,
        BuildCategory::Lights,
        BuildCategory::Materials,
        BuildCategory::Sky,
    ];
}

/// Every screen of the application.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum Screen {
    /// Start screen: recent projects, new, open and import.
    #[default]
    Home,
    /// The full-screen 3D view in one of its modes.
    Workspace(Mode),
    /// Nature asset library.
    Library,
    /// Detail view of one library asset.
    AssetInspector,
    /// Full PBR material editor.
    MaterialEditor,
    /// Engine and viewport settings.
    Settings,
    /// Help, shortcuts and diagnostics.
    Help,
}

impl Screen {
    /// Every screen, in navigation order.
    pub const ALL: [Screen; 10] = [
        Screen::Home,
        Screen::Workspace(Mode::Build),
        Screen::Workspace(Mode::Photo),
        Screen::Workspace(Mode::Movie),
        Screen::Workspace(Mode::Export),
        Screen::Library,
        Screen::AssetInspector,
        Screen::MaterialEditor,
        Screen::Settings,
        Screen::Help,
    ];

    /// Stable identifier used by the `--screen=` launch option.
    pub fn slug(self) -> &'static str {
        match self {
            Screen::Home => "home",
            Screen::Workspace(Mode::Build) => "build",
            Screen::Workspace(Mode::Photo) => "photo",
            Screen::Workspace(Mode::Movie) => "movie",
            Screen::Workspace(Mode::Export) => "render",
            Screen::Library => "library",
            Screen::AssetInspector => "inspector",
            Screen::MaterialEditor => "materials",
            Screen::Settings => "settings",
            Screen::Help => "help",
        }
    }

    /// Finds a screen by its slug; `projects` and `export` are accepted as older names.
    pub fn from_slug(slug: &str) -> Option<Screen> {
        match slug {
            "projects" => Some(Screen::Home),
            "export" => Some(Screen::Workspace(Mode::Export)),
            _ => Screen::ALL.into_iter().find(|screen| screen.slug() == slug),
        }
    }

    /// The workspace mode this screen shows, if it is the workspace.
    pub fn mode(self) -> Option<Mode> {
        match self {
            Screen::Workspace(mode) => Some(mode),
            _ => None,
        }
    }

    /// Whether the mode bar is shown above this screen (everything but the start screen).
    pub fn shows_mode_bar(self) -> bool {
        self != Screen::Home
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn slugs_round_trip() {
        for screen in Screen::ALL {
            assert_eq!(Screen::from_slug(screen.slug()), Some(screen));
        }
    }

    #[test]
    fn older_slugs_still_work() {
        assert_eq!(Screen::from_slug("projects"), Some(Screen::Home));
        assert_eq!(
            Screen::from_slug("export"),
            Some(Screen::Workspace(Mode::Export))
        );
    }

    #[test]
    fn mode_index_matches_bar_order() {
        for (index, mode) in Mode::ALL.into_iter().enumerate() {
            assert_eq!(mode.index(), index);
        }
    }

    #[test]
    fn only_the_workspace_has_a_mode() {
        assert_eq!(Screen::Workspace(Mode::Photo).mode(), Some(Mode::Photo));
        assert_eq!(Screen::Library.mode(), None);
        assert!(!Screen::Home.shows_mode_bar());
    }
}
