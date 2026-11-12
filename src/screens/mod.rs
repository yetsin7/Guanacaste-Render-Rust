//! Screen composition and navigation. Each screen lives in its own flow folder and only composes components.

mod build;
mod library;
mod projects;
mod render;

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
}

impl Screens {
    /// Draws `screen` into `ui`.
    pub fn show(&mut self, ui: &mut Ui, screen: Screen, context: &mut ScreenContext) {
        match screen {
            Screen::Projects => self.projects.show(ui, context),
            Screen::Build => self.build.show(ui, context),
            Screen::Library => self.library.show(ui, context),
            Screen::AssetInspector => self.inspector.show(ui, context),
            _ => render::show(ui, context),
        }
    }
}
