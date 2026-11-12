//! Text buttons of the design system: 2 px corners, mono labels, optional leading icon.

use super::text;
use crate::config::theme::{colors, typography};
use eframe::egui::{Color32, CornerRadius, CursorIcon, FontId, Pos2, Response, Sense, Ui, Vec2};

/// Visual variants of a button.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ButtonStyle {
    /// Pale cerulean fill with dark-blue text: the main call to action.
    Pale,
    /// Electric-blue fill with white text: the primary tool button.
    Solid,
    /// Raised `surface-container-high` fill.
    Raised,
    /// `surface-container` fill.
    Container,
    /// Page-colored fill, used on top of cards.
    Surface,
    /// No fill until hovered.
    Quiet,
    /// Primary-colored text with no fill, used for links.
    Link,
}

impl ButtonStyle {
    /// `(fill, text)` colors for the given interaction state.
    fn colors(self, hovered: bool) -> (Color32, Color32) {
        match (self, hovered) {
            (ButtonStyle::Pale, false) => (colors::PRIMARY, colors::ON_PRIMARY),
            (ButtonStyle::Pale, true) => (colors::PRIMARY_CONTAINER, colors::ON_PRIMARY),
            (ButtonStyle::Solid, false) => (colors::SECONDARY_CONTAINER, Color32::WHITE),
            (ButtonStyle::Solid, true) => (colors::PRIMARY_CONTAINER, Color32::WHITE),
            (ButtonStyle::Raised, false) => (colors::SURFACE_HIGH, colors::ON_SURFACE),
            (ButtonStyle::Raised, true) => (colors::SURFACE_HIGHEST, colors::ON_SURFACE),
            (ButtonStyle::Container, false) => (colors::SURFACE_CONTAINER, colors::ON_SURFACE),
            (ButtonStyle::Container, true) => (colors::SURFACE_HIGH, colors::ON_SURFACE),
            (ButtonStyle::Surface, false) => (colors::SURFACE, colors::ON_SURFACE),
            (ButtonStyle::Surface, true) => (colors::SURFACE_CONTAINER, colors::ON_SURFACE),
            (ButtonStyle::Quiet, false) => (Color32::TRANSPARENT, colors::ON_SURFACE_VARIANT),
            (ButtonStyle::Quiet, true) => (colors::SURFACE_CONTAINER, colors::ON_SURFACE),
            (ButtonStyle::Link, false) => (Color32::TRANSPARENT, colors::PRIMARY),
            (ButtonStyle::Link, true) => (Color32::TRANSPARENT, colors::ON_SURFACE),
        }
    }
}

/// A clickable text button.
pub struct Button {
    text: String,
    icon: Option<char>,
    icon_size: f32,
    style: ButtonStyle,
    font: FontId,
    padding: Vec2,
    width: Option<f32>,
    height: Option<f32>,
    fill_width: bool,
    custom: Option<(Color32, Color32)>,
    disabled: Option<String>,
}

impl Button {
    /// A button with the default look (`label-md`, 8x6 padding).
    pub fn new(text: impl Into<String>, style: ButtonStyle) -> Self {
        Self {
            text: text.into(),
            icon: None,
            icon_size: 16.0,
            style,
            font: typography::label_md(),
            padding: Vec2::new(8.0, 6.0),
            width: None,
            height: None,
            fill_width: false,
            custom: None,
            disabled: None,
        }
    }

    /// Adds a leading icon of the given size.
    pub fn icon(mut self, glyph: char, size: f32) -> Self {
        self.icon = Some(glyph);
        self.icon_size = size;
        self
    }

    /// Sets the label font.
    pub fn font(mut self, font: FontId) -> Self {
        self.font = font;
        self
    }

    /// Sets the inner padding.
    pub fn padding(mut self, x: f32, y: f32) -> Self {
        self.padding = Vec2::new(x, y);
        self
    }

    /// Forces a fixed width.
    pub fn width(mut self, width: f32) -> Self {
        self.width = Some(width);
        self
    }

    /// Forces a fixed height.
    pub fn height(mut self, height: f32) -> Self {
        self.height = Some(height);
        self
    }

    /// Stretches the button over the remaining width of the row.
    pub fn fill_width(mut self) -> Self {
        self.fill_width = true;
        self
    }

    /// Overrides the style with explicit `(fill, text)` colors; hovering brightens the fill.
    pub fn colors(mut self, fill: Color32, ink: Color32) -> Self {
        self.custom = Some((fill, ink));
        self
    }

    /// Disables the button: it is drawn dimmed, never reports clicks and explains `reason` on hover.
    pub fn disabled(mut self, reason: impl Into<String>) -> Self {
        self.disabled = Some(reason.into());
        self
    }

    /// Lays out, paints and returns the click response.
    pub fn show(self, ui: &mut Ui) -> Response {
        let color_probe = Color32::WHITE;
        let galley = ui
            .painter()
            .layout_no_wrap(self.text.clone(), self.font.clone(), color_probe);
        let gap = 4.0;
        let icon_width = self.icon.map_or(0.0, |_| self.icon_size + gap);
        let content = icon_width + galley.size().x;
        let width = if self.fill_width {
            ui.available_width()
        } else {
            self.width.unwrap_or(content + self.padding.x * 2.0)
        };
        let height = self.height.unwrap_or(
            galley
                .size()
                .y
                .max(self.icon.map_or(0.0, |_| self.icon_size))
                + self.padding.y * 2.0,
        );
        let enabled = self.disabled.is_none();
        let sense = if enabled {
            Sense::click()
        } else {
            Sense::hover()
        };
        let (rect, response) = ui.allocate_exact_size(Vec2::new(width, height), sense);
        let mut response = if enabled {
            response.on_hover_cursor(CursorIcon::PointingHand)
        } else {
            response
        };
        let hovered = enabled && response.hovered();
        let (mut fill, mut ink) = self.custom.unwrap_or_else(|| self.style.colors(hovered));
        if self.custom.is_some() && hovered {
            fill = super::paint::mix(fill, Color32::WHITE, 0.08);
        }
        if let Some(reason) = &self.disabled {
            ink = super::paint::mix(ink, fill, 0.55);
            response = response.on_hover_text(reason.as_str());
        }
        let painter = ui.painter();
        painter.rect_filled(rect, CornerRadius::same(2), fill);
        let mut x = rect.center().x - content / 2.0;
        if let Some(glyph) = self.icon {
            text::paint_icon(
                painter,
                Pos2::new(x + self.icon_size / 2.0, rect.center().y),
                glyph,
                self.icon_size,
                ink,
            );
            x += icon_width;
        }
        let galley = painter.layout_no_wrap(self.text, self.font, ink);
        painter.galley(
            Pos2::new(x, rect.center().y - galley.size().y / 2.0),
            galley,
            ink,
        );
        response
    }
}
