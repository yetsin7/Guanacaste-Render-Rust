//! The guided-step card: instructions, the active specimen and progress controls.

use crate::components::{
    button::{Button, ButtonStyle},
    frames, icons,
    layout::split_row,
    paint,
    tag::Tag,
    text, text_layout,
};
use crate::config::theme::{colors, typography};
use crate::features::build::models::{BuildGuide, BuildState};
use crate::services::localization::{tr, trf};
use crate::services::sample_data::shell;
use crate::services::textures::{Placeholder, Textures};
use eframe::egui::{self, pos2, vec2, CornerRadius, Margin, Rect, Sense, Ui, UiBuilder, Vec2};

/// Card width in points.
pub const WIDTH: f32 = 336.0;

/// Draws the card at `origin`; returns true when "Next specimen" was clicked.
pub fn show(
    ui: &mut Ui,
    origin: egui::Pos2,
    guide: &BuildGuide,
    state: &BuildState,
    textures: &mut Textures,
) -> bool {
    let mut next = false;
    let area = Rect::from_min_size(origin, vec2(WIDTH, 400.0));
    let mut child = ui.new_child(UiBuilder::new().max_rect(area));
    frames::overlay(14).show(&mut child, |ui| {
        ui.set_width(WIDTH - 28.0);
        ui.spacing_mut().item_spacing.y = 12.0;
        split_row(
            ui,
            20.0,
            |ui| {
                ui.spacing_mut().item_spacing.x = 8.0;
                let step = trf(
                    "build.step",
                    &[&state.step.to_string(), &guide.step_total.to_string()],
                );
                Tag::new(step, colors::PRIMARY)
                    .font(typography::mono_medium(11.0))
                    .fill(colors::with_alpha(colors::PRIMARY, 38))
                    .padding(8.0, 2.0)
                    .show(ui);
                text::label(
                    ui,
                    tr("build.archetype"),
                    typography::label_sm(),
                    colors::OUTLINE,
                );
            },
            |ui| {
                Tag::new(shell::FPS_LABEL, colors::TERTIARY)
                    .dot(colors::TERTIARY)
                    .gap(6.0)
                    .fill(colors::SURFACE_LOWEST)
                    .padding(8.0, 2.0)
                    .show(ui);
            },
        );
        ui.vertical(|ui| {
            ui.spacing_mut().item_spacing.y = 4.0;
            text::label(
                ui,
                guide.title,
                typography::headline_sm(),
                colors::ON_SURFACE,
            );
            text::paragraph(
                ui,
                guide.body,
                typography::body_sm(),
                colors::ON_SURFACE_VARIANT,
                19.5,
            );
        });
        specimen(ui, guide, textures);
        let placed = state.placed.to_string();
        split_row(
            ui,
            28.0,
            |ui| {
                ui.spacing_mut().item_spacing.x = 0.0;
                Tag::new(tr("build.placed"), colors::ON_SURFACE_VARIANT)
                    .part(format!(" {placed}"), colors::PRIMARY)
                    .part(
                        format!("/{}", guide.instances_total),
                        colors::ON_SURFACE_VARIANT,
                    )
                    .padding(0.0, 0.0)
                    .show(ui);
            },
            |ui| {
                ui.spacing_mut().item_spacing.x = 6.0;
                next = Button::new(tr("build.next"), ButtonStyle::Solid)
                    .font(typography::mono_medium(11.0))
                    .icon(icons::ARROW_FORWARD, 14.0)
                    .padding(12.0, 4.0)
                    .colors(colors::SECONDARY_CONTAINER, colors::ON_SECONDARY_CONTAINER)
                    .show(ui)
                    .clicked();
                Button::new(tr("build.skip"), ButtonStyle::Quiet)
                    .font(typography::label_sm())
                    .padding(10.0, 4.0)
                    .colors(egui::Color32::TRANSPARENT, colors::OUTLINE)
                    .show(ui);
            },
        );
    });
    next
}

/// The thumbnail and facts of the active specimen.
fn specimen(ui: &mut Ui, guide: &BuildGuide, textures: &mut Textures) {
    egui::Frame::new()
        .fill(colors::with_alpha(colors::SURFACE_HIGH, 153))
        .corner_radius(CornerRadius::same(2))
        .inner_margin(Margin::same(8))
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing.x = 10.0;
                let (thumb, _) = ui.allocate_exact_size(Vec2::splat(64.0), Sense::hover());
                let texture = textures.get(ui.ctx(), Placeholder::ElmSpecimen).clone();
                paint::image_cover(
                    &ui.painter().with_clip_rect(thumb),
                    thumb,
                    &texture,
                    vec2(0.5, 0.5),
                    egui::Color32::WHITE,
                );
                let width = ui.available_width();
                ui.allocate_ui_with_layout(
                    vec2(width, 64.0),
                    egui::Layout::top_down(egui::Align::Min),
                    |ui| {
                        ui.spacing_mut().item_spacing.y = 4.0;
                        ui.add_space(2.0);
                        split_row(
                            ui,
                            16.0,
                            |ui| {
                                text::label(
                                    ui,
                                    guide.asset,
                                    typography::label_md(),
                                    colors::ON_SURFACE,
                                );
                            },
                            |ui| {
                                text::label(
                                    ui,
                                    tr("build.active"),
                                    typography::label_sm(),
                                    colors::PRIMARY,
                                );
                            },
                        );
                        let cell = (width - 8.0) / 2.0;
                        for pair in guide.facts.chunks(2) {
                            let (row, _) =
                                ui.allocate_exact_size(vec2(width, 14.0), Sense::hover());
                            for (index, fact) in pair.iter().enumerate() {
                                let rect = Rect::from_min_size(
                                    pos2(row.left() + index as f32 * (cell + 8.0), row.top()),
                                    vec2(cell, 14.0),
                                );
                                text_layout::clamped(
                                    &mut ui.new_child(UiBuilder::new().max_rect(rect)),
                                    fact,
                                    typography::label_sm(),
                                    colors::OUTLINE,
                                    cell,
                                    1,
                                    14.0,
                                );
                            }
                        }
                    },
                );
            });
        });
}
