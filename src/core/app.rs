//! The eframe application: frameless shell (title bar, mode bar) around the active screen.

use crate::components::{mode_bar, title_bar};
use crate::config::launch::LaunchOptions;
use crate::config::theme::{self, colors};
use crate::core::dialogs::DialogHost;
use crate::core::shortcuts;
use crate::models::navigation::Screen;
use crate::models::preferences::Preferences;
use crate::models::AppState;
use crate::screens::Screens;
use crate::services::sample_data::shell;
use eframe::egui;

/// Application state.
pub struct StudioApp {
    screen: Screen,
    screens: Screens,
    app: AppState,
    dialogs: DialogHost,
    preferences: Preferences,
    applied_scale: usize,
    pending_dialog: Option<crate::models::dialog::Dialog>,
    window_title: String,
    last_project: Option<[u8; 16]>,
}

impl StudioApp {
    /// Creates the app, installing the theme and opening the screen requested at launch.
    pub fn new(creation: &eframe::CreationContext<'_>, options: LaunchOptions) -> Self {
        theme::apply(&creation.egui_ctx);
        if let Some(zoom) = options.zoom {
            creation.egui_ctx.set_zoom_factor(zoom);
        }
        let mut app = AppState::default();
        let mut dialogs = DialogHost::new(&app.workspace);
        if let Some(path) = &options.open {
            dialogs.open_at_start(&mut app.workspace, path);
        }
        let screen = options.screen.unwrap_or_default();
        if let Some(mode) = screen.mode() {
            app.mode = mode;
        }
        Self {
            screen,
            screens: Screens::default(),
            last_project: app.workspace.project.as_ref().map(|p| p.id),
            app,
            dialogs,
            preferences: Preferences::default(),
            applied_scale: Preferences::default().scale_index,
            pending_dialog: options.dialog,
            window_title: String::new(),
        }
    }

    /// The screen currently shown.
    pub fn screen(&self) -> Screen {
        self.screen
    }

    /// Applies a navigation request, remembering the workspace mode.
    fn go(&mut self, screen: Screen) {
        self.screen = screen;
        if let Some(mode) = screen.mode() {
            self.app.mode = mode;
        }
    }

    /// Leaves the start screen when a different project was just created, opened or imported.
    fn follow_project(&mut self) {
        let id = self.app.workspace.project.as_ref().map(|p| p.id);
        if id != self.last_project {
            self.last_project = id;
            if id.is_some() && self.screen == Screen::Home {
                self.go(Screen::Workspace(self.app.mode));
            }
        }
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
                let workspace = &self.app.workspace;
                let info = title_bar::TitleBarInfo {
                    fps: shell::FPS_LABEL,
                    resolution: shell::RESOLUTION_LABEL,
                    show_fps: self.preferences.show_fps,
                    project: workspace.name().map(|name| (name, workspace.dirty())),
                };
                if let Some(anchor) = title_bar::show(ui, &info) {
                    self.dialogs.open_menu(anchor);
                }
            });
        if self.screen.shows_mode_bar() {
            egui::Panel::top("mode-bar")
                .frame(bar)
                .exact_size(mode_bar::HEIGHT)
                .show(ui, |ui| {
                    if let Some(target) = mode_bar::show(ui, self.screen, self.app.mode) {
                        self.app.navigate(target);
                    }
                });
        }
        egui::CentralPanel::default()
            .frame(egui::Frame::NONE.fill(colors::SURFACE))
            .show(ui, |ui| self.screens.show(ui, self.screen, &mut self.app));
        let ctx = ui.ctx().clone();
        shortcuts::handle(
            &ctx,
            &mut self.app,
            self.screen,
            self.dialogs.active().is_some(),
        );
        if let Some(target) = self.app.requests.navigate.take() {
            self.go(target);
        }
        if let Some(dialog) = self.pending_dialog.take() {
            crate::commands::request_dialog(&ctx, dialog);
        }
        if ctx.input(|input| input.viewport().close_requested())
            && self.dialogs.intercept_close(&ctx, &mut self.app.workspace)
        {
            ctx.send_viewport_cmd(egui::ViewportCommand::CancelClose);
        }
        self.dialogs
            .show(&ctx, &mut self.app.workspace, &mut self.preferences);
        self.follow_project();
        let title = self.app.workspace.window_title();
        if title != self.window_title {
            ctx.send_viewport_cmd(egui::ViewportCommand::Title(title.clone()));
            self.window_title = title;
        }
        if self.preferences.scale_index != self.applied_scale {
            self.applied_scale = self.preferences.scale_index;
            ctx.set_zoom_factor(self.preferences.zoom());
        }
        crate::core::window_frame::resize_edges(&ctx);
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
