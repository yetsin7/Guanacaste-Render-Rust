//! The timeline toolbar: timecode, transport, key recording and the export button.

use crate::components::{
    button::{Button, ButtonStyle},
    icon_button::IconButton,
    icons,
    layout::split_row,
    tag::Tag,
    text,
};
use crate::config::theme::{colors, typography};
use crate::features::movie::logic::{self, timecode};
use crate::features::movie::models::{Animation, MovieState, Selection};
use crate::services::localization::tr;
use eframe::egui::{self, CornerRadius, Margin, Sense, Ui, Vec2};

/// What the user did in the toolbar.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct ToolbarOutcome {
    /// The animation data changed.
    pub changed: bool,
    /// The user asked to export the video.
    pub export: bool,
}

fn divider(ui: &mut Ui) {
    let (bar, _) = ui.allocate_exact_size(Vec2::new(1.0, 16.0), Sense::hover());
    ui.painter().rect_filled(bar, 0.0, colors::OUTLINE_VARIANT);
}

/// Draws the toolbar (40 px tall).
pub fn show(ui: &mut Ui, animation: &mut Animation, state: &mut MovieState) -> ToolbarOutcome {
    let mut outcome = ToolbarOutcome::default();
    egui::Frame::new()
        .fill(colors::SURFACE_CONTAINER)
        .inner_margin(Margin::symmetric(12, 0))
        .show(ui, |ui| {
            split_row(
                ui,
                40.0,
                |ui| {
                    ui.spacing_mut().item_spacing.x = 2.0;
                    Tag::new(timecode(state.time, animation.fps), colors::PRIMARY)
                        .font(typography::label_lg())
                        .part(" / ", colors::OUTLINE)
                        .part(
                            timecode(animation.total_duration(), animation.fps),
                            colors::ON_SURFACE_VARIANT,
                        )
                        .fill(colors::SURFACE_LOWEST)
                        .padding(8.0, 4.0)
                        .show(ui);
                    ui.add_space(14.0);
                    transport(ui, animation, state);
                    ui.add_space(10.0);
                    divider(ui);
                    ui.add_space(10.0);
                    if Button::new(tr("movie.record_key"), ButtonStyle::Raised)
                        .icon(icons::ADD_CIRCLE, 15.0)
                        .font(typography::label_sm())
                        .padding(10.0, 6.0)
                        .show(ui)
                        .clicked()
                    {
                        let index = animation.record_camera(state.time, &state.live_camera);
                        state.selection = Selection::Camera(index);
                        outcome.changed = true;
                    }
                },
                |ui| {
                    if Button::new(tr("movie.export_video"), ButtonStyle::Solid)
                        .icon(icons::MOVIE, 16.0)
                        .font(typography::mono_medium(11.0))
                        .padding(16.0, 6.0)
                        .colors(colors::SECONDARY_CONTAINER, colors::ON_SECONDARY_CONTAINER)
                        .show(ui)
                        .clicked()
                    {
                        outcome.export = true;
                    }
                },
            );
        });
    outcome
}

fn transport(ui: &mut Ui, animation: &mut Animation, state: &mut MovieState) {
    let small = |glyph, tip| {
        IconButton::new(glyph)
            .sizes(16.0, 28.0)
            .fill(None)
            .tooltip(tr(tip))
    };
    if small(icons::FIRST_PAGE, "movie.rewind").show(ui).clicked() {
        state.time = 0.0;
    }
    if small(icons::SKIP_PREVIOUS, "movie.previous_key")
        .show(ui)
        .clicked()
    {
        state.time = logic::previous_key_time(animation, state.time);
    }
    play_button(ui, animation, state);
    if small(icons::SKIP_NEXT, "movie.next_key").show(ui).clicked() {
        state.time = logic::next_key_time(animation, state.time);
    }
    if small(icons::LAST_PAGE, "movie.to_end").show(ui).clicked() {
        state.time = animation.total_duration();
    }
    if small(icons::REPEAT, "movie.loop")
        .active(animation.looping)
        .show(ui)
        .clicked()
    {
        animation.looping = !animation.looping;
    }
}

/// The round play/pause button.
fn play_button(ui: &mut Ui, animation: &Animation, state: &mut MovieState) {
    let (rect, response) = ui.allocate_exact_size(Vec2::splat(32.0), Sense::click());
    let fill = if response.hovered() {
        colors::PRIMARY_CONTAINER
    } else {
        colors::PRIMARY
    };
    ui.painter().rect_filled(rect, CornerRadius::same(16), fill);
    let glyph = if state.playing {
        icons::PAUSE
    } else {
        icons::PLAY_ARROW
    };
    text::paint_icon(ui.painter(), rect.center(), glyph, 18.0, colors::ON_PRIMARY);
    if response.clicked() {
        logic::toggle_play(state, animation);
    }
}
