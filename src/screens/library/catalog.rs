//! Library screen: header, filters, catalog grid and footer.

use super::super::ScreenContext;
use crate::config::theme::{colors, typography};
use crate::features::library::components::asset_card::CardAction;
use crate::features::library::components::{asset_card, filter_bar, footer_bar, header_bar};
use crate::features::library::logic;
use crate::features::library::models::{LibraryAsset, LibraryState};
use crate::models::navigation::Screen;
use crate::services::localization::tr;
use crate::services::sample_data::library as sample;
use crate::utils::grid;
use eframe::egui::{self, Margin, ScrollArea, Ui};

/// State of the library screen.
pub struct LibraryScreen {
    state: LibraryState,
    assets: Vec<LibraryAsset>,
}

impl Default for LibraryScreen {
    fn default() -> Self {
        Self {
            state: LibraryState::default(),
            assets: sample::assets(),
        }
    }
}

impl LibraryScreen {
    /// Draws the screen.
    pub fn show(&mut self, ui: &mut Ui, context: &mut ScreenContext) {
        ui.spacing_mut().item_spacing.y = 0.0;
        header_bar::show(ui);
        ScrollArea::vertical()
            .auto_shrink([false, false])
            .show(ui, |ui| {
                egui::Frame::new()
                    .inner_margin(Margin::same(8))
                    .show(ui, |ui| {
                        ui.spacing_mut().item_spacing.y = 8.0;
                        let visible = logic::visible(&self.assets, &self.state);
                        filter_bar::show(ui, &mut self.state, visible.len());
                        self.grid(ui, &visible, context);
                        footer_bar::show(ui, &mut self.state.page);
                    });
            });
    }

    fn grid(&mut self, ui: &mut Ui, visible: &[usize], context: &mut ScreenContext) {
        let columns = if self.state.layout == 0 {
            grid::column_count(ui.available_width(), 250.0, 8.0, 4)
        } else {
            1
        };
        let width = grid::column_width(ui.available_width(), columns, 8.0);
        for row in visible.chunks(columns) {
            ui.horizontal_top(|ui| {
                ui.spacing_mut().item_spacing.x = 8.0;
                for &index in row {
                    let active = index == self.state.selected;
                    match asset_card::show(ui, width, &self.assets[index], active, context.textures)
                    {
                        Some(CardAction::Select) => self.state.selected = index,
                        Some(CardAction::Place) => *context.navigate = Some(Screen::Build),
                        Some(CardAction::Inspect) => {
                            self.state.selected = index;
                            *context.navigate = Some(Screen::AssetInspector);
                        }
                        None => {}
                    }
                }
            });
            ui.add_space(8.0);
        }
        if visible.is_empty() {
            crate::components::text::label(
                ui,
                tr("library.empty"),
                typography::body_md(),
                colors::OUTLINE,
            );
        }
    }
}
