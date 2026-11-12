//! The object-properties panel docked at the right of the 3D view.

use crate::components::{
    frames, icon_button::IconButton, icons, layout::split_row, paint, section_title,
    segmented::Segmented, slider::Slider, tag::Tag, text, toggle,
};
use crate::config::theme::{colors, typography};
use crate::features::build::logic;
use crate::features::build::models::BuildState;
use crate::services::localization::tr;
use crate::services::sample_data::build as sample;
use crate::services::textures::{Placeholder, Textures};
use eframe::egui::{
    self, pos2, vec2, CornerRadius, Margin, Rect, ScrollArea, Sense, Ui, UiBuilder,
};

/// Panel width in points.
pub const WIDTH: f32 = 280.0;

/// Draws the panel with its top-left corner at `origin`, limited to `max_height`.
pub fn show(
    ui: &mut Ui,
    origin: egui::Pos2,
    max_height: f32,
    state: &mut BuildState,
    textures: &mut Textures,
) {
    let area = Rect::from_min_size(origin, vec2(WIDTH, max_height));
    let mut child = ui.new_child(UiBuilder::new().max_rect(area));
    frames::overlay(0).show(&mut child, |ui| {
        ui.set_width(WIDTH);
        ui.spacing_mut().item_spacing.y = 0.0;
        header(ui);
        ScrollArea::vertical()
            .max_height(max_height - 48.0)
            .show(ui, |ui| {
                egui::Frame::new()
                    .inner_margin(Margin::same(12))
                    .show(ui, |ui| {
                        ui.set_width(WIDTH - 24.0);
                        ui.spacing_mut().item_spacing.y = 16.0;
                        preview(ui, textures);
                        switches(ui, state);
                        sliders(ui, state);
                        options(ui, state);
                        gizmo(ui);
                    });
            });
    });
}

fn header(ui: &mut Ui) {
    egui::Frame::new()
        .fill(colors::SURFACE_CONTAINER)
        .inner_margin(Margin::same(12))
        .show(ui, |ui| {
            ui.set_width(WIDTH - 24.0);
            split_row(
                ui,
                24.0,
                |ui| {
                    ui.spacing_mut().item_spacing.x = 6.0;
                    text::icon(ui, icons::TUNE, 18.0, colors::PRIMARY);
                    text::label(
                        ui,
                        tr("build.properties"),
                        typography::headline_sm(),
                        colors::ON_SURFACE,
                    );
                },
                |ui| {
                    ui.spacing_mut().item_spacing.x = 2.0;
                    for (glyph, tip) in [
                        (icons::TARGET, "build.locate"),
                        (icons::COPY, "build.duplicate"),
                        (icons::LOCK, "build.lock"),
                    ] {
                        IconButton::new(glyph)
                            .sizes(15.0, 24.0)
                            .fill(None)
                            .tooltip(tr(tip))
                            .show(ui);
                    }
                },
            );
        });
}

/// The material preview with its translucency readout.
fn preview(ui: &mut Ui, textures: &mut Textures) {
    let (rect, _) = ui.allocate_exact_size(vec2(ui.available_width(), 112.0), Sense::hover());
    let painter = ui.painter().with_clip_rect(rect);
    painter.rect_filled(rect, CornerRadius::same(2), colors::SURFACE_LOWEST);
    let texture = textures.get(ui.ctx(), Placeholder::ElmSpecimen).clone();
    paint::image_cover(
        &painter,
        rect,
        &texture,
        vec2(0.5, 0.25),
        colors::with_alpha(egui::Color32::WHITE, 102),
    );
    let mut top = ui.new_child(UiBuilder::new().max_rect(rect.shrink(8.0)));
    split_row(
        &mut top,
        20.0,
        |ui| {
            Tag::new(tr("build.foliage"), colors::ON_SURFACE)
                .fill(colors::with_alpha(colors::SURFACE_LOWEST, 204))
                .padding(6.0, 2.0)
                .show(ui);
        },
        |ui| {
            text::label(ui, "SSS: 0.42", typography::label_sm(), colors::TERTIARY);
        },
    );
    let strip = Rect::from_min_size(
        pos2(rect.left() + 8.0, rect.bottom() - 8.0 - 28.0),
        vec2(rect.width() - 16.0, 28.0),
    );
    painter.rect_filled(
        strip,
        CornerRadius::same(2),
        colors::with_alpha(colors::SURFACE_LOW, 217),
    );
    let mut bottom = ui.new_child(UiBuilder::new().max_rect(strip.shrink2(vec2(6.0, 0.0))));
    split_row(
        &mut bottom,
        28.0,
        |ui| {
            ui.spacing_mut().item_spacing.x = 6.0;
            text::icon(ui, icons::SUN, 14.0, colors::SECONDARY);
            text::label(
                ui,
                tr("build.translucency"),
                typography::label_sm(),
                colors::ON_SURFACE,
            );
        },
        |ui| {
            text::label(ui, "85%", typography::mono_medium(11.0), colors::PRIMARY);
        },
    );
}

