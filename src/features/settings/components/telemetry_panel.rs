//! The hardware telemetry card: GPU, VRAM, thermals, power and frame-time history.

use super::render_engine_panel::card;
use crate::components::{
    icons,
    layout::split_row,
    sparkline::{self, SeriesStyle},
    tag::Tag,
    text, text_layout,
};
use crate::config::theme::{colors, typography};
use crate::services::localization::tr;
use crate::services::sample_data::settings as sample;
use eframe::egui::{
    self, pos2, vec2, Color32, CornerRadius, Margin, Rect, Sense, Stroke, Ui, UiBuilder,
};

fn block(ui: &mut Ui, padding: i8, add: impl FnOnce(&mut Ui)) {
    egui::Frame::new()
        .fill(colors::SURFACE_CONTAINER)
        .corner_radius(CornerRadius::same(2))
        .inner_margin(Margin::same(padding))
        .show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.spacing_mut().item_spacing.y = 4.0;
            add(ui);
        });
}

fn meter(ui: &mut Ui, height: f32, parts: &[(f32, Color32)]) {
    let (rect, _) = ui.allocate_exact_size(vec2(ui.available_width(), height), Sense::hover());
    ui.painter()
        .rect_filled(rect, CornerRadius::same(255), colors::SURFACE_LOWEST);
    let mut x = rect.left();
    for (fraction, color) in parts {
        let width = rect.width() * fraction;
        ui.painter().rect_filled(
            Rect::from_min_size(pos2(x, rect.top()), vec2(width, height)),
            CornerRadius::ZERO,
            *color,
        );
        x += width;
    }
}

