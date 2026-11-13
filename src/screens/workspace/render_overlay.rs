//! What floats over the 3D view while the render queue works: the progress bar with its Stop button, the
//! performance card, and the "render completed" notice with its save button. Only real queue data is shown;
//! memory and GPU load are not measured (there is no system sensor), and the card says so.

use crate::components::button::{Button, ButtonStyle};
use crate::components::{frames, icons, progress_bar, sparkline, text};
use crate::config::theme::{colors, typography};
use crate::features::rendering::logic::clock;
use crate::features::rendering::models::{JobInfo, JobStatus};
use crate::features::rendering::services::open_folder;
use crate::models::AppState;
use crate::services::localization::{tr, trf};
use eframe::egui::{self, pos2, vec2, Align2, Area, Id, Order, Rect, Sense};

/// Draws the overlays inside `free` (the part of the window that shows the 3D view).
pub fn show(ctx: &egui::Context, free: Rect, app: &mut AppState) {
    let jobs = app.render.queue.snapshot();
    app.render_watch.update(&jobs);
    if let Some(job) = jobs.iter().find(|job| job.status == JobStatus::Rendering) {
        progress(ctx, free, app, job);
        performance(ctx, free, app, job);
        ctx.request_repaint_after(std::time::Duration::from_millis(250));
    }
    completed(ctx, free, app);
}

fn progress(ctx: &egui::Context, free: Rect, app: &mut AppState, job: &JobInfo) {
    let percent = (job.progress * 100.0).round().to_string();
    let label = trf(
        "overlay.progress",
        &[&job.name, &percent, &clock(job.elapsed)],
    );
    let mut stop = false;
    Area::new(Id::new("render-progress"))
        .order(Order::Middle)
        .pivot(Align2::CENTER_BOTTOM)
        .fixed_pos(pos2(free.center().x, free.bottom() - 14.0))
        .show(ctx, |ui| {
            frames::floating(10).show(ui, |ui| {
                ui.set_width(440.0_f32.min(free.width() - 40.0));
                ui.spacing_mut().item_spacing.y = 6.0;
                ui.horizontal(|ui| {
                    text::label(ui, label, typography::label_md(), colors::ON_SURFACE);
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        stop = Button::new(tr("overlay.stop"), ButtonStyle::Container)
                            .icon(icons::CLOSE, 14.0)
                            .show(ui)
                            .clicked();
                    });
                });
                progress_bar::show(ui, job.progress, colors::PRIMARY, 6.0);
            });
        });
    if stop {
        app.render.queue.cancel(job.id);
    }
}

fn performance(ctx: &egui::Context, free: Rect, app: &AppState, job: &JobInfo) {
    let threads = std::thread::available_parallelism().map_or(1, |n| n.get());
    let remaining = job
        .remaining
        .map_or_else(|| tr("overlay.calculating").to_owned(), clock);
    Area::new(Id::new("render-performance"))
        .order(Order::Middle)
        .pivot(Align2::RIGHT_BOTTOM)
        .fixed_pos(pos2(free.right() - 14.0, free.bottom() - 70.0))
        .show(ctx, |ui| {
            frames::floating(10).show(ui, |ui| {
                ui.set_width(230.0);
                ui.spacing_mut().item_spacing.y = 4.0;
                text::label(
                    ui,
                    text::caps(tr("perf.title")),
                    typography::label_sm(),
                    colors::SECONDARY,
                );
                text::label(
                    ui,
                    trf("perf.threads", &[&threads.to_string()]),
                    typography::body_sm(),
                    colors::ON_SURFACE,
                );
                text::label(
                    ui,
                    trf("perf.elapsed", &[&clock(job.elapsed)]),
                    typography::body_sm(),
                    colors::ON_SURFACE,
                );
                text::label(
                    ui,
                    trf("perf.remaining", &[&remaining]),
                    typography::body_sm(),
                    colors::ON_SURFACE,
                );
                let (rect, _) = ui.allocate_exact_size(vec2(210.0, 34.0), Sense::hover());
                ui.painter().rect_filled(rect, 2.0, colors::SURFACE_LOWEST);
                sparkline::show(
                    ui,
                    rect.shrink(3.0),
                    &app.render_watch.history,
                    colors::TERTIARY,
                    true,
                );
                text::paragraph(
                    ui,
                    tr("perf.note"),
                    typography::label_sm(),
                    colors::OUTLINE,
                    14.0,
                );
            });
        });
}

fn completed(ctx: &egui::Context, free: Rect, app: &mut AppState) {
    let Some(done) = app.render_watch.completed.clone() else {
        return;
    };
    let (mut save, mut close) = (false, false);
    Area::new(Id::new("render-done"))
        .order(Order::Tooltip)
        .pivot(Align2::CENTER_TOP)
        .fixed_pos(pos2(free.center().x, free.top() + 120.0))
        .show(ctx, |ui| {
            frames::floating(12).show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing.x = 10.0;
                    text::icon(ui, icons::CHECK_CIRCLE, 18.0, colors::TERTIARY);
                    text::label(
                        ui,
                        tr("done.title"),
                        typography::label_lg(),
                        colors::ON_SURFACE,
                    );
                    save = Button::new(tr("done.save"), ButtonStyle::Pale)
                        .icon(icons::IMAGE, 16.0)
                        .show(ui)
                        .clicked();
                    close = Button::new(tr("done.close"), ButtonStyle::Quiet)
                        .show(ui)
                        .clicked();
                });
            });
        });
    if save {
        if let Some(folder) = done.output.as_deref().and_then(std::path::Path::parent) {
            open_folder(folder);
        }
    }
    if save || close {
        app.render_watch.completed = None;
    }
    ctx.request_repaint_after(std::time::Duration::from_secs(1));
}
