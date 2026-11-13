//! The 44 px mode bar under the title bar, visible everywhere but the start screen: start screen on the left,
//! the four workspace modes (each with its number key) in the middle, and save, library, settings and help on
//! the right.

use super::{icon_button::IconButton, key_cap::KeyCap, text};
use crate::config::theme::{colors, typography};
use crate::models::dialog::Dialog;
use crate::models::navigation::{Mode, Screen};
use crate::services::localization::tr;
use eframe::egui::{pos2, vec2, Align, CornerRadius, CursorIcon, Layout, Rect, Sense, Ui};

/// Height of the bar in points.
pub const HEIGHT: f32 = 44.0;
const TAB_WIDTH: f32 = 132.0;

/// Icon and label key of each mode, in [`Mode::ALL`] order.
const MODES: [(char, &str); 4] = [
    (super::icons::BUILD, "mode.build"),
    (super::icons::CAMERA, "mode.photo"),
    (super::icons::MOVIE, "mode.movie"),
    (super::icons::IMAGE, "mode.export"),
];

/// Number-key labels of the modes.
const MODE_KEYS: [&str; 4] = ["F1", "F2", "F3", "F4"];

/// Secondary screens on the right: icon, tooltip key and target.
const SECONDARY: [(char, &str, Screen); 3] = [
    (super::icons::LIBRARY, "modebar.library", Screen::Library),
    (super::icons::SETTINGS, "modebar.settings", Screen::Settings),
    (super::icons::HELP, "modebar.help", Screen::Help),
];

/// Draws the bar inside `ui` (exactly [`HEIGHT`] tall). `current` is the screen shown and `mode` the active
/// workspace mode; returns the screen the user asked for.
pub fn show(ui: &mut Ui, current: Screen, mode: Mode) -> Option<Screen> {
    let bar = ui.max_rect();
    let mut target = None;
    ui.horizontal_centered(|ui| {
        ui.spacing_mut().item_spacing.x = 4.0;
        ui.add_space(8.0);
        if home_button(ui) {
            target = Some(Screen::Home);
        }
    });
    let total = TAB_WIDTH * MODES.len() as f32;
    let left = (bar.center().x - total / 2.0).max(bar.left() + 120.0);
    for (index, (glyph, key)) in MODES.into_iter().enumerate() {
        let rect = Rect::from_min_size(
            pos2(left + index as f32 * TAB_WIDTH, bar.top()),
            vec2(TAB_WIDTH, bar.height()),
        );
        let this = Mode::ALL[index];
        let active = current.mode() == Some(this);
        let remembered = current.mode().is_none() && mode == this;
        if mode_tab(
            ui,
            rect,
            glyph,
            tr(key),
            MODE_KEYS[index],
            active,
            remembered,
        ) {
            target = Some(Screen::Workspace(this));
        }
    }
    let right = Rect::from_min_max(pos2(left + total, bar.top()), bar.max);
    let mut row = ui.new_child(
        eframe::egui::UiBuilder::new()
            .max_rect(right)
            .layout(Layout::right_to_left(Align::Center)),
    );
    row.spacing_mut().item_spacing.x = 4.0;
    row.add_space(8.0);
    for (glyph, key, screen) in SECONDARY.into_iter().rev() {
        let button = IconButton::new(glyph)
            .sizes(18.0, 32.0)
            .fill(None)
            .active(current == screen)
            .tooltip(tr(key));
        if button.show(&mut row).clicked() {
            target = Some(screen);
        }
    }
    row.add_space(4.0);
    IconButton::new(super::icons::SAVE)
        .sizes(18.0, 32.0)
        .fill(None)
        .tooltip(tr("modebar.save"))
        .opens(Dialog::SaveProject)
        .show(&mut row);
    target
}

fn home_button(ui: &mut Ui) -> bool {
    let label = tr("modebar.home");
    let width = 28.0 + text::text_width(ui, label, &typography::label_md()) + 12.0;
    let (rect, response) = ui.allocate_exact_size(vec2(width, 32.0), Sense::click());
    let response = response
        .on_hover_cursor(CursorIcon::PointingHand)
        .on_hover_text(tr("modebar.home_hint"));
    let painter = ui.painter();
    if response.hovered() {
        painter.rect_filled(rect, CornerRadius::same(2), colors::SURFACE_HIGH);
    }
    let ink = if response.hovered() {
        colors::ON_SURFACE
    } else {
        colors::ON_SURFACE_VARIANT
    };
    text::paint_icon(
        painter,
        pos2(rect.left() + 16.0, rect.center().y),
        super::icons::FOLDER_OPEN,
        17.0,
        ink,
    );
    text::paint_text(
        painter,
        pos2(rect.left() + 30.0, rect.center().y),
        eframe::egui::Align2::LEFT_CENTER,
        label,
        typography::label_md(),
        ink,
    );
    response.clicked()
}

fn mode_tab(
    ui: &mut Ui,
    rect: Rect,
    glyph: char,
    label: &str,
    key: &str,
    active: bool,
    remembered: bool,
) -> bool {
    let response = ui
        .interact(rect, ui.id().with(("mode-tab", label)), Sense::click())
        .on_hover_cursor(CursorIcon::PointingHand);
    let painter = ui.painter();
    let cell = rect.shrink2(vec2(2.0, 6.0));
    if active {
        painter.rect_filled(cell, CornerRadius::same(3), colors::SURFACE_HIGH);
        painter.rect_filled(
            Rect::from_min_max(
                pos2(cell.left() + 12.0, rect.bottom() - 2.0),
                pos2(cell.right() - 12.0, rect.bottom()),
            ),
            0.0,
            colors::PRIMARY_CONTAINER,
        );
    } else if response.hovered() {
        painter.rect_filled(cell, CornerRadius::same(3), colors::SURFACE_CONTAINER);
    }
    let ink = if active {
        colors::PRIMARY
    } else if response.hovered() || remembered {
        colors::ON_SURFACE
    } else {
        colors::ON_SURFACE_VARIANT
    };
    let label_width = text::text_width(ui, label, &typography::label_lg());
    let content = 20.0 + 6.0 + label_width + 8.0 + 14.0;
    let start = rect.center().x - content / 2.0;
    text::paint_icon(
        painter,
        pos2(start + 10.0, rect.center().y),
        glyph,
        18.0,
        ink,
    );
    text::paint_text(
        painter,
        pos2(start + 26.0, rect.center().y),
        eframe::egui::Align2::LEFT_CENTER,
        label,
        typography::label_lg(),
        ink,
    );
    let key_rect = Rect::from_center_size(
        pos2(start + content - 7.0, rect.center().y),
        vec2(14.0, 16.0),
    );
    let mut key_ui = ui.new_child(eframe::egui::UiBuilder::new().max_rect(key_rect));
    KeyCap::new(key)
        .small()
        .padding(4.0, 1.0)
        .colors(colors::SURFACE_HIGHEST, colors::OUTLINE)
        .show(&mut key_ui);
    response.clicked()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_mode_has_a_tab_and_a_key() {
        assert_eq!(MODES.len(), Mode::ALL.len());
        assert_eq!(MODE_KEYS.len(), Mode::ALL.len());
    }
}
