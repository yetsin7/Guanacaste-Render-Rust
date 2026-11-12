//! The render queue table with progress, status chips and an empty state.

use crate::components::{
    button::{Button, ButtonStyle},
    empty_state,
    icon_button::IconButton,
    icons,
    layout::split_row,
    progress_bar,
    tag::Tag,
    text,
};
use crate::config::theme::{colors, typography};
use crate::features::rendering::logic::{clear_finished, summary};
use crate::features::rendering::models::{JobStatus, RenderJob, RenderStudioState};
use crate::services::localization::{tr, trf};
use crate::services::sample_data::rendering as sample;
use eframe::egui::{
    self, pos2, vec2, Align2, Color32, CornerRadius, Margin, Rect, Sense, Ui, UiBuilder,
};

const COLUMNS: [(&str, f32); 6] = [
    ("queue.col_job", 0.30),
    ("queue.col_preset", 0.20),
    ("queue.col_resolution", 0.12),
    ("queue.col_progress", 0.18),
    ("queue.col_status", 0.10),
    ("queue.col_eta", 0.10),
];

fn status_look(status: JobStatus) -> (Color32, Color32) {
    match status {
        JobStatus::Queued => (colors::SURFACE_HIGHEST, colors::ON_SURFACE_VARIANT),
        JobStatus::Rendering => (
            colors::with_alpha(colors::PRIMARY_CONTAINER, 51),
            colors::PRIMARY,
        ),
        JobStatus::Done => (colors::with_alpha(colors::TERTIARY, 38), colors::TERTIARY),
        JobStatus::Failed => (colors::with_alpha(colors::ERROR, 38), colors::ERROR),
        JobStatus::Cancelled => (colors::SURFACE_HIGHEST, colors::OUTLINE),
    }
}

/// Draws the table inside `area`.
pub fn show(ui: &mut Ui, area: Rect, state: &mut RenderStudioState) {
    let mut inner = ui.new_child(UiBuilder::new().max_rect(area));
    egui::Frame::new()
        .inner_margin(Margin::same(16))
        .show(&mut inner, |ui| {
            ui.set_width(area.width() - 32.0);
            ui.spacing_mut().item_spacing.y = 8.0;
            let (queued, rendering, finished) = summary(&state.jobs);
            let mut clear = false;
            split_row(
                ui,
                28.0,
                |ui| {
                    ui.spacing_mut().item_spacing.x = 8.0;
                    for (key, count, color) in [
                        ("queue.queued", queued, colors::ON_SURFACE_VARIANT),
                        ("queue.rendering", rendering, colors::PRIMARY),
                        ("queue.finished", finished, colors::TERTIARY),
                    ] {
                        Tag::new(trf(key, &[&count.to_string()]), color)
                            .fill(colors::SURFACE_CONTAINER)
                            .padding(8.0, 4.0)
                            .show(ui);
                    }
                },
                |ui| {
                    ui.spacing_mut().item_spacing.x = 8.0;
                    clear = Button::new(tr("queue.clear"), ButtonStyle::Container)
                        .icon(icons::DELETE, 14.0)
                        .font(typography::label_sm())
                        .padding(10.0, 5.0)
                        .show(ui)
                        .clicked();
                },
            );
            if clear {
                clear_finished(&mut state.jobs);
            }
            header(ui);
            if state.jobs.is_empty()
                && empty_state::show(
                    ui,
                    icons::QUEUE,
                    tr("queue.empty_title"),
                    tr("queue.empty_body"),
                    Some(tr("queue.restore")),
                )
            {
                state.jobs = sample::jobs();
            }
            for job in &state.jobs {
                row(ui, job);
            }
        });
}

