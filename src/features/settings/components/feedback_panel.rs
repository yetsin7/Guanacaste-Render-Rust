//! The live viewport feedback card: a placeholder render with camera readouts.

use crate::components::{layout::split_row, paint, tag::Tag, text};
use crate::config::theme::{colors, typography};
use crate::services::localization::tr;
use crate::services::textures::{Placeholder, Textures};
use eframe::egui::{
    self, pos2, vec2, Align, CornerRadius, Layout, Margin, Rect, Sense, Ui, UiBuilder,
};

/// Draws the card.
pub fn show(ui: &mut Ui, textures: &mut Textures) {
    egui::Frame::new()
        .fill(colors::SURFACE_LOW)
        .corner_radius(CornerRadius::same(4))
        .show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.spacing_mut().item_spacing.y = 0.0;
            egui::Frame::new()
                .fill(colors::SURFACE_CONTAINER)
                .inner_margin(Margin::symmetric(12, 8))
                .show(ui, |ui| {
                    ui.set_width(ui.available_width());
                    split_row(
                        ui,
                        16.0,
                        |ui| {
                            Tag::new(text::caps(tr("settings.feedback")), colors::ON_SURFACE)
                                .font(typography::mono_medium(11.0))
                                .dot(colors::TERTIARY)
                                .gap(8.0)
                                .padding(0.0, 0.0)
                                .show(ui);
                        },
                        |ui| {
                            text::label(
                                ui,
                                tr("settings.camera"),
                                typography::label_sm(),
                                colors::OUTLINE,
                            );
                        },
                    );
                });
            let (rect, _) =
                ui.allocate_exact_size(vec2(ui.available_width(), 224.0), Sense::hover());
            ui.painter().rect_filled(rect, 0.0, colors::SURFACE_LOWEST);
            let texture = textures.get(ui.ctx(), Placeholder::VillaRender).clone();
            paint::image_cover(
                &ui.painter().with_clip_rect(rect),
                rect,
                &texture,
                vec2(0.5, 0.5),
                egui::Color32::WHITE,
            );
            let fill = colors::with_alpha(colors::SURFACE_LOWEST, 204);
            let mut top = ui.new_child(UiBuilder::new().max_rect(rect.shrink(8.0)));
            Tag::new("37 FPS", colors::TERTIARY)
                .font(typography::mono_medium(11.0))
                .part(format!("  |  {}", tr("settings.scene")), colors::ON_SURFACE)
                .fill(fill)
                .padding(8.0, 2.0)
                .show(&mut top);
            let bar = Rect::from_min_size(
                pos2(rect.left() + 8.0, rect.bottom() - 32.0),
                vec2(rect.width() - 16.0, 24.0),
            );
            ui.painter().rect_filled(
                bar,
                CornerRadius::same(2),
                colors::with_alpha(colors::SURFACE_LOWEST, 217),
            );
            let mut inner = ui.new_child(
                UiBuilder::new()
                    .max_rect(bar.shrink2(vec2(12.0, 0.0)))
                    .layout(Layout::left_to_right(Align::Center)),
            );
            split_row(
                &mut inner,
                24.0,
                |ui| {
                    ui.spacing_mut().item_spacing.x = 12.0;
                    for key in ["settings.focal", "settings.iso", "settings.shutter"] {
                        let (caption, value) = tr(key).split_once(' ').unwrap_or((tr(key), ""));
                        Tag::new(caption, colors::OUTLINE)
                            .part(format!(" {value}"), colors::ON_SURFACE)
                            .padding(0.0, 0.0)
                            .show(ui);
                    }
                },
                |ui| {
                    text::label(
                        ui,
                        "DXR RT ON",
                        typography::mono_medium(11.0),
                        colors::PRIMARY,
                    );
                },
            );
        });
}
