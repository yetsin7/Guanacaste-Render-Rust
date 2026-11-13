//! Screen composition and navigation. The start screen and the scene workspace read the shared [`AppState`];
//! the remaining screens keep their own state and only need the texture cache and a way to navigate.

mod help;
mod home;
mod library;
mod materials;
mod settings;
pub mod workspace;

use crate::models::navigation::Screen;
use crate::models::AppState;
use crate::services::textures::Textures;
use eframe::egui::Ui;

/// What a secondary screen may use while it draws.
pub struct ScreenContext<'a> {
    /// Placeholder image cache.
    pub textures: &'a mut Textures,
    /// Set by a screen to request navigation to another screen.
    pub navigate: &'a mut Option<Screen>,
}

/// State of the secondary screens.
#[derive(Default)]
pub struct Screens {
    library: library::LibraryScreen,
    inspector: library::InspectorScreen,
    materials: materials::MaterialScreen,
    settings: settings::SettingsScreen,
    help: help::HelpScreen,
}

impl Screens {
    /// Draws `screen` into `ui`.
    pub fn show(&mut self, ui: &mut Ui, screen: Screen, app: &mut AppState) {
        match screen {
            Screen::Home => home::show(ui, app),
            Screen::Workspace(mode) => workspace::show(ui, app, mode),
            other => {
                let mut context = ScreenContext {
                    textures: &mut app.textures,
                    navigate: &mut app.requests.navigate,
                };
                match other {
                    Screen::Library => self.library.show(ui, &mut context),
                    Screen::AssetInspector => self.inspector.show(ui, &mut context),
                    Screen::MaterialEditor => self.materials.show(ui, &mut context),
                    Screen::Settings => self.settings.show(ui, &mut context),
                    _ => self.help.show(ui, &mut context),
                }
            }
        }
    }
}
