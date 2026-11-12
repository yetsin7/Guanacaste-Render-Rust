//! The clip card over the 3D view: title, description, node chips and actions.

use crate::components::{
    button::{Button, ButtonStyle},
    frames, icons,
    layout::split_row,
    tag::Tag,
    text,
};
use crate::config::theme::{colors, typography};
use crate::services::localization::tr;
use eframe::egui::{self, pos2, vec2, CornerRadius, Pos2, Rect, Sense, Ui, UiBuilder};

const NODES: [(char, egui::Color32, &str, &str); 4] = [
    (
        icons::TIMELINE,
        colors::PRIMARY,
        "movie.node_path",
        "movie.node_path_note",
    ),
    (
        icons::SUN,
        colors::TERTIARY,
        "movie.node_sun",
        "movie.node_sun_note",
    ),
    (
        icons::CAMERA,
        colors::SECONDARY,
        "movie.node_focal",
        "movie.node_focal_note",
    ),
    (
        icons::TIMELINE,
        colors::PRIMARY_FIXED,
        "movie.node_speed",
        "movie.node_speed_note",
    ),
];

/// Draws the card with its top-left corner at `origin`.
pub fn show(ui: &mut Ui, origin: Pos2) {
    let mut child =
        ui.new_child(UiBuilder::new().max_rect(Rect::from_min_size(origin, vec2(320.0, 400.0))));
    frames::overlay(12)
        .corner_radius(CornerRadius::same(4))
        .show(&mut child, |ui| {
            ui.set_width(296.0);
            ui.spacing_mut().item_spacing.y = 8.0;
            split_row(
                ui,
                20.0,
                |ui| {
                    Tag::new(tr("movie.clip"), colors::PRIMARY)
                        .fill(colors::with_alpha(colors::PRIMARY_CONTAINER, 51))
                        .padding(8.0, 2.0)
                        .show(ui);
                },
                |ui| {
                    Tag::new("00:14.00", colors::ON_SURFACE_VARIANT)
                        .lead_icon(icons::SCHEDULE, 13.0, colors::PRIMARY)
                        .gap(4.0)
                        .padding(0.0, 0.0)
                        .show(ui);
                },
            );
            ui.vertical(|ui| {
                ui.spacing_mut().item_spacing.y = 2.0;
                text::label(
                    ui,
                    tr("movie.clip_title"),
                    typography::headline_md(),
                    colors::ON_SURFACE,
                );
                text::paragraph(
                    ui,
                    tr("movie.clip_body"),
                    typography::body_sm(),
                    colors::ON_SURFACE_VARIANT,
                    19.5,
                );
            });
            nodes(ui);
            split_row(
                ui,
                24.0,
                |ui| {
                    text::label(
                        ui,
                        tr("movie.samples"),
                        typography::label_sm(),
                        colors::OUTLINE,
                    );
                },
                |ui| {
                    ui.spacing_mut().item_spacing.x = 8.0;
                    Button::new(tr("movie.update_node"), ButtonStyle::Pale)
                        .font(typography::mono_medium(11.0))
                        .padding(8.0, 2.0)
                        .show(ui);
                    Button::new(tr("movie.duplicate"), ButtonStyle::Container)
                        .font(typography::label_sm())
                        .padding(8.0, 2.0)
                        .show(ui);
                },
            );
        });
}

fn nodes(ui: &mut Ui) {
    let gap = 4.0;
    let width = (ui.available_width() - gap) / 2.0;
    let (area, _) =
        ui.allocate_exact_size(vec2(ui.available_width(), 2.0 * 38.0 + gap), Sense::hover());
    for (index, (glyph, color, title, note)) in NODES.into_iter().enumerate() {
        let rect = Rect::from_min_size(
            pos2(
                area.left() + (index % 2) as f32 * (width + gap),
                area.top() + (index / 2) as f32 * (38.0 + gap),
            ),
            vec2(width, 38.0),
        );
        ui.painter()
            .rect_filled(rect, CornerRadius::same(2), colors::SURFACE_CONTAINER);
        text::paint_icon(
            ui.painter(),
            pos2(rect.left() + 14.0, rect.center().y),
            glyph,
            14.0,
            color,
        );
        let mut text_ui = ui.new_child(UiBuilder::new().max_rect(Rect::from_min_max(
            pos2(rect.left() + 28.0, rect.top() + 4.0),
            rect.max,
        )));
        text_ui.spacing_mut().item_spacing.y = 0.0;
        crate::components::text_layout::clamped(
            &mut text_ui,
            tr(title),
            typography::label_sm(),
            colors::ON_SURFACE,
            width - 32.0,
            1,
            14.0,
        );
        text::label(
            &mut text_ui,
            tr(note),
            typography::mono(9.0),
            colors::OUTLINE,
        );
    }
}
