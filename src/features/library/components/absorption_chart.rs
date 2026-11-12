//! The chlorophyll absorption and subsurface scattering curve panel.

use crate::components::{
    icons,
    layout::split_row,
    sparkline::{self, SeriesStyle},
    text,
};
use crate::config::theme::{colors, typography};
use crate::features::library::models::AssetDetails;
use crate::services::localization::tr;
use eframe::egui::{self, vec2, CornerRadius, Margin, Sense, Ui};

/// Draws the panel.
pub fn show(ui: &mut Ui, details: &AssetDetails) {
    egui::Frame::new()
        .fill(colors::SURFACE_LOW)
        .corner_radius(CornerRadius::same(8))
        .inner_margin(Margin::same(12))
        .show(ui, |ui| {
            ui.spacing_mut().item_spacing.y = 8.0;
            split_row(
                ui,
                24.0,
                |ui| {
                    ui.spacing_mut().item_spacing.x = 8.0;
                    text::icon(ui, icons::SHOW_CHART, 18.0, colors::PRIMARY);
                    text::label(
                        ui,
                        tr("inspector.chart"),
                        typography::headline_sm(),
                        colors::ON_SURFACE,
                    );
                },
                |ui| {
                    ui.spacing_mut().item_spacing.x = 16.0;
                    legend(ui, "inspector.scatter", colors::TERTIARY);
                    legend(ui, "inspector.transmittance", colors::SECONDARY);
                },
            );
            let (rect, _) =
                ui.allocate_exact_size(vec2(ui.available_width(), 96.0), Sense::hover());
            ui.painter()
                .rect_filled(rect, CornerRadius::same(4), colors::SURFACE_LOWEST);
            let plot = rect.shrink(8.0);
            let range = Some((0.0, 0.85));
            sparkline::show_series(
                ui,
                plot,
                &details.transmittance,
                range,
                SeriesStyle {
                    fill: true,
                    ..SeriesStyle::line(colors::SECONDARY)
                },
            );
            sparkline::show_series(
                ui,
                plot,
                &details.scatter,
                range,
                SeriesStyle {
                    dashed: true,
                    width: 1.2,
                    ..SeriesStyle::line(colors::TERTIARY)
                },
            );
            split_row(
                ui,
                14.0,
                |ui| {
                    text::label(
                        ui,
                        tr("inspector.uv"),
                        typography::label_sm(),
                        colors::OUTLINE,
                    );
                },
                |ui| {
                    text::label(
                        ui,
                        tr("inspector.ir"),
                        typography::label_sm(),
                        colors::OUTLINE,
                    );
                },
            );
        });
}

fn legend(ui: &mut Ui, key: &'static str, color: egui::Color32) {
    let (swatch, _) = ui.allocate_exact_size(vec2(10.0, 4.0), Sense::hover());
    ui.painter()
        .rect_filled(swatch, CornerRadius::same(2), color);
    text::label(
        ui,
        tr(key),
        typography::label_sm(),
        colors::ON_SURFACE_VARIANT,
    );
}
