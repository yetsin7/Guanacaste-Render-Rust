//! The shortcut key caps panel at the bottom-right of the 3D view.

use crate::components::{frames, key_cap::KeyCap, text};
use crate::config::theme::{colors, typography};
use crate::services::localization::tr;
use eframe::egui::{self, Align2, Margin, Pos2, Sense, Ui, Vec2};

fn divider(ui: &mut Ui) {
    let (rect, _) = ui.allocate_exact_size(Vec2::new(1.0, 12.0), Sense::hover());
    ui.painter().rect_filled(rect, 0.0, colors::SURFACE_HIGH);
}

/// Draws the panel with its bottom-right corner at `corner`.
pub fn show(ui: &mut Ui, corner: Pos2) {
    egui::Area::new(egui::Id::new("build-shortcuts"))
        .order(egui::Order::Middle)
        .pivot(Align2::RIGHT_BOTTOM)
        .fixed_pos(corner)
        .show(ui.ctx(), |ui| {
            frames::overlay(0)
                .inner_margin(Margin::symmetric(10, 6))
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        ui.spacing_mut().item_spacing.x = 6.0;
                        text::label(
                            ui,
                            tr("build.shortcuts"),
                            typography::label_sm(),
                            colors::OUTLINE,
                        );
                        let groups: [(&[&str], eframe::egui::Color32, bool); 3] = [
                            (
                                &["build.key_shift", "build.key_alt", "build.key_ctrl"],
                                colors::ON_SURFACE,
                                false,
                            ),
                            (&["build.key_wasd", "build.key_qe"], colors::PRIMARY, true),
                            (
                                &["build.key_rmb", "build.key_mmb", "build.key_scroll"],
                                colors::TERTIARY,
                                false,
                            ),
                        ];
                        for (index, (keys, ink, medium)) in groups.into_iter().enumerate() {
                            if index > 0 {
                                divider(ui);
                            }
                            for key in keys {
                                let font = if medium {
                                    typography::mono_medium(11.0)
                                } else {
                                    typography::label_sm()
                                };
                                KeyCap::new(tr(key))
                                    .font(font)
                                    .colors(colors::SURFACE_HIGHEST, ink)
                                    .show(ui);
                            }
                        }
                    });
                });
        });
}
