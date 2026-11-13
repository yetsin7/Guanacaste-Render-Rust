//! The timeline: track list, time ruler, lanes and playhead.

use super::timeline_lanes::{self as lanes, Axis, Quantity};
use crate::components::{icon_button::IconButton, icons, text};
use crate::config::theme::{colors, typography};
use crate::features::movie::logic;
use crate::features::movie::models::{
    Animation, MovieState, Selection, CHANNEL_CLOUDS, CHANNEL_TIME_OF_DAY,
};
use crate::services::localization::tr;
use eframe::egui::{
    self, pos2, vec2, Align2, CornerRadius, Id, Rect, ScrollArea, Sense, Stroke, Ui, UiBuilder,
};

const LIST_WIDTH: f32 = 192.0;
const RULER: f32 = 24.0;
const ROW: f32 = 28.0;
const GAP: f32 = 6.0;
const MARGIN: f32 = 16.0;
const TICK_STEPS: [f64; 9] = [0.1, 0.25, 0.5, 1.0, 2.0, 5.0, 10.0, 30.0, 60.0];

const HEIGHT: Quantity = Quantity {
    color: colors::TERTIARY,
    read: |pose| pose.position[2],
    caption: "movie.height_now",
    unit: "movie.unit_m",
};
const FOCAL: Quantity = Quantity {
    color: colors::SECONDARY,
    read: |pose| pose.focal_mm(),
    caption: "movie.focal_now",
    unit: "movie.unit_mm",
};

/// Distance between ruler ticks that keeps about eight labels visible.
fn tick_step(span: f64) -> f64 {
    TICK_STEPS
        .into_iter()
        .find(|step| span / step <= 10.0)
        .unwrap_or(60.0)
}

/// Draws the list and the lanes inside `area` (below the toolbar); returns true when the animation changed.
pub fn show(ui: &mut Ui, area: Rect, animation: &mut Animation, state: &mut MovieState) -> bool {
    let list = Rect::from_min_size(area.min, vec2(LIST_WIDTH, area.height()));
    let lane = Rect::from_min_max(pos2(list.right(), area.top()), area.max);
    ui.painter()
        .rect_filled(list, 0.0, colors::SURFACE_CONTAINER);
    ui.painter().rect_filled(lane, 0.0, colors::SURFACE_LOWEST);
    let axis = Axis {
        left: lane.left() + MARGIN,
        right: lane.right() - MARGIN,
        span: animation.total_duration().max(1.0),
    };
    let mut changed = header(ui, list, animation, state);
    ruler(
        ui,
        Rect::from_min_size(lane.min, vec2(lane.width(), RULER)),
        &axis,
        animation,
        state,
    );
    let rows = Rect::from_min_max(pos2(area.left(), area.top() + RULER), area.max);
    changed |= scroll_rows(ui, rows, &axis, animation, state);
    playhead(ui, lane, &axis, state);
    changed
}

fn header(ui: &mut Ui, list: Rect, animation: &mut Animation, state: &mut MovieState) -> bool {
    let rect = Rect::from_min_size(list.min, vec2(list.width(), RULER));
    ui.painter().rect_filled(rect, 0.0, colors::SURFACE_HIGH);
    text::paint_text(
        ui.painter(),
        pos2(rect.left() + 12.0, rect.center().y),
        Align2::LEFT_CENTER,
        &text::caps(tr("movie.tracks")),
        typography::label_sm(),
        colors::OUTLINE,
    );
    let button =
        Rect::from_center_size(pos2(rect.right() - 14.0, rect.center().y), vec2(20.0, 20.0));
    let mut child = ui.new_child(UiBuilder::new().max_rect(button));
    let add = IconButton::new(icons::ADD)
        .sizes(14.0, 20.0)
        .fill(None)
        .tooltip(tr("movie.add_channel"))
        .show(&mut child);
    if add.clicked() {
        let preferred = [CHANNEL_TIME_OF_DAY, CHANNEL_CLOUDS]
            .into_iter()
            .find(|name| animation.channels.iter().all(|c| c.name != *name))
            .unwrap_or("canal");
        animation.add_channel(preferred);
        state.selection = Selection::None;
        return true;
    }
    false
}

fn ruler(ui: &mut Ui, rect: Rect, axis: &Axis, animation: &Animation, state: &mut MovieState) {
    ui.painter()
        .rect_filled(rect, 0.0, colors::SURFACE_CONTAINER);
    let step = tick_step(axis.span);
    let mut time = 0.0;
    while time <= axis.span + 1e-6 {
        let x = axis.x_of(time);
        let anchor = if x > rect.right() - 40.0 {
            Align2::RIGHT_CENTER
        } else {
            Align2::LEFT_CENTER
        };
        text::paint_text(
            ui.painter(),
            pos2(x + 2.0, rect.center().y - 2.0),
            anchor,
            &logic::timecode(time, animation.fps),
            typography::mono(10.0),
            colors::OUTLINE,
        );
        ui.painter().line_segment(
            [pos2(x, rect.bottom() - 6.0), pos2(x, rect.bottom())],
            Stroke::new(1.0, colors::with_alpha(colors::OUTLINE, 80)),
        );
        time += step;
    }
    let response = ui.interact(rect, Id::new("timeline-ruler"), Sense::click_and_drag());
    if let Some(pointer) = response.interact_pointer_pos() {
        let step = 1.0 / f64::from(animation.fps.max(1));
        let time = (axis.time_at(pointer.x).clamp(0.0, axis.span) / step).round() * step;
        state.time = time.min(animation.total_duration());
        state.playing = false;
    }
}

