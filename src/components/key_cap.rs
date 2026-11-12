//! Keyboard key caps: the small boxed labels used for shortcuts.

use super::paint;
use crate::config::theme::{colors, typography};
use eframe::egui::{Color32, CornerRadius, FontId, Response, Sense, Ui, Vec2};

/// A boxed key label.
pub struct KeyCap<'a> {
    label: &'a str,
    font: FontId,
    fill: Color32,
    ink: Color32,
    border: Option<Color32>,
    padding: Vec2,
}

impl<'a> KeyCap<'a> {
    /// A key cap with the default 11 px label.
    pub fn new(label: &'a str) -> Self {
        Self {
            label,
            font: typography::label_sm(),
            fill: colors::SURFACE_HIGHEST,
            ink: colors::ON_SURFACE_VARIANT,
            border: None,
            padding: Vec2::new(6.0, 2.0),
        }
    }

    /// The 10 px variant used inside inputs.
    pub fn small(mut self) -> Self {
        self.font = typography::mono(10.0);
        self
    }

    /// Sets fill and text colors.
    pub fn colors(mut self, fill: Color32, ink: Color32) -> Self {
        self.fill = fill;
        self.ink = ink;
        self
    }

    /// Adds a hairline border.
    pub fn border(mut self, border: Color32) -> Self {
        self.border = Some(border);
        self
    }

    /// Sets the inner padding.
    pub fn padding(mut self, x: f32, y: f32) -> Self {
        self.padding = Vec2::new(x, y);
        self
    }

    /// Sets the label font.
    pub fn font(mut self, font: FontId) -> Self {
        self.font = font;
        self
    }

    /// Paints the key cap.
    pub fn show(self, ui: &mut Ui) -> Response {
        let galley = ui
            .painter()
            .layout_no_wrap(self.label.to_owned(), self.font, self.ink);
        let size = galley.size() + self.padding * 2.0;
        let (rect, response) = ui.allocate_exact_size(size, Sense::hover());
        let painter = ui.painter();
        painter.rect_filled(rect, CornerRadius::same(2), self.fill);
        if let Some(border) = self.border {
            paint::hairline(painter, rect, 2, border);
        }
        painter.galley(rect.min + self.padding, galley, self.ink);
        response
    }
}
