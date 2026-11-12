//! Horizontal slider over a normalized 0..1 value, with a flat bar or a thumb.

use crate::config::theme::colors;
use eframe::egui::{
    pos2, vec2, Color32, CornerRadius, CursorIcon, Rect, Response, Sense, Ui, Vec2,
};

/// Shape of the draggable handle.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Thumb {
    /// No handle: only the filled track.
    None,
    /// A rectangular handle of the given size (design-system default is 8x14).
    Bar(Vec2),
    /// A round handle of the given radius.
    Circle(f32),
}

/// A normalized slider.
pub struct Slider {
    track_height: f32,
    rounded: bool,
    thumb: Thumb,
    fill: Color32,
    track: Color32,
}

impl Slider {
    /// A 2 px rectangular-track slider with an 8x14 bar thumb, as in the design system.
    pub fn precision() -> Self {
        Self {
            track_height: 2.0,
            rounded: false,
            thumb: Thumb::Bar(vec2(8.0, 14.0)),
            fill: colors::SECONDARY_CONTAINER,
            track: colors::SURFACE_HIGHEST,
        }
    }

    /// A thick, fully rounded bar without handle.
    pub fn bar(height: f32) -> Self {
        Self {
            track_height: height,
            rounded: true,
            thumb: Thumb::None,
            fill: colors::PRIMARY,
            track: colors::SURFACE_HIGHEST,
        }
    }

    /// Sets the handle shape.
    pub fn thumb(mut self, thumb: Thumb) -> Self {
        self.thumb = thumb;
        self
    }

    /// Sets the track thickness and whether its ends are rounded.
    pub fn track(mut self, height: f32, rounded: bool) -> Self {
        self.track_height = height;
        self.rounded = rounded;
        self
    }

    /// Sets the filled-part and empty-track colors.
    pub fn colors(mut self, fill: Color32, track: Color32) -> Self {
        self.fill = fill;
        self.track = track;
        self
    }

    /// Draws the slider across the available width and edits `value` (0..=1); returns the response.
    pub fn show(self, ui: &mut Ui, value: &mut f32) -> Response {
        let handle_height = match self.thumb {
            Thumb::None => 0.0,
            Thumb::Bar(size) => size.y,
            Thumb::Circle(radius) => radius * 2.0,
        };
        let height = handle_height.max(self.track_height).max(8.0);
        let (rect, mut response) =
            ui.allocate_exact_size(vec2(ui.available_width(), height), Sense::click_and_drag());
        let response_cursor = response.clone().on_hover_cursor(CursorIcon::PointingHand);
        if let Some(pointer) = response_cursor.interact_pointer_pos() {
            let new = value_at(rect, pointer.x);
            if (new - *value).abs() > f32::EPSILON {
                *value = new;
                response.mark_changed();
            }
        }
        let painter = ui.painter();
        let radius = if self.rounded {
            CornerRadius::same(255)
        } else {
            CornerRadius::ZERO
        };
        let track = Rect::from_center_size(rect.center(), vec2(rect.width(), self.track_height));
        painter.rect_filled(track, radius, self.track);
        let x = rect.left() + rect.width() * value.clamp(0.0, 1.0);
        painter.rect_filled(
            Rect::from_min_max(track.min, pos2(x, track.bottom())),
            radius,
            self.fill,
        );
        match self.thumb {
            Thumb::None => {}
            Thumb::Bar(size) => {
                let handle = Rect::from_center_size(pos2(x, rect.center().y), size);
                painter.rect_filled(handle, CornerRadius::same(2), colors::ON_SURFACE);
                super::paint::hairline(painter, handle, 2, colors::OUTLINE_VARIANT);
            }
            Thumb::Circle(radius) => {
                painter.circle_filled(pos2(x, rect.center().y), radius, colors::PRIMARY_CONTAINER);
            }
        }
        response
    }
}

/// Normalized value for a pointer at horizontal position `x` over a slider occupying `rect`.
pub fn value_at(rect: Rect, x: f32) -> f32 {
    ((x - rect.left()) / rect.width().max(1.0)).clamp(0.0, 1.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pointer_maps_to_the_unit_range() {
        let rect = Rect::from_min_size(pos2(10.0, 0.0), vec2(100.0, 10.0));
        assert_eq!(value_at(rect, 10.0), 0.0);
        assert_eq!(value_at(rect, 60.0), 0.5);
        assert_eq!(value_at(rect, 500.0), 1.0);
        assert_eq!(value_at(rect, -5.0), 0.0);
    }
}
