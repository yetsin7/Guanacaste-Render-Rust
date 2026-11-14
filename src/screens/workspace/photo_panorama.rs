//! The panorama tab of photo mode: render a 360 x 180 degree equirectangular picture from the eye of the 3D view
//! with the photo engine, save it as a PNG and look around it in a simple viewer.

use crate::components::button::{Button, ButtonStyle};
use crate::components::chip::Chip;
use crate::components::text;
use crate::config::theme::{colors, typography};
use crate::features::rendering::logic::plan::plan_from_studio;
use crate::features::rendering::models::{JobStatus, WIDTHS};
use crate::features::rendering::services::safe_stem;
use crate::models::AppState;
use crate::services::localization::{tr, trf};
use eframe::egui::{Ui, Vec2};
use std::path::PathBuf;

/// The controls of the tab.
pub fn show(ui: &mut Ui, app: &mut AppState) {
    text::paragraph(
        ui,
        tr("pano.intro"),
        typography::body_sm(),
        colors::OUTLINE,
        16.0,
    );
    text::label(
        ui,
        text::caps(tr("pano.size")),
        typography::label_sm(),
        colors::SECONDARY,
    );
    ui.horizontal_wrapped(|ui| {
        ui.spacing_mut().item_spacing = Vec2::splat(4.0);
        for (index, width) in WIDTHS.iter().enumerate() {
            let label = format!("{width}×{}", width / 2);
            if Chip::new(&label)
                .selected(app.studio.panorama.width == index)
                .show(ui)
                .clicked()
            {
                app.studio.panorama.width = index;
            }
        }
    });
    text::paragraph(
        ui,
        tr("pano.quality_note"),
        typography::label_sm(),
        colors::OUTLINE,
        14.0,
    );
    let ready = app
        .workspace
        .project
        .as_ref()
        .is_some_and(|project| !project.meshes.is_empty());
    let busy = app.studio.panorama.job.is_some();
    let mut button = Button::new(tr("pano.render"), ButtonStyle::Solid)
        .colors(colors::TERTIARY, colors::ON_TERTIARY);
    if !ready {
        button = button.disabled(tr("render.no_scene"));
    } else if busy {
        button = button.disabled(tr("pano.busy"));
    }
    if button.show(ui).clicked() {
        start(app);
    }
    if app.studio.panorama.viewer.source.is_some()
        && Button::new(tr("pano.open_viewer"), ButtonStyle::Container)
            .show(ui)
            .clicked()
    {
        app.studio.panorama.viewer.open = true;
    }
    if let Some(path) = app.studio.panorama.viewer.path.clone() {
        text::paragraph(
            ui,
            trf("pano.saved_at", &[&path.display().to_string()]),
            typography::label_sm(),
            colors::OUTLINE,
            14.0,
        );
    }
}

/// Queues a panorama from the 3D view's camera.
fn start(app: &mut AppState) {
    let mut plan = plan_from_studio(&app.studio.state);
    plan.output.file_name = format!("{}_panorama", safe_stem(&plan.output.file_name));
    plan.output.format = crate::features::rendering::models::ImageFormat::Png;
    plan.transparent = false;
    let camera = *app.viewport.camera();
    let width = WIDTHS[app.studio.panorama.width.min(WIDTHS.len() - 1)];
    match app
        .render
        .enqueue_panorama(&app.workspace, &plan, &camera, width)
    {
        Ok(_) => {
            app.studio.panorama.job = app.render.last_job;
            app.workspace.notify(tr("pano.queued").to_owned(), false);
        }
        Err(reason) => app.workspace.notify(tr(reason).to_owned(), true),
    }
}

/// Notices when the panorama job finishes: opens the viewer on the picture, or reports why it failed.
pub fn poll(app: &mut AppState) {
    let Some(id) = app.studio.panorama.job else {
        return;
    };
    let jobs = app.render.queue.snapshot();
    let Some(job) = jobs.iter().find(|job| job.id == id) else {
        app.studio.panorama.job = None;
        return;
    };
    match job.status {
        JobStatus::Done => {
            app.studio.panorama.job = None;
            let path: Option<PathBuf> = job.output.clone();
            if let Some(path) = path {
                match app.studio.panorama.viewer.load(&path) {
                    Ok(()) => app.workspace.notify(tr("pano.done").to_owned(), false),
                    Err(reason) => app.workspace.notify(tr(reason).to_owned(), true),
                }
            }
        }
        JobStatus::Failed | JobStatus::Cancelled => {
            app.studio.panorama.job = None;
            app.workspace.notify(tr("pano.failed").to_owned(), true);
        }
        _ => {}
    }
}
