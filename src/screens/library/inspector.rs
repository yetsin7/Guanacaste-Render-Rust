//! Asset inspector screen: preview, dimensions, spectrum chart, metadata and seasonal presets.

use super::super::ScreenContext;
use crate::features::library::components::{
    absorption_chart, inspector_header, metadata_panel, metric_cards, preview_panel,
    seasonal_presets,
};
use crate::features::library::models::{AssetDetails, InspectorState};
use crate::models::navigation::Screen;
use crate::services::sample_data::inspector as sample;
use eframe::egui::{self, vec2, Margin, ScrollArea, Ui};

/// State of the inspector screen.
pub struct InspectorScreen {
    state: InspectorState,
    details: AssetDetails,
    filter: String,
}

impl Default for InspectorScreen {
    fn default() -> Self {
        Self {
            state: InspectorState::default(),
            details: sample::details(),
            filter: String::new(),
        }
    }
}

impl InspectorScreen {
    /// Draws the screen.
    pub fn show(&mut self, ui: &mut Ui, context: &mut ScreenContext) {
        ui.spacing_mut().item_spacing.y = 0.0;
        inspector_header::show(ui, &mut self.filter);
        ScrollArea::vertical()
            .auto_shrink([false, false])
            .show(ui, |ui| {
                egui::Frame::new()
                    .inner_margin(Margin::same(8))
                    .show(ui, |ui| {
                        let total = ui.available_width();
                        let gap = 8.0;
                        if total < 1000.0 {
                            ui.spacing_mut().item_spacing.y = gap;
                            self.left(ui, context);
                            self.right(ui);
                            return;
                        }
                        let column = (total - gap * 11.0) / 12.0;
                        let left = (column * 8.0 + gap * 7.0).floor();
                        let right = total - left - gap;
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
                    });
            });
    }

    fn left(&mut self, ui: &mut Ui, context: &mut ScreenContext) {
        if let Some(preview_panel::PreviewAction::Place) =
            preview_panel::show(ui, &self.details, &mut self.state, context.textures)
        {
            *context.navigate = Some(Screen::Build);
        }
        metric_cards::show(ui, &self.details);
        absorption_chart::show(ui, &self.details);
    }

    fn right(&mut self, ui: &mut Ui) {
        metadata_panel::show(ui, &self.details, &mut self.state);
        seasonal_presets::show(ui, &mut self.state);
    }
}
