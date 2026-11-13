//! The inspector docked at the top-right of the 3D view: the selected keyframe and the studio camera.

use super::fields::{number, pose_editor};
use crate::components::{
    button::{Button, ButtonStyle},
    form, frames, icons,
    layout::split_row,
    section_title, text,
};
use crate::config::theme::{colors, typography};
use crate::features::movie::models::{
    Animation, CameraKey, Ease, MovieState, Selection, MAX_DURATION,
};
use crate::services::localization::tr;
use eframe::egui::{self, vec2, CornerRadius, Margin, Rect, ScrollArea, Ui, UiBuilder};

/// Panel width in points.
pub const WIDTH: f32 = 288.0;

fn small_button(ui: &mut Ui, label: &'static str, style: ButtonStyle) -> bool {
    Button::new(tr(label), style)
        .font(typography::label_sm())
        .padding(10.0, 4.0)
        .show(ui)
        .clicked()
}

/// Draws the panel with its top-left corner at `origin`, at most `max_height` tall. Returns true when the
/// animation changed.
pub fn show(
    ui: &mut Ui,
    origin: egui::Pos2,
    max_height: f32,
    animation: &mut Animation,
    state: &mut MovieState,
) -> bool {
    let mut changed = false;
    let mut child = ui.new_child(UiBuilder::new().max_rect(Rect::from_min_size(
        origin,
        vec2(WIDTH, max_height.max(120.0)),
    )));
    frames::overlay(0)
        .corner_radius(CornerRadius::same(4))
        .show(&mut child, |ui| {
            ui.set_width(WIDTH);
            ui.spacing_mut().item_spacing.y = 0.0;
            egui::Frame::new()
                .fill(colors::SURFACE_CONTAINER)
                .inner_margin(Margin::symmetric(12, 8))
                .show(ui, |ui| {
                    ui.set_width(WIDTH - 24.0);
                    ui.horizontal(|ui| {
                        ui.spacing_mut().item_spacing.x = 4.0;
                        text::icon(ui, icons::VIDEOCAM, 15.0, colors::PRIMARY);
                        text::label(
                            ui,
                            text::caps(tr("movie.inspector")),
                            typography::label_md(),
                            colors::ON_SURFACE,
                        );
                    });
                });
            ScrollArea::vertical()
                .max_height((max_height - 48.0).max(80.0))
                .auto_shrink([false, true])
                .show(ui, |ui| {
                    egui::Frame::new()
                        .inner_margin(Margin::same(12))
                        .show(ui, |ui| {
                            ui.set_width(WIDTH - 24.0);
                            ui.spacing_mut().item_spacing.y = 8.0;
                            changed |= selected(ui, animation, state);
                            changed |= live_camera(ui, animation, state);
                        });
                });
        });
    changed
}

fn selected(ui: &mut Ui, animation: &mut Animation, state: &mut MovieState) -> bool {
    match state.selection {
        Selection::Camera(index) if index < animation.camera.keys.len() => {
            camera_key(ui, animation, state, index)
        }
        Selection::Value { channel, key }
            if animation
                .channels
                .get(channel)
                .is_some_and(|c| key < c.keys.len()) =>
        {
            value_key(ui, animation, state, channel, key)
        }
        _ => {
            // Wrapping: a long single line would widen the panel and push the right-aligned fields out of view.
            text::paragraph(
                ui,
                tr("movie.nothing_selected"),
                typography::body_sm(),
                colors::OUTLINE,
                16.0,
            );
            false
        }
    }
}

fn ease_picker(ui: &mut Ui, id: &str, ease: &mut Ease) -> bool {
    let labels: Vec<&str> = Ease::ALL.iter().map(|e| tr(e.label_key())).collect();
    let mut index = Ease::ALL.iter().position(|e| e == ease).unwrap_or(0);
    split_row(
        ui,
        26.0,
        |ui| {
            text::label(
                ui,
                tr("movie.ease"),
                typography::body_sm(),
                colors::ON_SURFACE,
            );
        },
        |ui| {
            form::dropdown(ui, id, &mut index, &labels, 120.0);
        },
    );
    let picked = Ease::ALL[index];
    let changed = picked != *ease;
    *ease = picked;
    changed
}

