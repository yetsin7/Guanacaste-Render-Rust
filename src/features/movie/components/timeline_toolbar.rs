//! The timeline toolbar: timecode, transport, keyframe tools and render button.

use crate::components::{
    button::{Button, ButtonStyle},
    icon_button::IconButton,
    icons,
    layout::split_row,
    tag::Tag,
    text,
};
use crate::config::theme::{colors, typography};
use crate::features::movie::logic::timecode;
use crate::features::movie::models::{MovieState, DURATION};
use crate::services::localization::tr;
use eframe::egui::{self, CornerRadius, Margin, Sense, Ui, Vec2};

/// What the user asked for in the toolbar.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ToolbarAction {
    /// Render the current clip.
    RenderClip,
}

fn divider(ui: &mut Ui) {
    let (bar, _) = ui.allocate_exact_size(Vec2::new(1.0, 16.0), Sense::hover());
    ui.painter().rect_filled(bar, 0.0, colors::OUTLINE_VARIANT);
}

/// Draws the toolbar (40 px tall).
pub fn show(ui: &mut Ui, state: &mut MovieState) -> Option<ToolbarAction> {
    let mut action = None;
    egui::Frame::new()
        .fill(colors::SURFACE_CONTAINER)
        .inner_margin(Margin::symmetric(12, 0))
        .show(ui, |ui| {
            split_row(
                ui,
                40.0,
                |ui| {
                    ui.spacing_mut().item_spacing.x = 2.0;
                    Tag::new(timecode(state.time), colors::PRIMARY)
                        .font(typography::label_lg())
                        .part(" / ", colors::OUTLINE)
                        .part(timecode(DURATION), colors::ON_SURFACE_VARIANT)
                        .fill(colors::SURFACE_LOWEST)
                        .padding(8.0, 4.0)
                        .show(ui);
                    ui.add_space(14.0);
                    let small = |glyph| IconButton::new(glyph).sizes(16.0, 28.0).fill(None);
                    if small(icons::FIRST_PAGE).show(ui).clicked() {
                        state.time = 0.0;
                    }
                    if small(icons::SKIP_PREVIOUS).show(ui).clicked() {
                        state.time = (state.time - 1.0).max(0.0);
                    }
                    play_button(ui, state);
                    if small(icons::SKIP_NEXT).show(ui).clicked() {
                        state.time = (state.time + 1.0).min(DURATION);
                    }
                    if small(icons::LAST_PAGE).show(ui).clicked() {
                        state.time = DURATION;
                    }
                    if small(icons::REPEAT)
                        .active(state.looping)
                        .show(ui)
                        .clicked()
                    {
                        state.looping = !state.looping;
                    }
                    ui.add_space(10.0);
                    divider(ui);
                    ui.add_space(10.0);
                    ui.spacing_mut().item_spacing.x = 8.0;
                    if Button::new(tr("movie.add_key"), ButtonStyle::Raised)
                        .icon(icons::ADD_CIRCLE, 15.0)
                        .font(typography::label_sm())
                        .padding(10.0, 6.0)
                        .show(ui)
                        .clicked()
                    {
                        state.added_keyframes += 1;
                    }
                    let spline_style = if state.auto_spline {
                        ButtonStyle::Raised
                    } else {
                        ButtonStyle::Container
                    };
                    if Button::new(tr("movie.auto_spline"), spline_style)
                        .icon(icons::EFFECTS, 14.0)
                        .font(typography::label_sm())
                        .padding(8.0, 6.0)
                        .show(ui)
                        .clicked()
                    {
                        state.auto_spline = !state.auto_spline;
                    }
                },
                |ui| {
                    ui.spacing_mut().item_spacing.x = 8.0;
                    if Button::new(tr("movie.render_clip"), ButtonStyle::Solid)
                        .icon(icons::MOVIE, 16.0)
                        .font(typography::mono_medium(11.0))
                        .padding(16.0, 6.0)
                        .colors(colors::SECONDARY_CONTAINER, colors::ON_SECONDARY_CONTAINER)
                        .show(ui)
                        .clicked()
                    {
                        action = Some(ToolbarAction::RenderClip);
                    }
                    Button::new(tr("movie.curve_graph"), ButtonStyle::Raised)
                        .icon(icons::TUNE, 15.0)
                        .font(typography::label_sm())
                        .padding(12.0, 6.0)
                        .show(ui);
                    Tag::new(tr("movie.curve"), colors::OUTLINE)
                        .part(format!(" {}", tr("movie.curve_value")), colors::TERTIARY)
                        .trail_icon(icons::SHOW_CHART, 14.0, colors::TERTIARY)
                        .gap(6.0)
                        .fill(colors::SURFACE_LOWEST)
                        .padding(8.0, 4.0)
                        .show(ui);
                },
            );
        });
    action
}

/// The round play/pause button.
fn play_button(ui: &mut Ui, state: &mut MovieState) {
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
        if state.time >= DURATION {
            state.time = 0.0;
        }
        state.playing = !state.playing;
    }
}
