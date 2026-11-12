//! Filter chips: a label with optional icon and count badge that toggles a selection.

use super::text;
use crate::config::theme::{colors, typography};
use eframe::egui::{Color32, CornerRadius, CursorIcon, Pos2, Response, Sense, Ui, Vec2};

/// A selectable filter chip.
pub struct Chip<'a> {
    label: &'a str,
    icon: Option<(char, Color32)>,
    count: Option<&'a str>,
    selected: bool,
    padding: Vec2,
}

impl<'a> Chip<'a> {
    /// A chip with the given label.
    pub fn new(label: &'a str) -> Self {
        Self {
            label,
            icon: None,
            count: None,
            selected: false,
            padding: Vec2::new(8.0, 4.0),
        }
    }

    /// Adds a leading icon of the given color (shown in `on-primary` when selected).
    pub fn icon(mut self, glyph: char, color: Color32) -> Self {
        self.icon = Some((glyph, color));
        self
    }

    /// Adds a small count after the label.
    pub fn count(mut self, count: &'a str) -> Self {
        self.count = Some(count);
        self
    }

    /// Marks the chip as selected.
    pub fn selected(mut self, selected: bool) -> Self {
        self.selected = selected;
        self
    }

    /// Sets the inner padding.
    pub fn padding(mut self, x: f32, y: f32) -> Self {
        self.padding = Vec2::new(x, y);
        self
    }

    /// Paints the chip and returns its click response.
    pub fn show(self, ui: &mut Ui) -> Response {
        let (fill, ink) = if self.selected {
            (colors::PRIMARY, colors::ON_PRIMARY)
        } else {
            (colors::SURFACE_LOW, colors::ON_SURFACE_VARIANT)
        };
        let font = typography::label_sm();
        let label = ui
            .painter()
            .layout_no_wrap(self.label.to_owned(), font, ink);
        let badge_font = typography::mono(10.0);
        let badge_color = if self.selected {
            colors::PRIMARY_FIXED
        } else {
            colors::OUTLINE
        };
        let badge = self.count.map(|count| {
            ui.painter()
                .layout_no_wrap(count.to_owned(), badge_font, badge_color)
        });
        let gap = 4.0;
        let icon_width = self.icon.map_or(0.0, |_| 14.0 + gap);
        let badge_width = badge.as_ref().map_or(0.0, |badge| {
            gap + badge.size().x + if self.selected { 12.0 } else { 0.0 }
        });
        let width = self.padding.x * 2.0 + icon_width + label.size().x + badge_width;
        let height = label.size().y.max(14.0) + self.padding.y * 2.0;
        let (rect, response) = ui.allocate_exact_size(Vec2::new(width, height), Sense::click());
        let response = response.on_hover_cursor(CursorIcon::PointingHand);
        let fill = if !self.selected && response.hovered() {
            colors::SURFACE_CONTAINER
        } else {
            fill
        };
        let painter = ui.painter();
        painter.rect_filled(rect, CornerRadius::same(2), fill);
        let mut x = rect.left() + self.padding.x;
        if let Some((glyph, color)) = self.icon {
            let color = if self.selected { ink } else { color };
            text::paint_icon(
                painter,
                Pos2::new(x + 7.0, rect.center().y),
                glyph,
                14.0,
                color,
            );
            x += icon_width;
        }
        let label_height = label.size().y;
        let label_width = label.size().x;
        painter.galley(
            Pos2::new(x, rect.center().y - label_height / 2.0),
            label,
            ink,
        );
        x += label_width + gap;
        if let Some(badge) = badge {
            let badge_size = badge.size();
            if self.selected {
                let pill = eframe::egui::Rect::from_min_size(
                    Pos2::new(x, rect.center().y - (badge_size.y + 2.0) / 2.0),
                    Vec2::new(badge_size.x + 12.0, badge_size.y + 2.0),
                );
                painter.rect_filled(pill, CornerRadius::same(2), colors::ON_PRIMARY_CONTAINER);
                painter.galley(pill.min + Vec2::new(6.0, 1.0), badge, colors::PRIMARY_FIXED);
            } else {
                painter.galley(
                    Pos2::new(x, rect.center().y - badge_size.y / 2.0),
                    badge,
                    badge_color,
                );
            }
        }
        response
    }
}
