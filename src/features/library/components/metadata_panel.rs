//! The asset metadata panel: region, version, wind, shading, LOD levels and transforms.

use crate::components::{
    icons,
    layout::split_row,
    slider::{Slider, Thumb},
    tag::Tag,
    text,
};
use crate::config::theme::{colors, typography};
use crate::features::library::logic::inspector::beaufort_number;
use crate::features::library::models::{AssetDetails, InspectorState};
use crate::services::localization::tr;
use eframe::egui::{self, pos2, vec2, CornerRadius, Margin, Sense, Ui};

fn block(ui: &mut Ui, add: impl FnOnce(&mut Ui)) {
    egui::Frame::new()
        .fill(colors::SURFACE_CONTAINER)
        .corner_radius(CornerRadius::same(4))
        .inner_margin(Margin::same(8))
        .show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.spacing_mut().item_spacing.y = 4.0;
            add(ui);
        });
}

fn title(ui: &mut Ui, glyph: char, key: &'static str) {
    ui.spacing_mut().item_spacing.x = 8.0;
    text::icon(ui, glyph, 18.0, colors::OUTLINE);
    text::label(ui, tr(key), typography::body_md(), colors::ON_SURFACE);
}

/// Draws the panel.
pub fn show(ui: &mut Ui, details: &AssetDetails, state: &mut InspectorState) {
    egui::Frame::new()
        .fill(colors::SURFACE_LOW)
        .corner_radius(CornerRadius::same(8))
        .show(ui, |ui| {
            ui.spacing_mut().item_spacing.y = 0.0;
            egui::Frame::new()
                .fill(colors::SURFACE_HIGH)
                .corner_radius(CornerRadius {
                    nw: 8,
                    ne: 8,
                    sw: 0,
                    se: 0,
                })
                .inner_margin(Margin::symmetric(12, 8))
                .show(ui, |ui| {
                    ui.set_width(ui.available_width());
                    split_row(
                        ui,
                        24.0,
                        |ui| {
                            ui.spacing_mut().item_spacing.x = 4.0;
                            text::icon(ui, icons::TUNE, 18.0, colors::PRIMARY);
                            text::label(
                                ui,
                                text::caps(tr("inspector.metadata")),
                                typography::headline_sm(),
                                colors::ON_SURFACE,
                            );
                        },
                        |ui| {
                            Tag::new(tr("inspector.verified"), colors::ON_SURFACE_VARIANT)
                                .dot(colors::TERTIARY)
                                .gap(4.0)
                                .padding(0.0, 0.0)
                                .show(ui);
                        },
                    );
                });
            egui::Frame::new()
                .inner_margin(Margin::same(12))
                .show(ui, |ui| {
                    ui.set_width(ui.available_width());
                    ui.spacing_mut().item_spacing.y = 12.0;
                    rows(ui, details, state);
                });
            footer(ui, details);
        });
}

fn rows(ui: &mut Ui, details: &AssetDetails, state: &mut InspectorState) {
    block(ui, |ui| {
        split_row(
            ui,
            24.0,
            |ui| title(ui, icons::PUBLIC, "inspector.region"),
            |ui| {
                ui.spacing_mut().item_spacing.x = 4.0;
                Tag::new(details.climate, colors::OUTLINE)
                    .fill(colors::SURFACE_HIGHEST)
                    .padding(4.0, 2.0)
                    .show(ui);
                text::label(
                    ui,
                    details.region,
                    typography::label_md(),
                    colors::SECONDARY,
                );
            },
        );
    });
    block(ui, |ui| {
        split_row(
            ui,
            24.0,
            |ui| title(ui, icons::ARCHIVE, "inspector.version"),
            |ui| {
                ui.spacing_mut().item_spacing.x = 4.0;
                text::icon(ui, icons::VERIFIED, 16.0, colors::TERTIARY);
                Tag::new("L26.0-PRO", colors::PRIMARY)
                    .font(typography::label_md())
                    .fill(colors::with_alpha(colors::PRIMARY_CONTAINER, 51))
                    .padding(8.0, 2.0)
                    .show(ui);
            },
        );
    });
    block(ui, |ui| {
        split_row(
            ui,
            24.0,
            |ui| title(ui, icons::WIND, "inspector.wind"),
            |ui| {
                text::label(
                    ui,
                    tr("inspector.wind_value"),
                    typography::label_md(),
                    colors::SECONDARY,
                );
            },
        );
        ui.add_space(4.0);
        split_row(
            ui,
            14.0,
            |ui| {
                text::label(
                    ui,
                    text::caps(tr("inspector.turbulence")),
                    typography::label_sm(),
                    colors::OUTLINE,
                );
            },
            |ui| {
                text::label(
                    ui,
                    format!("{:.2}", state.wind),
                    typography::mono_medium(11.0),
                    colors::PRIMARY,
                );
            },
        );
        Slider::precision()
            .track(6.0, true)
            .thumb(Thumb::Circle(8.0))
            .colors(colors::PRIMARY, colors::SURFACE_HIGHEST)
            .show(ui, &mut state.wind);
        let bft = format!(
            "BFT {} ({})",
            beaufort_number(state.wind),
            tr(crate::features::library::logic::inspector::beaufort_label(
                state.wind
            ))
        );
        crate::components::text_layout::space_between_row(
            ui,
            14.0,
            &[
                (
                    tr("inspector.calm"),
                    typography::mono(10.0),
                    colors::OUTLINE,
                ),
                (&bft, typography::mono(10.0), colors::OUTLINE),
                (
                    tr("inspector.storm"),
                    typography::mono(10.0),
                    colors::OUTLINE,
                ),
            ],
        );
    });
    block(ui, |ui| {
        split_row(
            ui,
            24.0,
            |ui| title(ui, icons::SUN, "inspector.shading"),
            |ui| {
                Tag::new(tr("inspector.shading_value"), colors::TERTIARY)
                    .font(typography::label_md())
                    .dot(colors::TERTIARY)
                    .gap(4.0)
                    .padding(0.0, 0.0)
                    .show(ui);
            },
        );
    });
    block(ui, |ui| lod_levels(ui, details));
    block(ui, |ui| transforms(ui, details));
}

