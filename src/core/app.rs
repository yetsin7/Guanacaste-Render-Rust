//! The eframe application: frameless shell (title bar, tool rail) around the active screen.

use crate::components::{title_bar, tool_rail};
use crate::config::launch::LaunchOptions;
use crate::config::theme::{self, colors};
use crate::models::navigation::Screen;
use crate::screens::{ScreenContext, Screens};
use crate::services::sample_data::shell;
use crate::services::textures::Textures;
use eframe::egui;

/// Application state.
pub struct StudioApp {
    screen: Screen,
    screens: Screens,
    textures: Textures,
}

impl StudioApp {
    /// Creates the app, installing the theme and opening the screen requested at launch.
    pub fn new(creation: &eframe::CreationContext<'_>, options: LaunchOptions) -> Self {
        theme::apply(&creation.egui_ctx);
        if let Some(zoom) = options.zoom {
            creation.egui_ctx.set_zoom_factor(zoom);
        }
        Self {
            screen: options.screen.unwrap_or_default(),
            screens: Screens::default(),
            textures: Textures::default(),
        }
    }

    /// The screen currently shown.
    pub fn screen(&self) -> Screen {
        self.screen
    }
}

impl eframe::App for StudioApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let bar = egui::Frame::NONE
            .fill(colors::SURFACE_LOW)
            .stroke(egui::Stroke::new(1.0, colors::SURFACE_HIGHEST));
        egui::Panel::top("title-bar")
            .frame(bar)
            .exact_size(title_bar::HEIGHT)
            .show(ui, |ui| {
                let info = title_bar::TitleBarInfo {
                    fps: shell::FPS_LABEL,
                    resolution: shell::RESOLUTION_LABEL,
                };
                title_bar::show(ui, &info);
            });
        let mut requested = None;
        egui::Panel::left("tool-rail")
            .frame(bar)
            .exact_size(tool_rail::WIDTH)
            .resizable(false)
            .show(ui, |ui| {
                if let Some(item) = tool_rail::show(ui, self.screen.rail_item()) {
                    requested = Some(item.target());
                }
            });
        egui::CentralPanel::default()
            .frame(egui::Frame::NONE.fill(colors::SURFACE))
            .show(ui, |ui| {
                let mut context = ScreenContext {
                    textures: &mut self.textures,
                    navigate: &mut requested,
                };
                self.screens.show(ui, self.screen, &mut context);
            });
        if let Some(target) = requested {
            self.screen = target;
        }
        crate::core::window_frame::resize_edges(ui.ctx());
    }
}

/// Builds the native window options of the application.
pub fn native_options(options: &LaunchOptions) -> eframe::NativeOptions {
    let mut viewport = egui::ViewportBuilder::default()
        .with_title("Guanacaste Render")
        .with_min_inner_size([900.0, 560.0])
        .with_decorations(false);
    viewport = match options.size {
        Some((width, height)) => {
            let zoom = options.zoom.unwrap_or(1.0);
            viewport.with_inner_size([width * zoom, height * zoom])
        }
        None => viewport
            .with_inner_size([1280.0, 720.0])
            .with_maximized(true),
    };
    eframe::NativeOptions {
        viewport,
        ..Default::default()
    }
}

/// Starts the application.
pub fn run() -> eframe::Result {
    let options = LaunchOptions::from_environment();
    eframe::run_native(
        "Guanacaste Render",
        native_options(&options),
        Box::new(move |creation| Ok(Box::new(StudioApp::new(creation, options)))),
    )
}
