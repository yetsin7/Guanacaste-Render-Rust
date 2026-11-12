//! The PBR channel strength cells and the ambient-occlusion switch.

use super::properties_panel::section;
use crate::components::{
    checkbox, icons,
    layout::split_row,
    slider::{Slider, Thumb},
    text,
};
use crate::config::theme::{colors, typography};
use crate::features::materials::models::MaterialState;
use crate::models::slider_spec::SliderSpec;
use crate::services::localization::tr;
use eframe::egui::{pos2, vec2, Align, CornerRadius, Layout, Rect, Sense, Ui, UiBuilder};

const CHANNELS: [(&str, SliderSpec, &str, eframe::egui::Color32); 3] = [
    (
        "material.normal",
        SliderSpec {
            min: 0.0,
            max: 1.77,
            unit: "",
            decimals: 2,
        },
        "",
        colors::TERTIARY,
    ),
    (
        "material.metallic",
        SliderSpec {
            min: 0.0,
            max: 0.4,
            unit: "",
            decimals: 2,
        },
        "DIE",
        colors::OUTLINE,
    ),
    (
        "material.specular",
        SliderSpec {
            min: 0.0,
            max: 1.0,
            unit: "",
            decimals: 2,
        },
        "IOR",
        colors::TERTIARY,
    ),
];

/// Draws the section.
pub fn show(ui: &mut Ui, state: &mut MaterialState) {
    section(ui, |ui| {
        ui.spacing_mut().item_spacing.y = 8.0;
        split_row(
            ui,
            20.0,
            |ui| {
                ui.spacing_mut().item_spacing.x = 4.0;
                text::icon(ui, icons::TUNE, 14.0, colors::ON_SURFACE);
                text::label(
                    ui,
                    text::caps(tr("material.channels")),
                    typography::label_md(),
                    colors::ON_SURFACE,
                );
            },
            |ui| {
                text::icon(ui, icons::MORE_HORIZ, 16.0, colors::OUTLINE);
            },
        );
        let gap = 8.0;
        let width = (ui.available_width() - gap * 2.0) / 3.0;
        let (row, _) = ui.allocate_exact_size(vec2(ui.available_width(), 62.0), Sense::hover());
        for (index, (key, spec, tag, tag_color)) in CHANNELS.into_iter().enumerate() {
            let rect = Rect::from_min_size(
                pos2(row.left() + index as f32 * (width + gap), row.top()),
                vec2(width, 62.0),
            );
            ui.painter()
                .rect_filled(rect, CornerRadius::same(2), colors::SURFACE_LOWEST);
            text::paint_text(
                ui.painter(),
                rect.min + vec2(8.0, 12.0),
                eframe::egui::Align2::LEFT_CENTER,
                &text::caps(tr(key)),
                typography::mono(10.0),
                colors::OUTLINE,
            );
            text::paint_text(
                ui.painter(),
                rect.min + vec2(8.0, 32.0),
                eframe::egui::Align2::LEFT_CENTER,
                &spec.format(state.channels[index]),
                typography::label_sm(),
                colors::ON_SURFACE,
            );
            if tag.is_empty() {
                text::paint_icon(
                    ui.painter(),
                    pos2(rect.right() - 14.0, rect.top() + 32.0),
                    icons::CHECK_CIRCLE,
                    12.0,
                    colors::TERTIARY,
                );
            } else {
                text::paint_text(
                    ui.painter(),
                    pos2(rect.right() - 8.0, rect.top() + 32.0),
                    eframe::egui::Align2::RIGHT_CENTER,
                    tag,
                    typography::mono(10.0),
                    tag_color,
                );
            }
            let mut bar = ui.new_child(UiBuilder::new().max_rect(Rect::from_min_size(
                rect.min + vec2(8.0, 44.0),
                vec2(width - 16.0, 10.0),
            )));
            let fill = if index == 0 {
                colors::SECONDARY_CONTAINER
            } else {
                colors::PRIMARY_CONTAINER
            };
            Slider::bar(4.0)
                .thumb(Thumb::None)
                .colors(fill, colors::SURFACE_HIGH)
                .show(&mut bar, &mut state.channels[index]);
        }
        let mut ao = ui.new_child(
            UiBuilder::new()
                .max_rect(ui.available_rect_before_wrap())
                .layout(Layout::top_down(Align::Min)),
        );
        split_row(
            &mut ao,
            24.0,
            |ui| {
                ui.spacing_mut().item_spacing.x = 4.0;
                text::icon(ui, icons::EFFECTS, 14.0, colors::ON_SURFACE_VARIANT);
                text::label(
                    ui,
                    tr("material.ao"),
                    typography::body_sm(),
                    colors::ON_SURFACE_VARIANT,
                );
            },
            |ui| {
                checkbox::show(ui, &mut state.ambient_occlusion);
            },
        );
        ui.allocate_space(vec2(1.0, 24.0));
    });
}
