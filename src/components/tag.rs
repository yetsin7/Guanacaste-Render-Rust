//! A small bordered or filled capsule: status pills, badges and chips with optional dot and icons. It is painted
//! as one allocated rectangle, so it reads left to right even inside right-to-left layouts.

use super::{paint, text};
use crate::config::theme::{colors, typography};
use eframe::egui::{Color32, CornerRadius, FontId, Pos2, Response, Sense, Ui, Vec2};

/// A chip made of colored text parts.
pub struct Tag {
    parts: Vec<(String, Color32)>,
    font: FontId,
    fill: Option<Color32>,
    border: Option<Color32>,
    radius: u8,
    padding: Vec2,
    gap: f32,
    lead_icon: Option<(char, f32, Color32)>,
    trail_icon: Option<(char, f32, Color32)>,
    dot: Option<Color32>,
    height: Option<f32>,
    sense: Sense,
}

impl Tag {
    /// A tag with one text part.
    pub fn new(text: impl Into<String>, color: Color32) -> Self {
        Self {
            parts: vec![(text.into(), color)],
            font: typography::label_sm(),
            fill: None,
            border: None,
            radius: 2,
            padding: Vec2::new(4.0, 2.0),
            gap: 4.0,
            lead_icon: None,
            trail_icon: None,
            dot: None,
            height: None,
            sense: Sense::hover(),
        }
    }

    /// Appends another text part drawn right after the previous one.
    pub fn part(mut self, text: impl Into<String>, color: Color32) -> Self {
        self.parts.push((text.into(), color));
        self
    }

    /// Sets the text font.
    pub fn font(mut self, font: FontId) -> Self {
        self.font = font;
        self
    }

    /// Sets the background fill.
    pub fn fill(mut self, fill: Color32) -> Self {
        self.fill = Some(fill);
        self
    }

    /// Sets the hairline border color.
    pub fn border(mut self, border: Color32) -> Self {
        self.border = Some(border);
        self
    }

    /// Sets the corner radius in points.
    pub fn radius(mut self, radius: u8) -> Self {
        self.radius = radius;
        self
    }

    /// Sets the inner padding.
    pub fn padding(mut self, x: f32, y: f32) -> Self {
        self.padding = Vec2::new(x, y);
        self
    }

    /// Sets the gap between dot, icons and text.
    pub fn gap(mut self, gap: f32) -> Self {
        self.gap = gap;
        self
    }

    /// Adds a leading icon.
    pub fn lead_icon(mut self, glyph: char, size: f32, color: Color32) -> Self {
        self.lead_icon = Some((glyph, size, color));
        self
    }

    /// Adds a trailing icon.
    pub fn trail_icon(mut self, glyph: char, size: f32, color: Color32) -> Self {
        self.trail_icon = Some((glyph, size, color));
        self
    }

    /// Adds a 6 px status dot before the text.
    pub fn dot(mut self, color: Color32) -> Self {
        self.dot = Some(color);
        self
    }

    /// Forces a fixed height.
    pub fn height(mut self, height: f32) -> Self {
        self.height = Some(height);
        self
    }

    /// Makes the tag respond to clicks.
    pub fn clickable(mut self) -> Self {
        self.sense = Sense::click();
        self
    }

    /// Lays out and paints the tag.
    pub fn show(self, ui: &mut Ui) -> Response {
        let galleys: Vec<_> = self
            .parts
            .iter()
            .map(|(value, color)| {
                ui.painter()
                    .layout_no_wrap(value.clone(), self.font.clone(), *color)
            })
            .collect();
        let text_width: f32 = galleys.iter().map(|galley| galley.size().x).sum();
        let text_height = galleys
            .iter()
            .map(|galley| galley.size().y)
            .fold(0.0, f32::max);
        let mut content = text_width;
        let mut items = galleys.len() as f32;
        for width in [
            self.lead_icon.map(|icon| icon.1),
            self.trail_icon.map(|icon| icon.1),
            self.dot.map(|_| 6.0),
        ]
        .into_iter()
        .flatten()
        {
            content += width;
            items += 1.0;
        }
        // Text parts sit flush against each other; gaps apply between groups only.
        let groups = items - galleys.len() as f32 + 1.0;
        content += self.gap * (groups - 1.0).max(0.0);
        let inner_height = text_height.max(
            self.lead_icon
                .map_or(0.0, |icon| icon.1)
                .max(self.trail_icon.map_or(0.0, |icon| icon.1)),
        );
        let size = Vec2::new(
            content + self.padding.x * 2.0,
            self.height.unwrap_or(inner_height + self.padding.y * 2.0),
        );
        let (rect, response) = ui.allocate_exact_size(size, self.sense);
        let painter = ui.painter();
        let radius = CornerRadius::same(self.radius);
        if let Some(fill) = self.fill {
            painter.rect_filled(rect, radius, fill);
        }
        if let Some(border) = self.border {
            paint::hairline(painter, rect, self.radius, border);
        }
        let mut x = rect.left() + self.padding.x;
        let center_y = rect.center().y;
        if let Some((glyph, icon_size, color)) = self.lead_icon {
            text::paint_icon(
                painter,
                Pos2::new(x + icon_size / 2.0, center_y),
                glyph,
                icon_size,
                color,
            );
            x += icon_size + self.gap;
        }
        if let Some(color) = self.dot {
            painter.circle_filled(Pos2::new(x + 3.0, center_y), 3.0, color);
            x += 6.0 + self.gap;
        }
        for galley in galleys {
            let width = galley.size().x;
            let pos = Pos2::new(x, center_y - galley.size().y / 2.0);
            painter.galley(pos, galley, Color32::WHITE);
            x += width;
        }
        if let Some((glyph, icon_size, color)) = self.trail_icon {
            x += self.gap;
            text::paint_icon(
                painter,
                Pos2::new(x + icon_size / 2.0, center_y),
                glyph,
                icon_size,
                color,
            );
        }
        response
    }
}

/// The telemetry pill of the title bar: a green dot, the frame rate and the muted resolution.
pub fn fps_pill(ui: &mut Ui, fps: &str, resolution: &str) -> Response {
    Tag::new(fps, colors::TERTIARY)
        .part(resolution, colors::OUTLINE)
        .dot(colors::TERTIARY)
        .fill(colors::SURFACE_LOWEST)
        .border(colors::SURFACE_HIGHEST)
        .show(ui)
}
