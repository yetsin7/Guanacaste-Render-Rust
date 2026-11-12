//! Segmented controls: a row of mutually exclusive options, as text or icons.

use super::text;
use crate::config::theme::{colors, typography};
use eframe::egui::{Align2, Color32, CornerRadius, CursorIcon, Rect, Sense, Ui, Vec2};

/// What each segment shows.
pub enum Segments<'a> {
    /// Text labels.
    Text(&'a [&'a str]),
    /// Icon glyphs.
    Icons(&'a [char]),
}

/// A segmented control.
pub struct Segmented<'a> {
    segments: Segments<'a>,
    height: f32,
    width: Option<f32>,
    container: Color32,
    padding: f32,
    gap: f32,
    selected_fill: Color32,
    selected_ink: Color32,
    idle_ink: Color32,
}

impl<'a> Segmented<'a> {
    /// A control of text options filling the available width: recessed container, blue selected segment.
    pub fn text(options: &'a [&'a str]) -> Self {
        Self {
            segments: Segments::Text(options),
            height: 28.0,
            width: None,
            container: colors::SURFACE_LOWEST,
            padding: 4.0,
            gap: 4.0,
            selected_fill: colors::SECONDARY_CONTAINER,
            selected_ink: colors::ON_SECONDARY_CONTAINER,
            idle_ink: colors::OUTLINE,
        }
    }

    /// A compact control of icon options (24 px per segment) with a raised container.
    pub fn icons(glyphs: &'a [char]) -> Self {
        Self {
            segments: Segments::Icons(glyphs),
            height: 28.0,
            width: Some(glyphs.len() as f32 * 24.0 + 4.0 + (glyphs.len() as f32 - 1.0) * 4.0),
            container: colors::SURFACE_HIGH,
            padding: 2.0,
            gap: 4.0,
            selected_fill: colors::SURFACE,
            selected_ink: colors::PRIMARY,
            idle_ink: colors::OUTLINE,
        }
    }

    /// Sets the control height.
    pub fn height(mut self, height: f32) -> Self {
        self.height = height;
        self
    }

    /// Sets a fixed total width.
    pub fn width(mut self, width: f32) -> Self {
        self.width = Some(width);
        self
    }

    /// Sets container fill, inner padding and the gap between segments.
    pub fn container(mut self, fill: Color32, padding: f32, gap: f32) -> Self {
        self.container = fill;
        self.padding = padding;
        self.gap = gap;
        self
    }

    /// Sets the colors of the selected segment.
    pub fn selected(mut self, fill: Color32, ink: Color32) -> Self {
        self.selected_fill = fill;
        self.selected_ink = ink;
        self
    }

    /// Draws the control; returns true when the selection changed.
    pub fn show(self, ui: &mut Ui, selected: &mut usize) -> bool {
        let count = match &self.segments {
            Segments::Text(options) => options.len(),
            Segments::Icons(glyphs) => glyphs.len(),
        };
        let width = self.width.unwrap_or_else(|| ui.available_width());
        let (outer, _) = ui.allocate_exact_size(Vec2::new(width, self.height), Sense::hover());
        ui.painter()
            .rect_filled(outer, CornerRadius::same(2), self.container);
        let inner = outer.shrink(self.padding);
        let cell_width = (inner.width() - self.gap * (count as f32 - 1.0)) / count as f32;
        let mut changed = false;
        for index in 0..count {
            let rect = Rect::from_min_size(
                inner.min + Vec2::new(index as f32 * (cell_width + self.gap), 0.0),
                Vec2::new(cell_width, inner.height()),
            );
            let id = ui.id().with(("segment", index, rect.min.x as i32));
            let response = ui
                .interact(rect, id, Sense::click())
                .on_hover_cursor(CursorIcon::PointingHand);
            let active = index == *selected;
            let painter = ui.painter();
            if active {
                painter.rect_filled(rect, CornerRadius::same(2), self.selected_fill);
            } else if response.hovered() {
                painter.rect_filled(rect, CornerRadius::same(2), colors::SURFACE_HIGH);
            }
            let ink = if active {
                self.selected_ink
            } else if response.hovered() {
                colors::ON_SURFACE
            } else {
                self.idle_ink
            };
            match &self.segments {
                Segments::Text(options) => {
                    let font = if active {
                        typography::mono_medium(11.0)
                    } else {
                        typography::label_sm()
                    };
                    text::paint_text(
                        painter,
                        rect.center(),
                        Align2::CENTER_CENTER,
                        options[index],
                        font,
                        ink,
                    );
                }
                Segments::Icons(glyphs) => {
                    text::paint_icon(painter, rect.center(), glyphs[index], 15.0, ink);
                }
            }
            if response.clicked() && !active {
                *selected = index;
                changed = true;
            }
        }
        changed
    }
}