fn scroll_rows(
    ui: &mut Ui,
    rows: Rect,
    axis: &Axis,
    animation: &mut Animation,
    state: &mut MovieState,
) -> bool {
    let mut changed = false;
    let mut removed = None;
    let mut child = ui.new_child(UiBuilder::new().max_rect(rows));
    ScrollArea::vertical()
        .auto_shrink([false, false])
        .show(&mut child, |ui| {
            let count = 3 + animation.channels.len();
            let height = GAP + count as f32 * (ROW + GAP);
            let (content, _) = ui.allocate_exact_size(
                vec2(rows.width(), height.max(rows.height())),
                Sense::hover(),
            );
            lanes::grid(
                &ui.painter().with_clip_rect(rows),
                content,
                axis,
                tick_step(axis.span),
            );
            let strip_of = |index: usize| {
                let top = content.top() + GAP + index as f32 * (ROW + GAP);
                Rect::from_min_max(pos2(axis.left, top), pos2(axis.right, top + ROW))
            };
            let cell_of = |index: usize| {
                let top = content.top() + GAP + index as f32 * (ROW + GAP);
                Rect::from_min_size(
                    pos2(content.left() + 8.0, top),
                    vec2(LIST_WIDTH - 16.0, ROW),
                )
            };
            let names: Vec<(String, egui::Color32)> = [
                (tr("movie.track_camera").to_owned(), colors::PRIMARY),
                (tr("movie.track_height").to_owned(), colors::TERTIARY),
                (tr("movie.track_focal").to_owned(), colors::SECONDARY),
            ]
            .into_iter()
            .chain(
                animation
                    .channels
                    .iter()
                    .map(|c| (c.name.clone(), colors::PRIMARY_FIXED)),
            )
            .collect();
            for (index, (name, color)) in names.iter().enumerate() {
                track_cell(ui, cell_of(index), name, *color);
                if index >= 3 && delete_button(ui, cell_of(index), index) {
                    removed = Some(index - 3);
                }
            }
            changed |= lanes::camera_row(ui, strip_of(0), axis, animation, state);
            lanes::camera_curve_row(ui, strip_of(1), axis, animation, (&HEIGHT, state.time));
            lanes::camera_curve_row(ui, strip_of(2), axis, animation, (&FOCAL, state.time));
            for channel in 0..animation.channels.len() {
                changed |=
                    lanes::channel_row(ui, strip_of(3 + channel), axis, animation, state, channel);
            }
        });
    if let Some(channel) = removed {
        animation.remove_channel(channel);
        state.selection = Selection::None;
        changed = true;
    }
    changed
}

fn track_cell(ui: &mut Ui, rect: Rect, name: &str, color: egui::Color32) {
    ui.painter()
        .rect_filled(rect, CornerRadius::same(2), colors::SURFACE_LOW);
    ui.painter()
        .circle_filled(pos2(rect.left() + 12.0, rect.center().y), 4.0, color);
    text::paint_text(
        ui.painter(),
        pos2(rect.left() + 24.0, rect.center().y),
        Align2::LEFT_CENTER,
        name,
        typography::label_sm(),
        colors::ON_SURFACE,
    );
}

fn delete_button(ui: &mut Ui, cell: Rect, index: usize) -> bool {
    let rect = Rect::from_center_size(pos2(cell.right() - 14.0, cell.center().y), vec2(20.0, 20.0));
    let response = ui.interact(rect, Id::new(("delete-channel", index)), Sense::click());
    let ink = if response.hovered() {
        colors::ON_SURFACE
    } else {
        colors::OUTLINE
    };
    text::paint_icon(ui.painter(), rect.center(), icons::DELETE, 13.0, ink);
    response.on_hover_text(tr("movie.delete_channel")).clicked()
}

fn playhead(ui: &mut Ui, lane: Rect, axis: &Axis, state: &MovieState) {
    let x = axis.x_of(state.time.clamp(0.0, axis.span));
    let painter = ui.painter().with_clip_rect(lane);
    painter.line_segment(
        [pos2(x, lane.top() + RULER - 10.0), pos2(x, lane.bottom())],
        Stroke::new(1.0, colors::PRIMARY),
    );
    let head = Rect::from_min_size(pos2(x - 5.0, lane.top() + RULER - 12.0), vec2(10.0, 12.0));
    painter.rect_filled(
        head,
        CornerRadius {
            nw: 0,
            ne: 0,
            sw: 3,
            se: 3,
        },
        colors::PRIMARY,
    );
    painter.circle_filled(head.center(), 2.0, colors::ON_PRIMARY);
}
