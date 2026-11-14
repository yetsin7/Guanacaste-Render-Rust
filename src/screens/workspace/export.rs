//! Export / render mode: output settings on the right; the render header (render, all views) and the real queue
//! in the bottom drawer. The picture of the running job opens over the 3D view while the queue works.

use super::{ModeSlots, Panel, Slot};
use crate::components::icons;
use crate::features::rendering::components::output_panel::{self, PanelAction};
use crate::features::rendering::components::queue_table::{self, QueueAction};
use crate::features::rendering::components::studio_header::{self, HeaderAction};
use crate::features::rendering::engine::ViewPreset;
use crate::features::rendering::logic::plan::plan_from_studio;
use crate::features::rendering::services::{open_folder, safe_stem};
use crate::models::AppState;
use crate::services::localization::{tr, trf};
use eframe::egui::{pos2, Rect, Ui};
use std::path::PathBuf;
use std::time::Duration;

const BOTTOM_HEIGHT: f32 = 300.0;

/// Slots of export mode.
pub fn slots() -> ModeSlots {
    ModeSlots {
        side: Slot {
            title: "panel.output",
            glyph: icons::IMAGE,
            extent: |_| output_panel::WIDTH,
            body: output as Panel,
        },
        bottom: Slot {
            title: "panel.queue",
            glyph: icons::QUEUE,
            extent: |_| BOTTOM_HEIGHT,
            body: queue as Panel,
        },
        overlay: |ui, rect, app| {
            let active = app.render.queue.is_busy() || !app.studio.state.jobs.is_empty();
            if active {
                let view = Rect::from_min_max(
                    rect.min,
                    pos2(
                        rect.right() - output_panel::WIDTH,
                        rect.bottom() - BOTTOM_HEIGHT,
                    ),
                );
                app.studio.preview.show(ui, view, &app.studio.state.jobs);
            }
        },
        rail: None,
        side_left: false,
        plays: false,
    }
}

/// Per-frame work of the mode: queue snapshot, folder picker and live preview.
pub fn update(ui: &Ui, app: &mut AppState) {
    app.studio.state.jobs = app.render.queue.snapshot();
    app.studio.state.saved_cameras = app
        .workspace
        .project
        .as_ref()
        .map(|p| p.document.cameras.iter().map(|c| c.name.clone()).collect())
        .unwrap_or_default();
    if let Some(folder) = app.studio.picker.poll() {
        app.studio.state.output = folder.display().to_string();
    }
    app.studio.preview.update(ui.ctx(), &app.render.queue);
    if app.render.queue.is_busy() || app.studio.picker.is_open() {
        ui.ctx().request_repaint_after(Duration::from_millis(100));
    }
}

fn output(ui: &mut Ui, app: &mut AppState) {
    match output_panel::show(ui, ui.max_rect(), &mut app.studio.state) {
        Some(PanelAction::Browse) => app.studio.picker.open(&app.studio.state.output),
        Some(PanelAction::OpenFolder)
            if !open_folder(&PathBuf::from(app.studio.state.output.trim())) =>
        {
            app.studio.state.notice = Some((tr("render.folder_missing").to_owned(), true));
        }
        Some(PanelAction::OpenFolder) => {}
        None => {}
    }
}

fn queue(ui: &mut Ui, app: &mut AppState) {
    let area = ui.max_rect();
    let ready = app
        .workspace
        .project
        .as_ref()
        .is_some_and(|project| !project.meshes.is_empty());
    let mut header = ui.new_child(eframe::egui::UiBuilder::new().max_rect(area));
    header.spacing_mut().item_spacing.y = 0.0;
    if let Some(action) = studio_header::show(&mut header, &mut app.studio.state, ready) {
        start(app, action);
    }
    let body = Rect::from_min_max(pos2(area.left(), header.min_rect().bottom()), area.max);
    match queue_table::show(ui, body, &app.studio.state) {
        Some(QueueAction::Cancel(id)) => app.render.queue.cancel(id),
        Some(QueueAction::Retry(id)) => app.render.queue.retry(id),
        Some(QueueAction::Remove(id)) => app.render.queue.remove(id),
        Some(QueueAction::ClearFinished) => {
            app.render.queue.clear_finished();
        }
        None => {}
    }
}

/// The photo mode's render button: one picture per saved camera (each with its own filters), or of the 3D view
/// when no camera is saved. The result is reported in the notices.
pub fn render_photos(app: &mut AppState) {
    let plan = plan_from_studio(&app.studio.state);
    if safe_stem(&plan.output.file_name) != plan.output.file_name.trim() {
        app.studio.state.file_name = safe_stem(&plan.output.file_name);
    }
    let plan = plan_from_studio(&app.studio.state);
    let camera = *app.viewport.camera();
    let result = if app.cameras.is_empty() {
        app.render.enqueue_viewport(&app.workspace, &plan, &camera)
    } else {
        app.render.enqueue_saved_cameras(&app.workspace, &plan)
    };
    let (text, error) = match result {
        Ok(count) => (trf("render.queued_n", &[&count.to_string()]), false),
        Err(reason) => (tr(reason).to_owned(), true),
    };
    app.workspace.notify(text.clone(), error);
    app.studio.state.notice = Some((text, error));
}

pub(super) fn start(app: &mut AppState, action: HeaderAction) {
    let plan = plan_from_studio(&app.studio.state);
    if safe_stem(&plan.output.file_name) != plan.output.file_name.trim() {
        app.studio.state.file_name = safe_stem(&plan.output.file_name);
    }
    let views: &[ViewPreset] = match action {
        HeaderAction::Render => &[plan.view],
        HeaderAction::RenderAllViews => &ViewPreset::ALL,
    };
    app.studio.state.notice = Some(match app.render.enqueue(&app.workspace, &plan, views) {
        Ok(count) => (trf("render.queued_n", &[&count.to_string()]), false),
        Err(reason) => (tr(reason).to_owned(), true),
    });
}
