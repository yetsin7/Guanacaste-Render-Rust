//! Settings screen: render engine, precision controls and profiles on the left; feedback and telemetry on the right.

use super::ScreenContext;
use crate::features::settings::components::header_bar::{self, HeaderAction};
use crate::features::settings::components::{
    feedback_panel, precision_panel, profiles_bar, render_engine_panel, telemetry_panel,
};
use crate::features::settings::models::SettingsState;
use eframe::egui::{self, vec2, Margin, ScrollArea, Ui};

/// State of the settings screen.
#[derive(Default)]
pub struct SettingsScreen {
    state: SettingsState,
}

impl SettingsScreen {
    /// Draws the screen.
    pub fn show(&mut self, ui: &mut Ui, context: &mut ScreenContext) {
        ui.spacing_mut().item_spacing.y = 0.0;
        match header_bar::show(ui, &mut self.state) {
            Some(HeaderAction::Defaults) => self.state = SettingsState::default(),
            Some(HeaderAction::Apply) => self.state.applied = true,
            None => {}
        }
        ScrollArea::vertical()
            .auto_shrink([false, false])
            .show(ui, |ui| {
                egui::Frame::new()
                    .inner_margin(Margin::same(16))
                    .show(ui, |ui| {
                        let total = ui.available_width();
                        let gap = 16.0;
                        if total < 1000.0 {
                            ui.spacing_mut().item_spacing.y = gap;
                            self.left(ui);
                            self.right(ui, context);
                            return;
                        }
                        let column = (total - gap * 11.0) / 12.0;
                        let left = (column * 7.0 + gap * 6.0).floor();
                        let right = total - left - gap;
                        ui.horizontal_top(|ui| {
                            ui.spacing_mut().item_spacing = vec2(gap, gap);
                            ui.allocate_ui_with_layout(
                                vec2(left, 0.0),
                                egui::Layout::top_down(egui::Align::Min),
                                |ui| {
                                    ui.set_width(left);
                                    ui.spacing_mut().item_spacing.y = gap;
                                    self.left(ui);
                                },
                            );
                            ui.allocate_ui_with_layout(
                                vec2(right, 0.0),
                                egui::Layout::top_down(egui::Align::Min),
                                |ui| {
                                    ui.set_width(right);
                                    ui.spacing_mut().item_spacing.y = gap;
                                    self.right(ui, context);
                                },
                            );
                        });
                    });
            });
    }

    fn left(&mut self, ui: &mut Ui) {
        render_engine_panel::show(ui, &mut self.state);
        precision_panel::show(ui, &mut self.state);
        profiles_bar::show(ui, &mut self.state);
    }

    fn right(&mut self, ui: &mut Ui, context: &mut ScreenContext) {
        feedback_panel::show(ui, context.textures);
        telemetry_panel::show(ui);
    }
}
