//! Help and diagnostics screen.

use super::ScreenContext;
use crate::components::icons;
use crate::config::theme::colors;
use crate::features::help::components::{
    header_bar, input_mapping, knowledge_base, shortcut_group, status_bar, tutorial_card,
};
use crate::features::help::models::{HelpState, Shortcut, TutorialStep};
use crate::models::navigation::Screen;
use crate::services::sample_data::help as sample;
use eframe::egui::{self, vec2, Margin, ScrollArea, Ui};

const MAX_WIDTH: f32 = 1280.0;

/// State of the help screen.
pub struct HelpScreen {
    state: HelpState,
    steps: Vec<TutorialStep>,
    placement: Vec<Shortcut>,
    transform: Vec<Shortcut>,
}

impl Default for HelpScreen {
    fn default() -> Self {
        Self {
            state: HelpState::default(),
            steps: sample::steps(),
            placement: sample::placement_shortcuts(),
            transform: sample::transform_shortcuts(),
        }
    }
}

impl HelpScreen {
    /// Draws the screen.
    pub fn show(&mut self, ui: &mut Ui, context: &mut ScreenContext) {
        ScrollArea::vertical()
            .auto_shrink([false, false])
            .show(ui, |ui| {
                egui::Frame::new()
                    .inner_margin(Margin::same(12))
                    .show(ui, |ui| {
                        let total = ui.available_width();
                        let width = total.min(MAX_WIDTH);
                        ui.horizontal_top(|ui| {
                            ui.add_space((total - width) / 2.0);
                            ui.vertical(|ui| {
                                ui.set_width(width);
                                ui.spacing_mut().item_spacing.y = 12.0;
                                if header_bar::show(ui, &mut self.state.search) {
                                    *context.navigate = Some(Screen::Projects);
                                }
                                self.columns(ui, width, context);
                                status_bar::show(ui);
                            });
                        });
                    });
            });
    }

    fn columns(&mut self, ui: &mut Ui, width: f32, context: &mut ScreenContext) {
        let gap = 12.0;
        if width < 1000.0 {
            ui.spacing_mut().item_spacing.y = gap;
            self.left(ui, context);
            self.right(ui);
            return;
        }
        let column = (width - gap * 11.0) / 12.0;
        let left = (column * 5.0 + gap * 4.0).floor();
        let right = width - left - gap;
        ui.horizontal_top(|ui| {
            ui.spacing_mut().item_spacing = vec2(gap, gap);
            ui.allocate_ui_with_layout(
                vec2(left, 0.0),
                egui::Layout::top_down(egui::Align::Min),
                |ui| {
                    ui.set_width(left);
                    ui.spacing_mut().item_spacing.y = gap;
                    self.left(ui, context);
                },
            );
            ui.allocate_ui_with_layout(
                vec2(right, 0.0),
                egui::Layout::top_down(egui::Align::Min),
                |ui| {
                    ui.set_width(right);
                    ui.spacing_mut().item_spacing.y = gap;
                    self.right(ui);
                },
            );
        });
    }

    fn left(&mut self, ui: &mut Ui, context: &mut ScreenContext) {
        tutorial_card::show(ui, &mut self.state, &self.steps, context.textures);
        knowledge_base::show(ui);
    }

    fn right(&mut self, ui: &mut Ui) {
        input_mapping::show(ui, &mut self.state);
        let half = (ui.available_width() - 12.0) / 2.0;
        ui.horizontal_top(|ui| {
            ui.spacing_mut().item_spacing.x = 12.0;
            ui.allocate_ui_with_layout(
                vec2(half, 0.0),
                egui::Layout::top_down(egui::Align::Min),
                |ui| {
                    ui.set_width(half);
                    shortcut_group::show(
                        ui,
                        "help.group_placement",
                        colors::TERTIARY,
                        icons::NATURE,
                        &self.placement,
                        &self.state.search,
                    );
                },
            );
            ui.allocate_ui_with_layout(
                vec2(half, 0.0),
                egui::Layout::top_down(egui::Align::Min),
                |ui| {
                    ui.set_width(half);
                    shortcut_group::show(
                        ui,
                        "help.group_transform",
                        colors::PRIMARY,
                        icons::MOVE,
                        &self.transform,
                        &self.state.search,
                    );
                },
            );
        });
    }
}
