//! The roughness map card with its texture thumbnail.

use super::properties_panel::section;
use crate::components::{
    button::{Button, ButtonStyle},
    icons,
    layout::split_row,
    text,
};
use crate::config::theme::{colors, typography};
use crate::services::localization::tr;
use eframe::egui::{pos2, vec2, Align, CornerRadius, Layout, Rect, Sense, Ui, UiBuilder};

/// Draws the section.
pub fn show(ui: &mut Ui) {
    section(ui, |ui| {
        split_row(
            ui,
            20.0,
            |ui| {
                ui.spacing_mut().item_spacing.x = 4.0;
                text::icon(ui, icons::TEXTURE, 14.0, colors::SECONDARY);
                text::label(
                    ui,
                    text::caps(tr("material.roughness")),
                    typography::label_md(),
                    colors::ON_SURFACE,
                );
            },
            |ui| {
                text::label(
                    ui,
                    tr("material.float16"),
                    typography::label_sm(),
                    colors::TERTIARY,
                );
            },
        );
        let (rect, _) = ui.allocate_exact_size(vec2(ui.available_width(), 72.0), Sense::hover());
        ui.painter()
            .rect_filled(rect, CornerRadius::same(2), colors::SURFACE_LOWEST);
        let thumb = Rect::from_min_size(rect.min + vec2(8.0, 8.0), vec2(56.0, 56.0));
        noise(ui, thumb);
        text::paint_text(
            ui.painter(),
            thumb.right_bottom() - vec2(2.0, 5.0),
            eframe::egui::Align2::RIGHT_CENTER,
            "EXR",
            typography::mono(8.0),
            colors::PRIMARY,
        );
        let mut info = ui.new_child(
            UiBuilder::new()
                .max_rect(Rect::from_min_max(
                    pos2(thumb.right() + 12.0, rect.top() + 10.0),
                    rect.max,
                ))
                .layout(Layout::top_down(Align::Min)),
        );
        info.spacing_mut().item_spacing.y = 2.0;
        text::label(
            &mut info,
            "Concrete_Smooth_Rough.exr",
            typography::mono_medium(11.0),
            colors::ON_SURFACE,
        );
        text::label(
            &mut info,
            tr("material.texture_info"),
            typography::mono(10.0),
            colors::OUTLINE,
        );
        info.add_space(4.0);
        info.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = 8.0;
            Button::new(tr("material.browse"), ButtonStyle::Raised)
                .icon(icons::FOLDER_OPEN, 12.0)
                .font(typography::label_sm())
                .padding(8.0, 2.0)
                .show(ui);
            Button::new("", ButtonStyle::Raised)
                .icon(icons::REFRESH, 12.0)
                .font(typography::label_sm())
                .padding(8.0, 2.0)
                .show(ui);
        });
    });
}

/// A grayscale granular pattern standing in for the real roughness texture.
fn noise(ui: &Ui, rect: Rect) {
    let cells = 14;
    let size = rect.width() / cells as f32;
    let mut seed = 11_u32;
    for row in 0..cells {
        for column in 0..cells {
            seed = seed.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
            let value = 90 + ((seed >> 16) % 130) as u8;
            let cell = Rect::from_min_size(
                rect.min + vec2(column as f32 * size, row as f32 * size),
                vec2(size, size),
            );
            ui.painter()
                .rect_filled(cell, 0.0, eframe::egui::Color32::from_gray(value));
        }
    }
}
