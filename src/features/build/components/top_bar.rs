//! The build-mode header floating over the 3D view.

use crate::components::{
    frames, icon_button::IconButton, icons, layout::split_row, tag::Tag, text,
};
use crate::config::theme::{colors, typography};
use crate::services::localization::tr;
use eframe::egui::{Margin, Rect, Sense, Ui, Vec2};

/// What the user asked for in the header.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TopBarAction {
    /// Leave build mode.
    Close,
}

fn divider(ui: &mut Ui) {
    let (rect, _) = ui.allocate_exact_size(Vec2::new(1.0, 14.0), Sense::hover());
    ui.painter().rect_filled(rect, 0.0, colors::SURFACE_HIGHEST);
}

/// Draws the header inside `area`; returns the requested action, if any.
pub fn show(ui: &mut Ui, area: Rect, scene: &str) -> Option<TopBarAction> {
    let mut action = None;
    let mut row = ui.new_child(eframe::egui::UiBuilder::new().max_rect(area));
    row.spacing_mut().item_spacing.y = 0.0;
    split_row(
        &mut row,
        40.0,
        |ui| {
            frames::overlay(0)
                .inner_margin(Margin::symmetric(12, 6))
                .show(ui, |ui| {
                    ui.spacing_mut().item_spacing.x = 8.0;
                    if IconButton::new(icons::CLOSE)
                        .sizes(16.0, 24.0)
                        .fill(None)
                        .tooltip(tr("build.close"))
                        .show(ui)
                        .clicked()
                    {
                        action = Some(TopBarAction::Close);
                    }
                    divider(ui);
                    text::icon(ui, icons::BUILD, 16.0, colors::PRIMARY);
                    ui.spacing_mut().item_spacing.x = 6.0;
                    text::label(
                        ui,
                        tr("build.title"),
                        typography::headline_sm(),
                        colors::ON_SURFACE,
                    );
                    Tag::new(scene, colors::ON_SURFACE_VARIANT)
                        .fill(colors::SURFACE_CONTAINER)
                        .padding(6.0, 2.0)
                        .show(ui);
                });
        },
        |ui| {
            frames::overlay(4).show(ui, |ui| {
                ui.spacing_mut().item_spacing.x = 4.0;
                let button = |glyph| IconButton::new(glyph).sizes(16.0, 28.0).fill(None);
                button(icons::FULLSCREEN)
                    .tooltip(tr("build.fullscreen"))
                    .show(ui);
                button(icons::VIDEOCAM).tooltip(tr("build.camera")).show(ui);
                divider(ui);
                button(icons::NOTIFICATIONS)
                    .badge(true)
                    .tooltip(tr("build.notifications"))
                    .show(ui);
                button(icons::SEARCH).tooltip(tr("build.search")).show(ui);
            });
        },
    );
    action
}