/// Draws the card.
pub fn show(ui: &mut Ui) {
    card(
        ui,
        icons::MEMORY,
        colors::PRIMARY,
        "settings.telemetry",
        "settings.nominal",
        |ui| {
            ui.spacing_mut().item_spacing.y = 8.0;
            block(ui, 12, |ui| {
                split_row(
                    ui,
                    36.0,
                    |ui| {
                        ui.spacing_mut().item_spacing.x = 8.0;
                        let (tile, _) = ui.allocate_exact_size(vec2(28.0, 28.0), Sense::hover());
                        ui.painter().rect_filled(
                            tile,
                            CornerRadius::same(2),
                            colors::SURFACE_LOWEST,
                        );
                        text::paint_icon(
                            ui.painter(),
                            tile.center(),
                            icons::MEMORY,
                            16.0,
                            colors::PRIMARY,
                        );
                        ui.vertical(|ui| {
                            ui.spacing_mut().item_spacing.y = 0.0;
                            text::label(
                                ui,
                                sample::GPU_NAME,
                                typography::label_md(),
                                colors::ON_SURFACE,
                            );
                            text::label(
                                ui,
                                tr("settings.gpu_detail"),
                                typography::label_sm(),
                                colors::OUTLINE,
                            );
                        });
                    },
                    |ui| {
                        Tag::new(tr("settings.dxr_active"), colors::SECONDARY)
                            .fill(colors::SURFACE_HIGHEST)
                            .padding(8.0, 2.0)
                            .show(ui);
                    },
                );
            });
            block(ui, 12, |ui| {
                split_row(
                    ui,
                    18.0,
                    |ui| {
                        text::label(
                            ui,
                            tr("settings.vram"),
                            typography::body_sm(),
                            colors::ON_SURFACE_VARIANT,
                        );
                    },
                    |ui| {
                        text::label(
                            ui,
                            "18.4 GB / 24.0 GB",
                            typography::label_md(),
                            colors::PRIMARY,
                        );
                    },
                );
                meter(
                    ui,
                    8.0,
                    &[
                        (sample::VRAM_GEOMETRY, colors::PRIMARY),
                        (sample::VRAM_RT, colors::SECONDARY),
                    ],
                );
                text_layout::space_between_row(
                    ui,
                    14.0,
                    &[
                        (
                            tr("settings.vram_geometry"),
                            typography::label_sm(),
                            colors::OUTLINE,
                        ),
                        (
                            tr("settings.vram_rt"),
                            typography::label_sm(),
                            colors::OUTLINE,
                        ),
                        (
                            tr("settings.vram_free"),
                            typography::label_sm(),
                            colors::OUTLINE,
                        ),
                    ],
                );
            });
            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing.x = 8.0;
                let width = (ui.available_width() - 8.0) / 2.0;
                for (caption, value, limit, fraction, color, value_color) in [
                    (
                        "settings.thermal",
                        "64°C",
                        "settings.thermal_limit",
                        0.72,
                        colors::TERTIARY,
                        colors::TERTIARY,
                    ),
                    (
                        "settings.power",
                        "348 W",
                        "settings.power_limit",
                        0.77,
                        colors::PRIMARY_CONTAINER,
                        colors::ON_SURFACE,
                    ),
                ] {
                    ui.allocate_ui_with_layout(
                        vec2(width, 0.0),
                        egui::Layout::top_down(egui::Align::Min),
                        |ui| {
                            ui.set_max_width(width);
                            block(ui, 12, |ui| {
                                text::label(
                                    ui,
                                    text::caps(tr(caption)),
                                    typography::label_sm(),
                                    colors::OUTLINE,
                                );
                                ui.horizontal(|ui| {
                                    ui.spacing_mut().item_spacing.x = 4.0;
                                    text::label(ui, value, typography::headline_md(), value_color);
                                    text::label(
                                        ui,
                                        tr(limit),
                                        typography::label_sm(),
                                        colors::OUTLINE,
                                    );
                                });
                                meter(ui, 4.0, &[(fraction, color)]);
                            });
                        },
                    );
                }
            });
            block(ui, 12, |ui| {
                split_row(
                    ui,
                    18.0,
                    |ui| {
                        ui.spacing_mut().item_spacing.x = 4.0;
                        text::label(
                            ui,
                            tr("settings.frametime"),
                            typography::body_sm(),
                            colors::ON_SURFACE,
                        );
                        text::label(
                            ui,
                            tr("settings.frametime_note"),
                            typography::label_sm(),
                            colors::OUTLINE,
                        );
                    },
                    |ui| {
                        Tag::new(tr("settings.avg"), colors::TERTIARY)
                            .font(typography::label_md())
                            .dot(colors::TERTIARY)
                            .gap(4.0)
                            .padding(0.0, 0.0)
                            .show(ui);
                    },
                );
                let (rect, _) =
                    ui.allocate_exact_size(vec2(ui.available_width(), 64.0), Sense::hover());
                let series = sample::frametimes();
                let range = (14.0, 36.0);
                for (value, key) in [(33.3, "settings.fps30"), (16.6, "settings.fps60")] {
                    let t = (value - range.0) / (range.1 - range.0);
                    let y = rect.bottom() - t * rect.height();
                    ui.painter().add(egui::Shape::dashed_line(
                        &[pos2(rect.left(), y), pos2(rect.right(), y)],
                        Stroke::new(1.0, colors::with_alpha(colors::OUTLINE, 90)),
                        4.0,
                        4.0,
                    ));
                    text::paint_text(
                        ui.painter(),
                        pos2(rect.left() + 6.0, y - 6.0),
                        egui::Align2::LEFT_CENTER,
                        tr(key),
                        typography::mono(9.0),
                        colors::OUTLINE,
                    );
                }
                let mut plot = ui.new_child(UiBuilder::new().max_rect(rect));
                sparkline::show_series(
                    &mut plot,
                    rect,
                    &series,
                    Some(range),
                    SeriesStyle {
                        fill: true,
                        width: 1.2,
                        ..SeriesStyle::line(colors::TERTIARY)
                    },
                );
                text_layout::space_between_row(
                    ui,
                    14.0,
                    &[
                        (tr("settings.min"), typography::label_sm(), colors::OUTLINE),
                        (
                            tr("settings.jitter"),
                            typography::label_sm(),
                            colors::OUTLINE,
                        ),
                        (tr("settings.max"), typography::label_sm(), colors::OUTLINE),
                    ],
                );
            });
            block(ui, 8, |ui| {
                split_row(
                    ui,
                    18.0,
                    |ui| {
                        Tag::new(tr("settings.operator"), colors::ON_SURFACE_VARIANT)
                            .part(
                                format!(" {}", tr("settings.operator_value")),
                                colors::ON_SURFACE,
                            )
                            .lead_icon(icons::ACCOUNT, 16.0, colors::OUTLINE)
                            .gap(4.0)
                            .padding(0.0, 0.0)
                            .show(ui);
                    },
                    |ui| {
                        text::label(
                            ui,
                            tr("settings.license"),
                            typography::label_sm(),
                            colors::OUTLINE,
                        );
                    },
                );
            });
        },
    );
}
