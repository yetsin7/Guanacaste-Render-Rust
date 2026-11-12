//! Navigation vocabulary shared by the shell and every screen.

/// Every screen of the application.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum Screen {
    /// Project repository.
    #[default]
    Projects,
    /// Object placement over the 3D view.
    Build,
    /// Nature asset library.
    Library,
    /// Detail view of one library asset.
    AssetInspector,
    /// PBR material editor.
    MaterialEditor,
    /// Camera inspector and timeline.
    MovieStudio,
    /// Render studio (not implemented yet).
    Render,
    /// Engine and viewport settings.
    Settings,
    /// Help, shortcuts and diagnostics.
    Help,
}

/// Entries of the left tool rail.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RailItem {
    /// Projects.
    Projects,
    /// Build mode (and the material editor opened from it).
    Build,
    /// Library (and the asset inspector opened from it).
    Library,
    /// Movie studio.
    Movie,
    /// Render studio.
    Render,
    /// Performance readouts (opens the settings screen).
    Performance,
    /// Settings.
    Settings,
    /// Help and diagnostics.
    Help,
}

impl Screen {
    /// Every screen, in navigation order.
    pub const ALL: [Screen; 9] = [
        Screen::Projects,
        Screen::Build,
        Screen::Library,
        Screen::AssetInspector,
        Screen::MaterialEditor,
        Screen::MovieStudio,
        Screen::Render,
        Screen::Settings,
        Screen::Help,
    ];

    /// Stable identifier used by the `--screen=` launch option.
    pub fn slug(self) -> &'static str {
        match self {
            Screen::Projects => "projects",
            Screen::Build => "build",
            Screen::Library => "library",
            Screen::AssetInspector => "inspector",
            Screen::MaterialEditor => "materials",
            Screen::MovieStudio => "movie",
            Screen::Render => "render",
            Screen::Settings => "settings",
            Screen::Help => "help",
        }
    }

    /// Finds a screen by its slug.
    pub fn from_slug(slug: &str) -> Option<Screen> {
        Screen::ALL.into_iter().find(|screen| screen.slug() == slug)
    }

    /// The rail entry highlighted while this screen is shown.
    pub fn rail_item(self) -> RailItem {
        match self {
            Screen::Projects => RailItem::Projects,
            Screen::Build | Screen::MaterialEditor => RailItem::Build,
            Screen::Library | Screen::AssetInspector => RailItem::Library,
            Screen::MovieStudio => RailItem::Movie,
            Screen::Render => RailItem::Render,
            Screen::Settings => RailItem::Settings,
            Screen::Help => RailItem::Help,
        }
    }
}

impl RailItem {
    /// The screen a click on this entry opens.
    pub fn target(self) -> Screen {
        match self {
            RailItem::Projects => Screen::Projects,
            RailItem::Build => Screen::Build,
            RailItem::Library => Screen::Library,
            RailItem::Movie => Screen::MovieStudio,
            RailItem::Render => Screen::Render,
            RailItem::Performance | RailItem::Settings => Screen::Settings,
            RailItem::Help => Screen::Help,
        }
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
    fn rail_targets_highlight_their_own_item() {
        for item in [
            RailItem::Projects,
            RailItem::Build,
            RailItem::Library,
            RailItem::Movie,
            RailItem::Render,
            RailItem::Settings,
            RailItem::Help,
        ] {
            assert_eq!(item.target().rail_item(), item);
        }
    }

    #[test]
    fn detail_screens_keep_their_parent_highlighted() {
        assert_eq!(Screen::MaterialEditor.rail_item(), RailItem::Build);
        assert_eq!(Screen::AssetInspector.rail_item(), RailItem::Library);
    }
}
