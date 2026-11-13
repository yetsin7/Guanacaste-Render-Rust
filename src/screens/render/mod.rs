//! Render studio screen: output settings with the live picture, and the render queue.

use super::ScreenContext;
use crate::features::rendering::components::output_panel::{self, PanelAction};
use crate::features::rendering::components::preview_view::PreviewView;
use crate::features::rendering::components::queue_table::{self, QueueAction};
use crate::features::rendering::components::studio_header::{self, HeaderAction};
use crate::features::rendering::engine::ViewPreset;
use crate::features::rendering::logic::plan::plan_from_studio;
use crate::features::rendering::models::RenderStudioState;
use crate::features::rendering::services::{open_folder, safe_stem, FolderPicker};
use crate::services::localization::{tr, trf};
use eframe::egui::{pos2, Rect, Ui};
use std::path::PathBuf;
use std::time::Duration;

/// State of the render studio screen.
#[derive(Default)]
pub struct RenderScreen {
    state: RenderStudioState,
    picker: FolderPicker,
    preview: PreviewView,
}

impl RenderScreen {
    /// Draws the screen.
    pub fn show(&mut self, ui: &mut Ui, context: &mut ScreenContext) {
        ui.spacing_mut().item_spacing.y = 0.0;
        self.state.jobs = context.render.queue.snapshot();
        if let Some(folder) = self.picker.poll() {
            self.state.output = folder.display().to_string();
        }
        let ready = context
            .workspace
            .project
            .as_ref()
            .is_some_and(|project| !project.meshes.is_empty());
        if let Some(action) = studio_header::show(ui, &mut self.state, ready) {
            self.start(context, action);
        }
        let body = Rect::from_min_max(
            pos2(ui.max_rect().left(), ui.cursor().top()),
            ui.max_rect().max,
        );
        if self.state.tab == 1 {
            match queue_table::show(ui, body, &self.state) {
                Some(QueueAction::Cancel(id)) => context.render.queue.cancel(id),
                Some(QueueAction::Retry(id)) => context.render.queue.retry(id),
                Some(QueueAction::Remove(id)) => context.render.queue.remove(id),
                Some(QueueAction::ClearFinished) => {
                    context.render.queue.clear_finished();
                }
                None => {}
            }
        } else {
            let panel = Rect::from_min_max(
                pos2(body.right() - output_panel::WIDTH, body.top()),
                body.max,
            );
            let view = Rect::from_min_max(body.min, pos2(panel.left(), body.bottom()));
            self.preview.update(ui.ctx(), &context.render.queue);
            self.preview.show(ui, view, &self.state.jobs);
            match output_panel::show(ui, panel, &mut self.state) {
                Some(PanelAction::Browse) => self.picker.open(&self.state.output),
                Some(PanelAction::OpenFolder) => self.show_output_folder(),
                None => {}
            }
        }
        if context.render.queue.is_busy() || self.picker.is_open() {
            ui.ctx().request_repaint_after(Duration::from_millis(100));
        }
    }

    fn show_output_folder(&mut self) {
        if !open_folder(&PathBuf::from(self.state.output.trim())) {
            self.state.notice = Some((tr("render.folder_missing").to_owned(), true));
        }
    }

    fn start(&mut self, context: &mut ScreenContext, action: HeaderAction) {
        let plan = plan_from_studio(&self.state);
        if safe_stem(&plan.output.file_name) != plan.output.file_name.trim() {
            self.state.file_name = safe_stem(&plan.output.file_name);
        }
        let views: &[ViewPreset] = match action {
            HeaderAction::Render => &[plan.view],
            HeaderAction::RenderAllViews => &ViewPreset::ALL,
        };
        self.state.notice = Some(
            match context.render.enqueue(&*context.workspace, &plan, views) {
                Ok(count) => (trf("render.queued_n", &[&count.to_string()]), false),
                Err(reason) => (tr(reason).to_owned(), true),
            },
        );
    }
}
