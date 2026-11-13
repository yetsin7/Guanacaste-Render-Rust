//! The sequence card over the 3D view: duration, frame rate, speed, loop and how the path is interpolated.

use super::fields::number;
use crate::components::{
    form, frames, icons, layout::split_row, segmented::Segmented, tag::Tag, text, toggle,
};
use crate::config::theme::{colors, typography};
use crate::features::movie::logic::timecode;
use crate::features::movie::models::{
    Animation, PathMode, Timing, FPS_RANGE, MAX_DURATION, SPEED_RANGE,
};
use crate::services::localization::{tr, trf};
use eframe::egui::{vec2, CornerRadius, Pos2, Rect, Ui, UiBuilder};

/// Card width in points.
pub const WIDTH: f32 = 296.0;

/// Draws the card with its top-left corner at `origin`; returns true when the animation changed.
pub fn show(ui: &mut Ui, origin: Pos2, animation: &mut Animation) -> bool {
    let mut changed = false;
    let mut child = ui.new_child(
        UiBuilder::new().max_rect(Rect::from_min_size(origin, vec2(WIDTH + 24.0, 400.0))),
    );
    frames::overlay(12)
        .corner_radius(CornerRadius::same(4))
        .show(&mut child, |ui| {
            ui.set_width(WIDTH);
            ui.spacing_mut().item_spacing.y = 8.0;
            split_row(
                ui,
                20.0,
                |ui| {
                    Tag::new(text::caps(tr("movie.sequence_card")), colors::PRIMARY)
                        .lead_icon(icons::TIMELINE, 13.0, colors::PRIMARY)
                        .gap(4.0)
                        .fill(colors::SURFACE_HIGH)
                        .padding(6.0, 2.0)
                        .show(ui);
                },
                |ui| {
                    text::label(
                        ui,
                        trf(
                            "movie.summary",
                            &[
                                &timecode(animation.total_duration(), animation.fps),
                                &animation.frame_count().to_string(),
                            ],
                        ),
                        typography::mono(10.0),
                        colors::OUTLINE,
                    );
                },
            );
            changed |= name(ui, animation);
            changed |= numbers(ui, animation);
            changed |= modes(ui, animation);
        });
    changed
}

fn name(ui: &mut Ui, animation: &mut Animation) -> bool {
    form::text_field(ui, &mut animation.name, tr("movie.sequence_name"), WIDTH).changed()
}

fn numbers(ui: &mut Ui, animation: &mut Animation) -> bool {
    let mut changed = number(
        ui,
        "movie.duration",
        &mut animation.duration,
        0.1,
        0.0..=MAX_DURATION,
        "movie.unit_s",
    );
    let mut fps = f64::from(animation.fps);
    if number(
        ui,
        "movie.fps",
        &mut fps,
        0.2,
        f64::from(FPS_RANGE.0)..=f64::from(FPS_RANGE.1),
        "movie.unit_fps",
    ) {
        animation.fps = fps.round() as u32;
        changed = true;
    }
    changed |= number(
        ui,
        "movie.speed",
        &mut animation.speed,
        0.02,
        SPEED_RANGE.0..=SPEED_RANGE.1,
        "movie.unit_times",
    );
    split_row(
        ui,
        24.0,
        |ui| {
            text::label(
                ui,
                tr("movie.loop"),
                typography::body_sm(),
                colors::ON_SURFACE,
            );
        },
        |ui| {
            let before = animation.looping;
            toggle::show_sized(ui, &mut animation.looping, vec2(36.0, 20.0));
            changed |= before != animation.looping;
        },
    );
    changed
}

fn modes(ui: &mut Ui, animation: &mut Animation) -> bool {
    let mut changed = false;
    let mut path = usize::from(animation.camera.path == PathMode::Linear);
    let labels = [tr("movie.path_smooth"), tr("movie.path_linear")];
    if Segmented::text(&labels).height(24.0).show(ui, &mut path) {
        animation.camera.path = if path == 1 {
            PathMode::Linear
        } else {
            PathMode::Smooth
        };
        changed = true;
    }
    let mut timing = usize::from(animation.camera.timing == Timing::ConstantSpeed);
    let labels = [tr("movie.timing_keyed"), tr("movie.timing_constant")];
    if Segmented::text(&labels).height(24.0).show(ui, &mut timing) {
        animation.camera.timing = if timing == 1 {
            Timing::ConstantSpeed
        } else {
            Timing::Keyed
        };
        changed = true;
    }
    changed
}
