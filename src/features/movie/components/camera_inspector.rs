//! The physical camera inspector docked at the top-right of the 3D view.

use crate::components::{
    frames, icons,
    layout::split_row,
    segmented::Segmented,
    slider::{Slider, Thumb},
    tag::Tag,
    text, toggle,
};
use crate::config::theme::{colors, typography};
use crate::features::movie::logic;
use crate::features::movie::models::MovieState;
use crate::services::localization::tr;
use eframe::egui::{
    self, pos2, vec2, Color32, CornerRadius, Margin, Rect, Sense, Ui, UiBuilder, Vec2,
};

/// Panel width in points.
pub const WIDTH: f32 = 288.0;
const APERTURES: [&str; 4] = ["f/1.4", "f/2.8", "f/5.6", "f/11"];

/// Draws the panel with its top-left corner at `origin`.
pub fn show(ui: &mut Ui, origin: egui::Pos2, state: &mut MovieState) {
    let mut child =
        ui.new_child(UiBuilder::new().max_rect(Rect::from_min_size(origin, vec2(WIDTH, 600.0))));
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
                    split_row(
                        ui,
                        18.0,
                        |ui| {
                            ui.spacing_mut().item_spacing.x = 4.0;
                            text::icon(ui, icons::VIDEOCAM, 15.0, colors::PRIMARY);
                            text::label(
                                ui,
                                text::caps(tr("movie.inspector")),
                                typography::label_md(),
                                colors::ON_SURFACE,
                            );
                        },
                        |ui| {
                            Tag::new(tr("movie.physical"), colors::PRIMARY)
                                .fill(colors::SURFACE_HIGH)
                                .padding(6.0, 2.0)
                                .show(ui);
                        },
                    );
                });
            egui::Frame::new()
                .inner_margin(Margin::same(12))
                .show(ui, |ui| {
                    ui.set_width(WIDTH - 24.0);
                    ui.spacing_mut().item_spacing.y = 12.0;
                    focal(ui, state);
                    aperture(ui, state);
                    exposure(ui);
                    blur(ui, state);
                    depth(ui, state);
                    atmosphere(ui, state);
                });
        });
}

fn row(ui: &mut Ui, left: &'static str, right: String, color: Color32) {
    split_row(
        ui,
        18.0,
        |ui| {
            text::label(ui, tr(left), typography::body_sm(), colors::ON_SURFACE);
        },
        |ui| {
            text::label(ui, right, typography::mono_medium(11.0), color);
        },
    );
}

fn focal(ui: &mut Ui, state: &mut MovieState) {
    row(
        ui,
        "movie.focal",
        format!("{:.1} mm", logic::focal_length(state.focal)),
        colors::PRIMARY,
    );
    ui.add_space(-6.0);
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 8.0;
        text::label(ui, "18mm", typography::label_sm(), colors::OUTLINE);
        let width = ui.available_width() - 44.0;
        ui.allocate_ui(vec2(width, 14.0), |ui| {
            Slider::bar(6.0)
                .thumb(Thumb::Circle(6.0))
                .colors(colors::PRIMARY, colors::SURFACE_HIGHEST)
                .show(ui, &mut state.focal);
        });
        text::label(ui, "85mm", typography::label_sm(), colors::OUTLINE);
    });
}

fn aperture(ui: &mut Ui, state: &mut MovieState) {
    row(
        ui,
        "movie.aperture",
        APERTURES[state.aperture].to_owned(),
        colors::SECONDARY,
    );
    ui.add_space(-6.0);
    Segmented::text(&APERTURES)
        .container(colors::SURFACE_CONTAINER, 0.0, 4.0)
        .height(24.0)
        .show(ui, &mut state.aperture);
}

