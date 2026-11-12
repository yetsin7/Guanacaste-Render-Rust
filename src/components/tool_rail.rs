//! The 48 px left tool rail: workspaces at the top, performance, settings and help at the bottom.

use super::{icons, text};
use crate::config::theme::colors;
use crate::models::navigation::RailItem;
use crate::services::localization::tr;
use eframe::egui::{CornerRadius, Pos2, Rect, Sense, Ui, Vec2};

/// Width of the rail in points.
pub const WIDTH: f32 = 48.0;
const BUTTON: f32 = 40.0;
const GAP: f32 = 2.0;
const PADDING: f32 = 4.0;

const TOP_ITEMS: [(RailItem, char, &str); 5] = [
    (RailItem::Projects, icons::FOLDER_OPEN, "rail.projects"),
    (RailItem::Build, icons::BUILD, "rail.build"),
    (RailItem::Library, icons::LIBRARY, "rail.library"),
    (RailItem::Movie, icons::MOVIE, "rail.movie"),
    (RailItem::Render, icons::CAMERA, "rail.render"),
];
const BOTTOM_ITEMS: [(RailItem, char, &str); 3] = [
    (RailItem::Performance, icons::SPEED, "rail.performance"),
    (RailItem::Settings, icons::SETTINGS, "rail.settings"),
    (RailItem::Help, icons::HELP, "rail.help"),
];

/// Vertical offset of the `index`-th button of a group, from the group's first button.
pub fn button_offset(index: usize) -> f32 {
    index as f32 * (BUTTON + GAP)
}

/// Draws the rail inside `ui` and returns the entry that was clicked, if any. `active` is highlighted.
pub fn show(ui: &mut Ui, active: RailItem) -> Option<RailItem> {
    let full = ui.max_rect();
    let mut clicked = None;
    let x = full.center().x;
    for (index, (item, glyph, key)) in TOP_ITEMS.into_iter().enumerate() {
        let top = full.top() + PADDING + button_offset(index);
        if button(
            ui,
            Pos2::new(x, top + BUTTON / 2.0),
            glyph,
            20.0,
            item == active,
            key,
        ) {
            clicked = Some(item);
        }
    }
    let group_height = BOTTOM_ITEMS.len() as f32 * BUTTON + (BOTTOM_ITEMS.len() - 1) as f32 * GAP;
    let first = full.bottom() - PADDING - group_height;
    ui.painter().rect_filled(
        Rect::from_min_size(
            Pos2::new(full.left(), first - PADDING - 1.0),
            Vec2::new(full.width(), 1.0),
        ),
        0.0,
        colors::SURFACE_HIGHEST,
    );
    for (index, (item, glyph, key)) in BOTTOM_ITEMS.into_iter().enumerate() {
        let top = first + button_offset(index);
        // The performance button opens settings but only the settings button shows as active.
        let is_active = item == active && item != RailItem::Performance;
        if button(
            ui,
            Pos2::new(x, top + BUTTON / 2.0),
            glyph,
            18.0,
            is_active,
            key,
        ) {
            clicked = Some(item);
        }
    }
    clicked
}

fn button(
    ui: &mut Ui,
    center: Pos2,
    glyph: char,
    size: f32,
    active: bool,
    tooltip: &'static str,
) -> bool {
    let rect = Rect::from_center_size(center, Vec2::splat(BUTTON));
    let response = ui
        .interact(rect, ui.id().with(tooltip), Sense::click())
        .on_hover_text(tr(tooltip))
        .on_hover_cursor(eframe::egui::CursorIcon::PointingHand);
    let painter = ui.painter();
    if active {
        painter.rect_filled(rect, CornerRadius::same(2), colors::SURFACE_HIGH);
        painter.rect_filled(
            Rect::from_min_size(rect.min, Vec2::new(2.0, rect.height())),
            0.0,
            colors::PRIMARY_CONTAINER,
        );
    } else if response.hovered() {
        painter.rect_filled(rect, CornerRadius::same(2), colors::SURFACE_CONTAINER);
    }
    let color = if active {
        colors::PRIMARY
    } else if response.hovered() {
        colors::ON_SURFACE
    } else {
        colors::ON_SURFACE_VARIANT
    };
    text::paint_icon(painter, rect.center(), glyph, size, color);
    response.clicked()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn buttons_are_spaced_by_button_plus_gap() {
        assert_eq!(button_offset(0), 0.0);
        assert_eq!(button_offset(2), 84.0);
    }
}
