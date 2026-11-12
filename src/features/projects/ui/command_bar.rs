//! The page header of the repository: title, session chip, search, notifications and "New scene".

use crate::components::{
    button::{Button, ButtonStyle},
    icon_button::IconButton,
    icons,
    layout::split_row,
    search_field::SearchField,
    tag::Tag,
    text,
};
use crate::config::theme::{colors, typography};
use crate::features::projects::models::ProjectsState;
use crate::services::localization::tr;
use eframe::egui::{self, Margin, Ui};

/// Draws the header; returns true when "New scene" was clicked.
pub fn show(ui: &mut Ui, state: &mut ProjectsState) -> bool {
    let mut new_scene = false;
    egui::Frame::new()
        .fill(colors::SURFACE_LOW)
        .inner_margin(Margin::symmetric(16, 8))
        .show(ui, |ui| {
            split_row(
                ui,
                32.0,
                |ui| {
                    ui.spacing_mut().item_spacing.x = 4.0;
                    text::label(
                        ui,
                        tr("projects.title"),
                        typography::headline_lg(),
                        colors::ON_SURFACE,
                    );
                    text::label(
                        ui,
                        text::caps(tr("projects.subtitle")),
                        typography::label_sm(),
                        colors::OUTLINE,
                    );
                    ui.add_space(8.0);
                    if ui.max_rect().width() < 1150.0 {
                        return;
                    }
                    Tag::new(tr("projects.session"), colors::ON_SURFACE)
                        .part(" / ", colors::OUTLINE)
                        .part(tr("projects.version"), colors::TERTIARY)
                        .dot(colors::TERTIARY)
                        .gap(4.0)
                        .padding(8.0, 4.0)
                        .fill(colors::SURFACE_HIGH)
                        .show(ui);
                },
                |ui| {
                    ui.spacing_mut().item_spacing.x = 8.0;
                    new_scene =
                        Button::new(text::caps(tr("projects.new_scene")), ButtonStyle::Pale)
                            .icon(icons::ADD_BOX, 16.0)
                            .font(typography::label_md())
                            .padding(8.0, 6.0)
                            .show(ui)
                            .clicked();
                    IconButton::bell()
                        .badge(true)
                        .tooltip(tr("projects.notifications"))
                        .show(ui);
                    SearchField::new(tr("projects.search_hint"))
                        .shortcut("⌘K")
                        .show(ui, &mut state.search);
                },
            );
        });
    new_scene
}