fn lod_levels(ui: &mut Ui, details: &AssetDetails) {
    split_row(
        ui,
        24.0,
        |ui| title(ui, icons::LAYERS, "inspector.lod"),
        |ui| {
            Tag::new(tr("inspector.lod_active"), colors::TERTIARY)
                .fill(colors::with_alpha(colors::TERTIARY, 26))
                .padding(8.0, 2.0)
                .show(ui);
        },
    );
    let gap = 4.0;
    let width = (ui.available_width() - gap * 3.0) / 4.0;
    let (row, _) = ui.allocate_exact_size(vec2(ui.available_width(), 40.0), Sense::hover());
    for (index, (name, count)) in details.lods.iter().enumerate() {
        let rect = egui::Rect::from_min_size(
            pos2(row.left() + index as f32 * (width + gap), row.top()),
            vec2(width, 40.0),
        );
        ui.painter()
            .rect_filled(rect, CornerRadius::same(2), colors::SURFACE_HIGH);
        ui.painter().rect_filled(
            egui::Rect::from_min_size(pos2(rect.left(), rect.bottom() - 2.0), vec2(width, 2.0)),
            0.0,
            colors::PRIMARY,
        );
        text::paint_text(
            ui.painter(),
            pos2(rect.center().x, rect.top() + 13.0),
            egui::Align2::CENTER_CENTER,
            name,
            typography::mono_medium(11.0),
            colors::PRIMARY,
        );
        text::paint_text(
            ui.painter(),
            pos2(rect.center().x, rect.top() + 28.0),
            egui::Align2::CENTER_CENTER,
            count,
            typography::mono(10.0),
            colors::ON_SURFACE_VARIANT,
        );
    }
}

fn transforms(ui: &mut Ui, details: &AssetDetails) {
    text::label(
        ui,
        text::caps(tr("inspector.transforms")),
        typography::label_sm(),
        colors::OUTLINE,
    );
    let gap = 4.0;
    let axes = [
        ("X", colors::PRIMARY),
        ("Y", colors::TERTIARY),
        ("Z", colors::SECONDARY),
    ];
    let width = (ui.available_width() - gap * 2.0) / 3.0;
    let (row, _) = ui.allocate_exact_size(vec2(ui.available_width(), 26.0), Sense::hover());
    for (index, (axis, color)) in axes.into_iter().enumerate() {
        let rect = egui::Rect::from_min_size(
            pos2(row.left() + index as f32 * (width + gap), row.top()),
            vec2(width, 26.0),
        );
        ui.painter()
            .rect_filled(rect, CornerRadius::same(2), colors::SURFACE_LOWEST);
        crate::components::text_layout::paint_space_between(
            ui.painter(),
            rect.shrink2(vec2(6.0, 0.0)),
            &[
                (axis, typography::mono_medium(11.0), color),
                (
                    details.position[index],
                    typography::label_sm(),
                    colors::ON_SURFACE,
                ),
            ],
        );
    }
    let width = (ui.available_width() - gap) / 2.0;
    let (row, _) = ui.allocate_exact_size(vec2(ui.available_width(), 26.0), Sense::hover());
    let pairs = [
        (
            tr("inspector.rotation"),
            details.rotation,
            colors::ON_SURFACE,
        ),
        (tr("inspector.scale"), details.scale, colors::TERTIARY),
    ];
    for (index, (label, value, color)) in pairs.into_iter().enumerate() {
        let rect = egui::Rect::from_min_size(
            pos2(row.left() + index as f32 * (width + gap), row.top()),
            vec2(width, 26.0),
        );
        ui.painter()
            .rect_filled(rect, CornerRadius::same(2), colors::SURFACE_LOWEST);
        crate::components::text_layout::paint_space_between(
            ui.painter(),
            rect.shrink2(vec2(6.0, 0.0)),
            &[
                (label, typography::label_sm(), colors::OUTLINE),
                (value, typography::label_sm(), color),
            ],
        );
    }
}

fn footer(ui: &mut Ui, details: &AssetDetails) {
    egui::Frame::new()
        .fill(colors::with_alpha(colors::SURFACE_HIGHEST, 153))
        .corner_radius(CornerRadius {
            nw: 0,
            ne: 0,
            sw: 8,
            se: 8,
        })
        .inner_margin(Margin::symmetric(12, 8))
        .show(ui, |ui| {
            ui.set_width(ui.available_width());
            split_row(
                ui,
                18.0,
                |ui| {
                    ui.spacing_mut().item_spacing.x = 4.0;
                    text::icon(ui, icons::MEMORY, 14.0, colors::OUTLINE);
                    text::label(
                        ui,
                        format!("{} {}", tr("inspector.vram"), details.vram),
                        typography::label_sm(),
                        colors::OUTLINE,
                    );
                },
                |ui| {
                    Tag::new(tr("inspector.rebake"), colors::PRIMARY)
                        .font(typography::mono_medium(11.0))
                        .trail_icon(icons::REFRESH, 14.0, colors::PRIMARY)
                        .gap(2.0)
                        .padding(0.0, 0.0)
                        .show(ui);
                },
            );
        });
}
