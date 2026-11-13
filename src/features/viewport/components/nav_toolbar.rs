//! The navigation toolbar floating over the 3D view: frame the model, preset views, helper toggles and readouts.

use crate::components::button::{Button, ButtonStyle};
use crate::components::{frames, icon_button::IconButton, icons, tag::Tag};
use crate::config::theme::colors;
use crate::features::viewport::models::ViewPreset;
use crate::features::viewport::ui::panel::NOTICE_TIME;
use crate::features::viewport::ui::ViewportState;
use crate::services::localization::{tr, trf};
use crate::utils::format::with_thousands;
use eframe::egui::{self, Align2, Id, Margin, Pos2, Ui};

fn preset_labels(preset: ViewPreset) -> (&'static str, &'static str) {
    match preset {
        ViewPreset::Isometric => (tr("view3d.preset_iso"), tr("view3d.tip_iso")),
        ViewPreset::Top => (tr("view3d.preset_top"), tr("view3d.tip_top")),
        ViewPreset::Front => (tr("view3d.preset_front"), tr("view3d.tip_front")),
        ViewPreset::Back => (tr("view3d.preset_back"), tr("view3d.tip_back")),
        ViewPreset::Left => (tr("view3d.preset_left"), tr("view3d.tip_left")),
        ViewPreset::Right => (tr("view3d.preset_right"), tr("view3d.tip_right")),
    }
}

fn divider(ui: &mut Ui) {
    let (rect, _) = ui.allocate_exact_size(egui::vec2(1.0, 14.0), egui::Sense::hover());
    ui.painter().rect_filled(rect, 0.0, colors::SURFACE_HIGHEST);
}

fn toggle(ui: &mut Ui, label: &'static str, on: bool, tip: &'static str) -> bool {
    let style = if on {
        ButtonStyle::Raised
    } else {
        ButtonStyle::Quiet
    };
    Button::new(label, style)
        .padding(6.0, 4.0)
        .show(ui)
        .on_hover_text(tip)
        .clicked()
}

/// Draws the toolbar with its top-center at `anchor` and applies what the user clicks to `state`.
pub fn show(ui: &mut Ui, anchor: Pos2, state: &mut ViewportState) {
    let aspect = state.aspect();
    egui::Area::new(Id::new("viewport-toolbar"))
        .order(egui::Order::Middle)
        .pivot(Align2::CENTER_TOP)
        .fixed_pos(anchor)
        .show(ui.ctx(), |ui| {
            frames::overlay(4)
                .inner_margin(Margin::symmetric(6, 4))
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        ui.spacing_mut().item_spacing.x = 4.0;
                        let frame = IconButton::new(icons::TARGET)
                            .sizes(16.0, 26.0)
                            .fill(None)
                            .tooltip(tr("view3d.frame"))
                            .show(ui);
                        if frame.clicked() {
                            state.frame_scene(aspect);
                        }
                        divider(ui);
                        for preset in ViewPreset::ALL {
                            let (label, tip) = preset_labels(preset);
                            let response = Button::new(label, ButtonStyle::Quiet)
                                .padding(6.0, 4.0)
                                .show(ui)
                                .on_hover_text(tip);
                            if response.clicked() {
                                state.show_preset(preset, aspect);
                            }
                        }
                        divider(ui);
                        let options = &mut state.options;
                        if toggle(
                            ui,
                            tr("view3d.grid"),
                            options.show_grid,
                            tr("view3d.tip_grid"),
                        ) {
                            options.show_grid = !options.show_grid;
                        }
                        if toggle(
                            ui,
                            tr("view3d.axes"),
                            options.show_axes,
                            tr("view3d.tip_axes"),
                        ) {
                            options.show_axes = !options.show_axes;
                        }
                        divider(ui);
                        readouts(ui, state);
                    });
                });
        });
}

fn readouts(ui: &mut Ui, state: &mut ViewportState) {
    let speed = format!("{:.1}", state.controller.speed());
    Tag::new(trf("view3d.speed", &[&speed]), colors::ON_SURFACE_VARIANT)
        .padding(6.0, 2.0)
        .show(ui);
    let stats = state.stats();
    if stats.triangles > 0 {
        let triangles = with_thousands(stats.triangles as f32, 0);
        Tag::new(trf("view3d.stats", &[&triangles]), colors::OUTLINE)
            .padding(6.0, 2.0)
            .show(ui);
    }
    if stats.skipped_meshes > 0 {
        let skipped = stats.skipped_meshes.to_string();
        Tag::new(trf("view3d.skipped", &[&skipped]), colors::WARNING)
            .padding(6.0, 2.0)
            .show(ui);
    }
    if let Some(notice) = state.notice.as_ref() {
        if notice.at.elapsed() < NOTICE_TIME {
            Tag::new(notice.text.clone(), colors::TERTIARY)
                .padding(6.0, 2.0)
                .show(ui);
        } else {
            state.notice = None;
        }
    }
}
