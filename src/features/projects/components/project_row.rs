//! A project as a compact row, for the list layout.

use super::project_card::thumbnail_size;
use crate::components::{
    button::{Button, ButtonStyle},
    icons, scene_art, text,
};
use crate::config::theme::{colors, typography};
use crate::features::projects::models::ProjectSummary;
use crate::services::localization::tr;
use eframe::egui::{pos2, vec2, Align, CornerRadius, Layout, Rect, Sense, Ui, UiBuilder};

/// Height of a row in points.
pub const HEIGHT: f32 = 60.0;

/// Draws the row; returns true when "Load project" was clicked.
pub fn show(ui: &mut Ui, project: &ProjectSummary) -> bool {
    let (rect, _) = ui.allocate_exact_size(vec2(ui.available_width(), HEIGHT), Sense::hover());
    let painter = ui.painter().clone();
    painter.rect_filled(rect, CornerRadius::same(2), colors::SURFACE_LOW);
    let thumb = Rect::from_min_size(
        pos2(
            rect.left() + 8.0,
            rect.center().y - thumbnail_size().y / 2.0,
        ),
        thumbnail_size(),
    );
    scene_art::paint_scene(&painter, thumb, &project.art);
    let mut row = ui.new_child(
        UiBuilder::new()
            .max_rect(Rect::from_min_max(
                pos2(thumb.right() + 12.0, rect.top()),
                pos2(rect.right() - 12.0, rect.bottom()),
            ))
            .layout(Layout::left_to_right(Align::Center)),
    );
    row.spacing_mut().item_spacing.x = 16.0;
    row.allocate_ui_with_layout(vec2(280.0, HEIGHT), Layout::top_down(Align::Min), |ui| {
        ui.add_space(12.0);
        ui.spacing_mut().item_spacing.y = 2.0;
        text::label(
            ui,
            project.title,
            typography::headline_sm(),
            colors::ON_SURFACE,
        );
        text::label(ui, project.id, typography::label_sm(), colors::OUTLINE);
    });
    text::label(
        &mut row,
        project.size,
        typography::label_sm(),
        colors::OUTLINE,
    );
    text::label(
        &mut row,
        project.engine,
        typography::label_sm(),
        colors::ON_SURFACE,
    );
    text::label(
        &mut row,
        format!("{} FPS", project.fps),
        typography::label_sm(),
        colors::SECONDARY,
    );
    row.with_layout(Layout::right_to_left(Align::Center), |ui| {
        Button::new(tr("projects.load"), ButtonStyle::Surface)
            .font(typography::label_sm())
            .icon(icons::FOLDER_OPEN, 14.0)
            .height(28.0)
            .show(ui)
            .clicked()
    })
    .inner
}
