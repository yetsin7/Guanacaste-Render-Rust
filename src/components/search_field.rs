//! Search input with a leading icon and an optional shortcut hint.

use super::{icons, key_cap::KeyCap, text};
use crate::config::theme::{colors, typography};
use eframe::egui::{
    self, Align, Color32, CornerRadius, FontId, Layout, Margin, Rect, RichText, Sense, TextEdit,
    Ui, UiBuilder, Vec2,
};

/// A single-line search box.
pub struct SearchField<'a> {
    hint: &'a str,
    width: f32,
    height: f32,
    shortcut: Option<&'a str>,
    badges: &'a [(&'a str, Color32, Color32)],
    fill: Color32,
    font: FontId,
    icon_size: f32,
}

impl<'a> SearchField<'a> {
    /// A 288 px wide, 32 px tall field with the placeholder `hint`.
    pub fn new(hint: &'a str) -> Self {
        Self {
            hint,
            width: 288.0,
            height: 32.0,
            shortcut: None,
            badges: &[],
            fill: colors::SURFACE_CONTAINER,
            font: typography::body_sm(),
            icon_size: 16.0,
        }
    }

    /// Sets the total width.
    pub fn width(mut self, width: f32) -> Self {
        self.width = width;
        self
    }

    /// Sets the total height.
    pub fn height(mut self, height: f32) -> Self {
        self.height = height;
        self
    }

    /// Shows a key hint such as `⌘K` at the right edge.
    pub fn shortcut(mut self, shortcut: &'a str) -> Self {
        self.shortcut = Some(shortcut);
        self
    }

    /// Shows small `(text, fill, ink)` badges at the right edge, last one rightmost.
    pub fn badges(mut self, badges: &'a [(&'a str, Color32, Color32)]) -> Self {
        self.badges = badges;
        self
    }

    /// Sets the text font and the icon size.
    pub fn font(mut self, font: FontId, icon_size: f32) -> Self {
        self.font = font;
        self.icon_size = icon_size;
        self
    }

    /// Sets the background fill.
    pub fn fill(mut self, fill: Color32) -> Self {
        self.fill = fill;
        self
    }

    /// Draws the field editing `value`; returns true when the text changed.
    pub fn show(self, ui: &mut Ui, value: &mut String) -> bool {
        let (rect, _) = ui.allocate_exact_size(Vec2::new(self.width, self.height), Sense::hover());
        ui.painter()
            .rect_filled(rect, CornerRadius::same(2), self.fill);
        text::paint_icon(
            ui.painter(),
            egui::pos2(rect.left() + 8.0 + self.icon_size / 2.0, rect.center().y),
            icons::SEARCH,
            self.icon_size,
            colors::OUTLINE,
        );
        let badge_width: f32 = self
            .badges
            .iter()
            .map(|(label, _, _)| label.chars().count() as f32 * 6.6 + 16.0)
            .sum();
        let right_reserved = if self.shortcut.is_some() { 44.0 } else { 8.0 } + badge_width;
        let edit_rect = Rect::from_min_max(
            egui::pos2(rect.left() + 16.0 + self.icon_size + 8.0, rect.top()),
            egui::pos2(rect.right() - right_reserved, rect.bottom()),
        );
        let mut edit_ui = ui.new_child(
            UiBuilder::new()
                .max_rect(edit_rect)
                .layout(Layout::left_to_right(Align::Center)),
        );
        let edit = TextEdit::singleline(value)
            .frame(egui::Frame::NONE)
            .font(self.font.clone())
            .text_color(colors::ON_SURFACE)
            .hint_text(RichText::new(self.hint).color(colors::OUTLINE))
            .margin(Margin::ZERO)
            .desired_width(edit_rect.width());
        let changed = edit_ui.add(edit).changed();
        if let Some(shortcut) = self.shortcut {
            let key_rect = Rect::from_min_max(
                egui::pos2(rect.right() - 40.0, rect.top()),
                egui::pos2(rect.right() - 6.0, rect.bottom()),
            );
            let mut key_ui = ui.new_child(
                UiBuilder::new()
                    .max_rect(key_rect)
                    .layout(Layout::right_to_left(Align::Center)),
            );
            KeyCap::new(shortcut).small().show(&mut key_ui);
        }
        if !self.badges.is_empty() {
            let area = Rect::from_min_max(
                egui::pos2(rect.right() - badge_width - 8.0, rect.top()),
                egui::pos2(rect.right() - 8.0, rect.bottom()),
            );
            let mut badge_ui = ui.new_child(
                UiBuilder::new()
                    .max_rect(area)
                    .layout(Layout::left_to_right(Align::Center)),
            );
            badge_ui.spacing_mut().item_spacing.x = 4.0;
            for (label, fill, ink) in self.badges {
                super::tag::Tag::new(*label, *ink)
                    .fill(*fill)
                    .padding(6.0, 2.0)
                    .show(&mut badge_ui);
            }
        }
        changed
    }
}