fn camera_key(
    ui: &mut Ui,
    animation: &mut Animation,
    state: &mut MovieState,
    index: usize,
) -> bool {
    section_title::caption(ui, tr("movie.key_camera"), colors::PRIMARY);
    let original = animation.camera.keys[index];
    let mut key = original;
    let mut changed = number(
        ui,
        "movie.time",
        &mut key.time,
        0.02,
        0.0..=MAX_DURATION,
        "movie.unit_s",
    );
    changed |= number(
        ui,
        "movie.pause",
        &mut key.pause,
        0.02,
        0.0..=MAX_DURATION,
        "movie.unit_s",
    );
    changed |= ease_picker(ui, "key-ease", &mut key.ease);
    changed |= pose_editor(ui, &mut key.pose);
    let mut remove = false;
    ui.horizontal_wrapped(|ui| {
        if small_button(ui, "movie.use_live", ButtonStyle::Raised) {
            key.pose = state.live_camera;
            changed = true;
        }
        if small_button(ui, "movie.go_to_key", ButtonStyle::Container) {
            state.time = key.time;
        }
        remove = small_button(ui, "movie.delete_key", ButtonStyle::Container);
    });
    if remove {
        animation.remove_camera_key(index);
        state.selection = Selection::None;
        return true;
    }
    if changed {
        animation.camera.keys[index] = CameraKey {
            time: original.time,
            pose: key.pose.sanitized(),
            ..key
        };
        if key.time != original.time {
            let moved = animation.move_camera_key(index, key.time);
            state.selection = Selection::Camera(moved);
        }
    }
    changed
}

fn value_key(
    ui: &mut Ui,
    animation: &mut Animation,
    state: &mut MovieState,
    channel: usize,
    index: usize,
) -> bool {
    section_title::caption(ui, tr("movie.key_value"), colors::TERTIARY);
    let mut changed = false;
    let mut name = animation.channels[channel].name.clone();
    if form::text_field(ui, &mut name, tr("movie.channel_name"), WIDTH - 24.0).changed() {
        animation.channels[channel].name = name;
        changed = true;
    }
    let original = animation.channels[channel].keys[index];
    let mut key = original;
    let mut time = key.time;
    let time_changed = number(
        ui,
        "movie.time",
        &mut time,
        0.02,
        0.0..=MAX_DURATION,
        "movie.unit_s",
    );
    changed |= number(
        ui,
        "movie.value",
        &mut key.value,
        0.05,
        -1.0e6..=1.0e6,
        "movie.unit_none",
    );
    changed |= ease_picker(ui, "value-ease", &mut key.ease);
    let mut remove = false;
    ui.horizontal_wrapped(|ui| {
        if small_button(ui, "movie.go_to_key", ButtonStyle::Container) {
            state.time = original.time;
        }
        remove = small_button(ui, "movie.delete_key", ButtonStyle::Container);
    });
    if remove {
        animation.remove_value_key(channel, index);
        state.selection = Selection::None;
        return true;
    }
    if changed {
        animation.channels[channel].keys[index] = key;
    }
    if time_changed {
        let moved = animation.move_value_key(channel, index, time);
        state.selection = Selection::Value {
            channel,
            key: moved,
        };
    }
    changed || time_changed
}

fn live_camera(ui: &mut Ui, animation: &mut Animation, state: &mut MovieState) -> bool {
    ui.add_space(4.0);
    section_title::caption(ui, tr("movie.live_camera"), colors::SECONDARY);
    pose_editor(ui, &mut state.live_camera);
    let mut changed = false;
    ui.horizontal_wrapped(|ui| {
        if small_button(ui, "movie.record_key", ButtonStyle::Solid) {
            let index = animation.record_camera(state.time, &state.live_camera);
            state.selection = Selection::Camera(index);
            changed = true;
        }
        if small_button(ui, "movie.take_playhead", ButtonStyle::Container) {
            state.live_camera = animation.sample(state.time);
        }
    });
    changed
}
