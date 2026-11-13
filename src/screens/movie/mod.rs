//! Movie studio screen: the camera path over the 3D view, the sequence card, the inspector and the timeline.
//! The animation is kept in the open project's document (`document.animation`); without a project it is local.

use super::ScreenContext;
use crate::features::movie::components::export_panel::ExportPanel;
use crate::features::movie::components::header_bar::HeaderAction;
use crate::features::movie::components::{
    camera_inspector, camera_path, clip_card, header_bar, timeline_rows, timeline_toolbar,
};
use crate::features::movie::logic;
use crate::features::movie::models::{Animation, MovieState, Selection};
use crate::features::viewport::components::backdrop;
use crate::models::navigation::Screen;
use eframe::egui::{pos2, Key, Rect, Ui};

const TIMELINE_HEIGHT: f32 = 240.0;

/// State of the movie studio screen.
#[derive(Default)]
pub struct MovieScreen {
    state: MovieState,
    animation: Animation,
    /// The animation as the project document holds it.
    baseline: Animation,
    synced_project: Option<[u8; 16]>,
    export: ExportPanel,
}

impl MovieScreen {
    /// Draws the screen.
    pub fn show(&mut self, ui: &mut Ui, context: &mut ScreenContext) {
        self.load_from_project(context);
        ui.spacing_mut().item_spacing.y = 0.0;
        let scene = context
            .workspace
            .project
            .as_ref()
            .map(|project| project.document.meta.name.clone());
        match header_bar::show(ui, scene.as_deref()) {
            Some(HeaderAction::Close) => *context.navigate = Some(Screen::Projects),
            Some(HeaderAction::Export) => self.export.show_dialog(),
            None => {}
        }
        self.keyboard(ui);
        let dt = f64::from(ui.input(|input| input.stable_dt).min(0.1));
        logic::advance(&mut self.state, &self.animation, dt);
        self.export.update();
        if self.state.playing || self.export.is_busy() {
            ui.ctx().request_repaint();
        }
        let body = Rect::from_min_max(
            pos2(ui.max_rect().left(), ui.cursor().top()),
            ui.max_rect().max,
        );
        let timeline =
            Rect::from_min_max(pos2(body.left(), body.bottom() - TIMELINE_HEIGHT), body.max);
        let view = Rect::from_min_max(body.min, pos2(body.right(), timeline.top()));
        backdrop::paint(ui, view, context.textures);
        camera_path::show(ui, view, &mut self.animation, &mut self.state);
        self.overlays(ui, view);
        self.timeline(ui, timeline);
        self.export.show(ui.ctx(), &self.animation);
        self.store_in_project(context);
    }

    /// Takes the animation of the open project when it changed (another project, or the file was reloaded).
    fn load_from_project(&mut self, context: &mut ScreenContext) {
        let Some(project) = context.workspace.project.as_mut() else {
            self.synced_project = None;
            return;
        };
        if self.synced_project == Some(project.id) && project.document.animation == self.baseline {
            return;
        }
        if self.synced_project != Some(project.id) {
            self.state = MovieState::default();
        }
        let mut animation = project.document.animation.clone();
        animation.normalize();
        project.document.animation = animation.clone();
        self.baseline = animation.clone();
        self.animation = animation;
        self.synced_project = Some(project.id);
    }

    /// Writes the edits back into the project and marks it as having unsaved changes.
    fn store_in_project(&mut self, context: &mut ScreenContext) {
        if self.animation == self.baseline {
            return;
        }
        self.animation.normalize();
        if let Some(project) = context.workspace.project.as_mut() {
            if !project.read_only {
                project.document.animation = self.animation.clone();
                project.dirty = true;
            }
        }
        self.baseline = self.animation.clone();
    }

    fn keyboard(&mut self, ui: &Ui) {
        if ui.ctx().egui_wants_keyboard_input() {
            return;
        }
        if ui.input(|input| input.key_pressed(Key::Space)) {
            logic::toggle_play(&mut self.state, &self.animation);
        }
        if ui.input(|input| input.key_pressed(Key::Delete)) {
            match self.state.selection {
                Selection::Camera(index) => {
                    self.animation.remove_camera_key(index);
                }
                Selection::Value { channel, key } => {
                    self.animation.remove_value_key(channel, key);
                }
                Selection::None => {}
            }
            self.state.selection = Selection::None;
        }
    }

    fn overlays(&mut self, ui: &mut Ui, view: Rect) {
        let inner = view.shrink(12.0);
        clip_card::show(ui, inner.min, &mut self.animation);
        camera_inspector::show(
            ui,
            pos2(inner.right() - camera_inspector::WIDTH, inner.top()),
            inner.height(),
            &mut self.animation,
            &mut self.state,
        );
    }

    fn timeline(&mut self, ui: &mut Ui, timeline: Rect) {
        ui.painter()
            .rect_filled(timeline, 0.0, crate::config::theme::colors::SURFACE_LOW);
        let mut inner = ui.new_child(eframe::egui::UiBuilder::new().max_rect(timeline));
        inner.spacing_mut().item_spacing.y = 0.0;
        let toolbar = timeline_toolbar::show(&mut inner, &mut self.animation, &mut self.state);
        if toolbar.export {
            self.export.show_dialog();
        }
        let rows = Rect::from_min_max(pos2(timeline.left(), timeline.top() + 40.0), timeline.max);
        timeline_rows::show(ui, rows, &mut self.animation, &mut self.state);
    }
}