fn header(ui: &mut Ui) {
    let (rect, _) = ui.allocate_exact_size(vec2(ui.available_width(), 24.0), Sense::hover());
    ui.painter()
        .rect_filled(rect, CornerRadius::same(2), colors::SURFACE_HIGH);
    let mut x = rect.left() + 12.0;
    let width = rect.width() - 24.0 - 64.0;
    for (key, fraction) in COLUMNS {
        text::paint_text(
            ui.painter(),
            pos2(x, rect.center().y),
            Align2::LEFT_CENTER,
            &text::caps(tr(key)),
            typography::mono(10.0),
            colors::OUTLINE,
        );
        x += width * fraction;
    }
}

fn row(ui: &mut Ui, job: &RenderJob) {
    let (rect, response) = ui.allocate_exact_size(vec2(ui.available_width(), 48.0), Sense::hover());
    let fill = if response.hovered() {
        colors::SURFACE_HIGH
    } else {
        colors::SURFACE_CONTAINER
    };
    ui.painter().rect_filled(rect, CornerRadius::same(2), fill);
    let width = rect.width() - 24.0 - 64.0;
    let mut x = rect.left() + 12.0;
    let cells = [job.name, job.preset, job.resolution];
    for (index, value) in cells.into_iter().enumerate() {
        let font = if index == 0 {
            typography::mono_medium(11.0)
        } else {
            typography::label_sm()
        };
        let color = if index == 0 {
            colors::ON_SURFACE
        } else {
            colors::ON_SURFACE_VARIANT
        };
        let cell = Rect::from_min_size(
            pos2(x, rect.top()),
            vec2(width * COLUMNS[index].1 - 8.0, rect.height()),
        );
        let painter = ui.painter().with_clip_rect(cell);
        crate::components::text_layout::paint_left(
            &painter,
            pos2(cell.left(), cell.center().y),
            value,
            font,
            color,
        );
        x += width * COLUMNS[index].1;
    }
    let progress = Rect::from_min_size(
        pos2(x, rect.center().y - 7.0),
        vec2(width * COLUMNS[3].1 - 16.0, 14.0),
    );
    let mut bar = ui.new_child(UiBuilder::new().max_rect(progress));
    let (_, ink) = status_look(job.status);
    progress_bar::show(&mut bar, job.progress, ink, 6.0);
    text::paint_text(
        ui.painter(),
        pos2(progress.left(), progress.bottom() + 2.0),
        Align2::LEFT_CENTER,
        &format!("{:.0}%", job.progress * 100.0),
        typography::mono(9.0),
        colors::OUTLINE,
    );
    x += width * COLUMNS[3].1;
    let (fill, ink) = status_look(job.status);
    let mut chip = ui.new_child(UiBuilder::new().max_rect(Rect::from_min_size(
        pos2(x, rect.center().y - 10.0),
        vec2(width * COLUMNS[4].1, 20.0),
    )));
    Tag::new(tr(job.status.label_key()), ink)
        .fill(fill)
        .padding(6.0, 2.0)
        .show(&mut chip);
    x += width * COLUMNS[4].1;
    let eta_clip = Rect::from_min_max(
        pos2(x, rect.top()),
        pos2(rect.right() - 64.0, rect.bottom()),
    );
    crate::components::text_layout::paint_left(
        &ui.painter().with_clip_rect(eta_clip),
        pos2(x, rect.center().y),
        job.eta,
        typography::mono(10.0),
        colors::OUTLINE,
    );
    let mut actions = ui.new_child(UiBuilder::new().max_rect(Rect::from_min_size(
        pos2(rect.right() - 60.0, rect.center().y - 12.0),
        vec2(56.0, 24.0),
    )));
    actions.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 2.0;
        let running = !job.status.is_finished();
        let glyph = if running {
            icons::PAUSE
        } else {
            icons::REFRESH
        };
        IconButton::new(glyph)
            .sizes(14.0, 24.0)
            .fill(None)
            .tooltip(tr("dialog.no_engine"))
            .show(ui);
        IconButton::new(icons::CLOSE)
            .sizes(14.0, 24.0)
            .fill(None)
            .tooltip(tr("dialog.no_engine"))
            .show(ui);
    });
}
