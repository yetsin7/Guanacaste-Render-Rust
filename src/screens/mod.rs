//! Screen composition and navigation. Each screen lives in its own flow folder and only composes components.

mod build;
mod help;
mod library;
mod materials;
mod movie;
mod projects;
mod render;
mod settings;

use crate::models::navigation::Screen;
use crate::services::textures::Textures;
use eframe::egui::Ui;

/// What a screen may use while it draws.
pub struct ScreenContext<'a> {
    /// Placeholder image cache.
    pub textures: &'a mut Textures,
    /// Set by a screen to request navigation to another screen.
    pub navigate: &'a mut Option<Screen>,
}

/// State of every screen.
#[derive(Default)]
pub struct Screens {
    projects: projects::ProjectsScreen,
    build: build::BuildScreen,
    library: library::LibraryScreen,
    inspector: library::InspectorScreen,
    materials: materials::MaterialScreen,
    movie: movie::MovieScreen,
    settings: settings::SettingsScreen,
    help: help::HelpScreen,
}

impl Screens {
    /// Draws `screen` into `ui`.
    pub fn show(&mut self, ui: &mut Ui, screen: Screen, context: &mut ScreenContext) {
        match screen {
            Screen::Projects => self.projects.show(ui, context),
            Screen::Build => self.build.show(ui, context),
            Screen::Library => self.library.show(ui, context),
            Screen::AssetInspector => self.inspector.show(ui, context),
            Screen::MaterialEditor => self.materials.show(ui, context),
            Screen::MovieStudio => self.movie.show(ui, context),
            Screen::Settings => self.settings.show(ui, context),
            Screen::Help => self.help.show(ui, context),
            _ => render::show(ui, context),
        }
    }
}
