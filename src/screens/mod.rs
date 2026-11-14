//! Screen composition and navigation. The start screen and the scene workspace read the shared [`AppState`];
//! the remaining screens keep their own state and only need the texture cache, the open project and a way to
//! navigate.

mod help;
mod home;
mod materials;
pub mod workspace;

use crate::commands::workspace::Workspace;
use crate::features::help::models::Diagnostics;
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
    /// The open project and its workspace.
    pub workspace: &'a mut Workspace,
}

/// State of the secondary screens.
#[derive(Default)]
pub struct Screens {
    materials: materials::MaterialScreen,
    help: help::HelpScreen,
}

impl Screens {
    /// Draws `screen` into `ui`.
    pub fn show(&mut self, ui: &mut Ui, screen: Screen, app: &mut AppState) {
        match screen {
            Screen::Home => home::show(ui, app),
            Screen::Workspace(mode) => workspace::show(ui, app, mode),
            other => {
                let diagnostics = diagnostics(app);
                let mut context = ScreenContext {
                    textures: &mut app.textures,
                    navigate: &mut app.requests.navigate,
                    workspace: &mut app.workspace,
                };
                match other {
                    Screen::MaterialEditor => self.materials.show(ui, &mut context),
                    _ => self.help.show(ui, &mut context, &diagnostics),
                }
            }
        }
    }
}

/// Facts about the session for the help screen.
fn diagnostics(app: &AppState) -> Diagnostics {
    let project = app.workspace.project.as_ref();
    Diagnostics {
        gpu: app.viewport.gpu_available,
        threads: std::thread::available_parallelism().map_or(1, |n| n.get()),
        project: app.workspace.name().map(str::to_owned),
        meshes: project.map_or(0, |p| p.meshes.len()),
        triangles: project.map_or(0, |p| p.meshes.iter().map(|m| m.indices.len() / 3).sum()),
        objects: app.objects.instances().len(),
        plants: app.landscape.landscape.vegetation.len(),
        cameras: app.cameras.len(),
    }
}