fn switches(ui: &mut Ui, state: &mut BuildState) {
    egui::Frame::new()
        .fill(colors::with_alpha(colors::SURFACE_HIGH, 102))
        .corner_radius(CornerRadius::same(2))
        .inner_margin(Margin::same(8))
        .show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.spacing_mut().item_spacing.y = 8.0;
            for (key, value) in [
                ("build.enabled", &mut state.enabled),
                ("build.marker", &mut state.show_marker),
            ] {
                split_row(
                    ui,
                    16.0,
                    |ui| {
                        text::label(ui, tr(key), typography::body_sm(), colors::ON_SURFACE);
                    },
                    |ui| {
                        toggle::show(ui, value);
                    },
                );
            }
        });
}

fn sliders(ui: &mut Ui, state: &mut BuildState) {
    ui.vertical(|ui| {
        ui.spacing_mut().item_spacing.y = 12.0;
        let specs = [
            ("build.brightness", sample::BRIGHTNESS),
            ("build.canopy", sample::CANOPY_HEIGHT),
            ("build.spread", sample::SPREAD_ANGLE),
        ];
        for (index, (key, spec)) in specs.into_iter().enumerate() {
            ui.vertical(|ui| {
                ui.spacing_mut().item_spacing.y = 6.0;
                split_row(
                    ui,
                    14.0,
                    |ui| {
                        text::label(
                            ui,
                            text::caps(tr(key)),
                            typography::label_sm(),
                            colors::ON_SURFACE_VARIANT,
                        );
                    },
                    |ui| {
                        text::label(
                            ui,
                            logic::format_value(&spec, state.sliders[index]),
                            typography::mono_medium(11.0),
                            colors::ON_SURFACE,
                        );
                    },
                );
                Slider::bar(8.0).show(ui, &mut state.sliders[index]);
            });
        }
    });
}

fn options(ui: &mut Ui, state: &mut BuildState) {
    for (caption, labels, selected) in [
        (
            "build.scale_variation",
            [tr("build.off"), tr("build.on"), tr("build.random")],
            &mut state.scale_variation,
        ),
        (
            "build.lod",
            [
                tr("build.lod_medium"),
                tr("build.lod_high"),
                tr("build.lod_dynamic"),
            ],
            &mut state.lod,
        ),
    ] {
        ui.vertical(|ui| {
            ui.spacing_mut().item_spacing.y = 6.0;
            section_title::caption(ui, tr(caption), colors::ON_SURFACE_VARIANT);
            Segmented::text(&labels).height(28.0).show(ui, selected);
        });
    }
}

fn gizmo(ui: &mut Ui) {
    egui::Frame::new()
        .fill(colors::with_alpha(colors::SURFACE_HIGH, 153))
        .corner_radius(CornerRadius::same(2))
        .inner_margin(Margin::same(8))
        .show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.spacing_mut().item_spacing.y = 4.0;
            split_row(
                ui,
                14.0,
                |ui| {
                    text::label(
                        ui,
                        tr("build.gizmo"),
                        typography::label_sm(),
                        colors::OUTLINE,
                    );
                },
                |ui| {
                    text::label(
                        ui,
                        tr("build.world_matrix"),
                        typography::label_sm(),
                        colors::TERTIARY,
                    );
                },
            );
            let cell = (ui.available_width() - 8.0) / 3.0;
            let (row, _) = ui.allocate_exact_size(vec2(ui.available_width(), 24.0), Sense::hover());
            let axes = [
                ("X:", colors::PRIMARY),
                ("Y:", colors::TERTIARY),
                ("Z:", colors::SECONDARY),
            ];
            for (index, (axis, color)) in axes.into_iter().enumerate() {
                let rect = Rect::from_min_size(
                    pos2(row.left() + index as f32 * (cell + 4.0), row.top()),
                    vec2(cell, 24.0),
                );
                ui.painter()
                    .rect_filled(rect, CornerRadius::same(2), colors::SURFACE_CONTAINER);
                let value = format!("{:.2}", sample::CURSOR[index]);
                crate::components::text_layout::paint_space_between(
                    ui.painter(),
                    rect.shrink2(vec2(10.0, 0.0)),
                    &[
                        (axis, typography::mono_medium(11.0), color),
                        (&value, typography::label_sm(), colors::ON_SURFACE),
                    ],
                );
            }
        });
}
