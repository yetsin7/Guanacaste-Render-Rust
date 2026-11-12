//! The strip under the header: filter chips, resource readouts and the layout toggle.

use crate::components::{chip::Chip, icons, layout::split_row, segmented::Segmented};
use crate::config::theme::{colors, typography};
use crate::features::projects::logic;
use crate::features::projects::models::{
    ProjectFilter, ProjectLayout, ProjectSummary, ProjectsState,
};
use crate::services::localization::tr;
use eframe::egui::{self, Margin, Ui};

/// Draws the filter strip and applies clicks to `state`.
pub fn show(ui: &mut Ui, state: &mut ProjectsState, projects: &[ProjectSummary]) {
    egui::Frame::new()
        .fill(colors::SURFACE_LOWEST)
        .inner_margin(Margin::symmetric(16, 4))
        .show(ui, |ui| {
            split_row(
                ui,
                28.0,
                |ui| {
                    ui.spacing_mut().item_spacing.x = 4.0;
                    let queued = format!("({})", logic::queued_count(projects));
                    if Chip::new(tr("projects.filter_all"))
                        .count(tr("projects.filter_all_badge"))
                        .selected(state.filter == ProjectFilter::All)
                        .show(ui)
                        .clicked()
                    {
                        state.filter = ProjectFilter::All;
                    }
                    let tabs = [
                        (
                            ProjectFilter::RenderQueue,
                            "projects.filter_queue",
                            icons::CHECK_CIRCLE,
                            colors::TERTIARY,
                            Some(queued),
                        ),
                        (
                            ProjectFilter::CloudSynced,
                            "projects.filter_cloud",
                            icons::CLOUD,
                            colors::SECONDARY,
                            None,
                        ),
                        (
                            ProjectFilter::Archived,
                            "projects.filter_archived",
                            icons::ARCHIVE,
                            colors::ON_SURFACE_VARIANT,
                            None,
                        ),
                    ];
                    for (filter, key, glyph, color, count) in tabs {
                        let mut chip = Chip::new(tr(key))
                            .icon(glyph, color)
                            .selected(state.filter == filter);
                        if let Some(count) = count.as_deref() {
                            chip = chip.count(count);
                        }
                        if chip.show(ui).clicked() {
                            state.filter = if state.filter == filter {
                                ProjectFilter::All
                            } else {
                                filter
                            };
                        }
                    }
                },
                |ui| {
                    ui.spacing_mut().item_spacing.x = 12.0;
                    let mut layout = usize::from(state.layout == ProjectLayout::List);
                    if Segmented::icons(&[icons::GRID_VIEW, icons::VIEW_LIST])
                        .height(28.0)
                        .show(ui, &mut layout)
                    {
                        state.layout = if layout == 0 {
                            ProjectLayout::Grid
                        } else {
                            ProjectLayout::List
                        };
                    }
                    if ui.max_rect().width() > 1300.0 {
                        readout(
                            ui,
                            icons::STORAGE,
                            colors::PRIMARY,
                            "projects.cache",
                            "projects.cache_value",
                            colors::ON_SURFACE_VARIANT,
                        );
                        readout(
                            ui,
                            icons::MEMORY,
                            colors::TERTIARY,
                            "projects.vram",
                            "projects.vram_value",
                            colors::TERTIARY,
                        );
                    }
                },
            );
        });
}

/// `icon  Caption value`, painted left to right even inside a right-to-left row.
fn readout(
    ui: &mut Ui,
    glyph: char,
    glyph_color: egui::Color32,
    caption: &'static str,
    value: &'static str,
    value_color: egui::Color32,
) {
    crate::components::tag::Tag::new(tr(caption), colors::ON_SURFACE)
        .part(format!(" {}", tr(value)), value_color)
        .lead_icon(glyph, 14.0, glyph_color)
        .font(typography::label_sm())
        .gap(4.0)
        .padding(0.0, 0.0)
        .show(ui);
}