fn exposure(ui: &mut Ui) {
    let gap = 8.0;
    let width = (ui.available_width() - gap) / 2.0;
    let (area, _) = ui.allocate_exact_size(vec2(ui.available_width(), 58.0), Sense::hover());
    let cells = [
        (
            "movie.shutter",
            "1/60 s",
            "movie.shutter_note",
            colors::TERTIARY,
            icons::SPEED,
        ),
        (
            "movie.gain",
            "ISO 200",
            "movie.gain_note",
            colors::OUTLINE,
            icons::TUNE,
        ),
    ];
    for (index, (caption, value, note, note_color, glyph)) in cells.into_iter().enumerate() {
        let rect = Rect::from_min_size(
            pos2(area.left() + index as f32 * (width + gap), area.top()),
            Vec2::new(width, 58.0),
        );
        ui.painter()
            .rect_filled(rect, CornerRadius::same(2), colors::SURFACE_CONTAINER);
        text::paint_text(
            ui.painter(),
            rect.min + vec2(8.0, 12.0),
            egui::Align2::LEFT_CENTER,
            &text::caps(tr(caption)),
            typography::label_sm(),
            colors::OUTLINE,
        );
        text::paint_text(
            ui.painter(),
            rect.min + vec2(8.0, 30.0),
            egui::Align2::LEFT_CENTER,
            value,
            typography::label_md(),
            colors::ON_SURFACE,
        );
        text::paint_icon(
            ui.painter(),
            pos2(rect.right() - 14.0, rect.top() + 30.0),
            glyph,
            13.0,
            colors::OUTLINE,
        );
        text::paint_text(
            ui.painter(),
            rect.min + vec2(8.0, 47.0),
            egui::Align2::LEFT_CENTER,
            tr(note),
            typography::mono(9.0),
            note_color,
        );
    }
}

fn blur(ui: &mut Ui, state: &mut MovieState) {
    row(
        ui,
        "movie.blur",
        format!(
            "{:.2} ({})",
            state.motion_blur,
            tr(logic::blur_label(state.motion_blur))
        ),
        colors::ON_SURFACE,
    );
    ui.add_space(-6.0);
    Slider::bar(6.0)
        .thumb(Thumb::Bar(vec2(10.0, 10.0)))
        .colors(colors::PRIMARY_CONTAINER, colors::SURFACE_HIGHEST)
        .show(ui, &mut state.motion_blur);
}

fn depth(ui: &mut Ui, state: &mut MovieState) {
    split_row(
        ui,
        34.0,
        |ui| {
            ui.vertical(|ui| {
                ui.spacing_mut().item_spacing.y = 0.0;
                text::label(
                    ui,
                    tr("movie.dof"),
                    typography::body_sm(),
                    colors::ON_SURFACE,
                );
                text::label(
                    ui,
                    tr("movie.dof_note"),
                    typography::label_sm(),
                    colors::OUTLINE,
                );
            });
        },
        |ui| {
            toggle::show_sized(ui, &mut state.depth_of_field, vec2(36.0, 20.0));
        },
    );
}

fn atmosphere(ui: &mut Ui, state: &mut MovieState) {
    crate::components::section_title::caption(ui, tr("movie.atmosphere"), colors::OUTLINE);
    ui.add_space(-6.0);
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 6.0;
        let width = (ui.available_width() - 6.0) / 2.0;
        for (index, key) in ["movie.bloom", "movie.anamorphic"].into_iter().enumerate() {
            let (rect, response) = ui.allocate_exact_size(vec2(width, 24.0), Sense::click());
            let on = state.atmosphere[index];
            let fill = if on {
                colors::SECONDARY_CONTAINER
            } else if response.hovered() {
                colors::SURFACE_HIGH
            } else {
                colors::SURFACE_CONTAINER
            };
            ui.painter().rect_filled(rect, CornerRadius::same(2), fill);
            text::paint_text(
                ui.painter(),
                rect.center(),
                egui::Align2::CENTER_CENTER,
                tr(key),
                typography::label_sm(),
                colors::ON_SURFACE,
            );
            if response.clicked() {
                state.atmosphere[index] = !on;
            }
        }
    });
}
