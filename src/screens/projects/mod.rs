//! The projects screen: repository header, filters, active-project hero, project grid and batch dispatcher.

use super::ScreenContext;
use crate::components::{
    button::{Button, ButtonStyle},
    icon_button::IconButton,
    icons,
    layout::split_row,
    section_title,
};
use crate::config::theme::colors;
use crate::features::projects::components::hero_panel::HeroAction;
use crate::features::projects::components::{
    batch_dispatcher, hero_panel, project_card, project_row,
};
use crate::features::projects::models::{
    ActiveProject, ProjectLayout, ProjectSummary, ProjectsState,
};
use crate::features::projects::{logic, ui as projects_ui};
use crate::models::navigation::Screen;
use crate::services::localization::tr;
use crate::services::sample_data::projects as sample;
use crate::utils::grid;
use eframe::egui::{self, Margin, ScrollArea, Ui};

/// State of the projects screen.
pub struct ProjectsScreen {
    state: ProjectsState,
    active: ActiveProject,
    projects: Vec<ProjectSummary>,
}

impl Default for ProjectsScreen {
    fn default() -> Self {
        Self {
            state: ProjectsState::default(),
            active: sample::active_project(),
            projects: sample::projects(),
        }
    }
}

impl ProjectsScreen {
    /// Draws the screen.
    pub fn show(&mut self, ui: &mut Ui, context: &mut ScreenContext) {
        ui.spacing_mut().item_spacing.y = 0.0;
        if projects_ui::command_bar::show(ui, &mut self.state) {
            *context.navigate = Some(Screen::Build);
        }
        projects_ui::filter_strip::show(ui, &mut self.state, &self.projects);
        ScrollArea::vertical()
            .auto_shrink([false, false])
            .show(ui, |ui| {
                egui::Frame::new()
                    .inner_margin(Margin::same(16))
                    .show(ui, |ui| {
                        self.content(ui, context);
                    });
            });
    }

    fn content(&mut self, ui: &mut Ui, context: &mut ScreenContext) {
        let visible = logic::visible(&self.projects, &self.state);
        split_row(
            ui,
            32.0,
            |ui| section_title::with_count(ui, tr("projects.recent"), Some("12")),
            |ui| {
                ui.spacing_mut().item_spacing.x = 8.0;
                IconButton::new(icons::MORE_VERT)
                    .fill(Some(colors::SURFACE_LOW))
                    .tooltip(tr("projects.more"))
                    .show(ui);
                Button::new(tr("projects.view_all"), ButtonStyle::Link)
                    .icon(icons::ARROW_FORWARD, 14.0)
                    .show(ui);
            },
        );
        ui.add_space(16.0);
        match hero_panel::show(ui, &self.active, context.textures) {
            Some(HeroAction::OpenViewport) => *context.navigate = Some(Screen::Build),
            Some(HeroAction::Render) => *context.navigate = Some(Screen::Render),
            Some(HeroAction::Settings) => *context.navigate = Some(Screen::Settings),
            None => {}
        }
        ui.add_space(16.0);
        if visible.is_empty() {
            crate::components::text::label(
                ui,
                tr("projects.empty"),
                crate::config::theme::typography::body_md(),
                colors::OUTLINE,
            );
        }
        let mut load = false;
        match self.state.layout {
            ProjectLayout::Grid => {
                let columns = grid::column_count(ui.available_width(), 270.0, 12.0, 4);
                let width = grid::column_width(ui.available_width(), columns, 12.0);
                for row in visible.chunks(columns) {
                    ui.horizontal_top(|ui| {
                        ui.spacing_mut().item_spacing.x = 12.0;
                        for project in row {
                            load |= project_card::show(ui, width, project);
                        }
                    });
                    ui.add_space(12.0);
                }
            }
            ProjectLayout::List => {
                for project in &visible {
                    load |= project_row::show(ui, project);
                    ui.add_space(4.0);
                }
                ui.add_space(8.0);
            }
        }
        if load {
            *context.navigate = Some(Screen::Build);
        }
        ui.add_space(4.0);
        if batch_dispatcher::show(ui) {
            *context.navigate = Some(Screen::Render);
        }
    }
}
