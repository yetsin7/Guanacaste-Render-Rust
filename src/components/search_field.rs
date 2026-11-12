//! Search input with a leading icon and an optional shortcut hint.

use super::{icons, key_cap::KeyCap, text};
use crate::config::theme::{colors, typography};
use eframe::egui::{
    self, Align, Color32, CornerRadius, Layout, Margin, Rect, RichText, Sense, TextEdit, Ui,
    UiBuilder, Vec2,
};

/// A single-line search box.
pub struct SearchField<'a> {
    hint: &'a str,
    width: f32,
    height: f32,
    shortcut: Option<&'a str>,
    fill: Color32,
}

impl<'a> SearchField<'a> {
    /// A 288 px wide, 32 px tall field with the placeholder `hint`.
    pub fn new(hint: &'a str) -> Self {
        Self {
            hint,
            width: 288.0,
            height: 32.0,
            shortcut: None,
            fill: colors::SURFACE_CONTAINER,
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
            egui::pos2(rect.left() + 8.0 + 8.0, rect.center().y),
            icons::SEARCH,
            16.0,
            colors::OUTLINE,
        );
        let right_reserved = if self.shortcut.is_some() { 44.0 } else { 8.0 };
        let edit_rect = Rect::from_min_max(
            egui::pos2(rect.left() + 32.0, rect.top()),
            egui::pos2(rect.right() - right_reserved, rect.bottom()),
        );
        let mut edit_ui = ui.new_child(
            UiBuilder::new()
                .max_rect(edit_rect)
                .layout(Layout::left_to_right(Align::Center)),
        );
        let edit = TextEdit::singleline(value)
            .frame(egui::Frame::NONE)
            .font(typography::body_sm())
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
        changed
    }
}
