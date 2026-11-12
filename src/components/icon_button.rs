//! Square icon-only buttons, optionally with a notification dot.

use super::{icons, text};
use crate::config::theme::colors;
use crate::models::dialog::Dialog;
use crate::services::localization::tr;
use eframe::egui::{Color32, CornerRadius, CursorIcon, Pos2, Response, Sense, Ui, Vec2};

/// A square button showing one icon.
pub struct IconButton {
    glyph: char,
    icon_size: f32,
    side: f32,
    fill: Option<Color32>,
    active: bool,
    badge: bool,
    tooltip: Option<&'static str>,
    opens: Option<Dialog>,
}

impl IconButton {
    /// A 32 px button with an 18 px icon and the `surface-container` fill.
    pub fn new(glyph: char) -> Self {
        Self {
            glyph,
            icon_size: 18.0,
            side: 32.0,
            fill: Some(colors::SURFACE_CONTAINER),
            active: false,
            badge: false,
            tooltip: None,
            opens: None,
        }
    }

    /// A notification bell that opens the notifications panel.
    pub fn bell() -> Self {
        let mut button = Self::new(icons::NOTIFICATIONS);
        button.opens = Some(Dialog::Notifications);
        button.tooltip = Some(tr("notices.title"));
        button.badge = true;
        button
    }

    /// Makes a click request the given dialog.
    pub fn opens(mut self, dialog: Dialog) -> Self {
        self.opens = Some(dialog);
        self
    }

    /// Sets the icon size and the button side.
    pub fn sizes(mut self, icon_size: f32, side: f32) -> Self {
        self.icon_size = icon_size;
        self.side = side;
        self
    }

    /// Sets the resting fill; `None` leaves the button transparent.
    pub fn fill(mut self, fill: Option<Color32>) -> Self {
        self.fill = fill;
        self
    }

    /// Marks the button as selected (primary icon on a raised fill).
    pub fn active(mut self, active: bool) -> Self {
        self.active = active;
        self
    }

    /// Shows the notification dot in the top-right corner.
    pub fn badge(mut self, badge: bool) -> Self {
        self.badge = badge;
        self
    }

    /// Sets the tooltip text (a localization key already resolved by the caller).
    pub fn tooltip(mut self, tooltip: &'static str) -> Self {
        self.tooltip = Some(tooltip);
        self
    }

    /// Paints the button and returns its response.
    pub fn show(self, ui: &mut Ui) -> Response {
        let (rect, response) = ui.allocate_exact_size(Vec2::splat(self.side), Sense::click());
        let mut response = response.on_hover_cursor(CursorIcon::PointingHand);
        if let Some(tooltip) = self.tooltip {
            response = response.on_hover_text(tooltip);
        }
        let hovered = response.hovered();
        let painter = ui.painter();
        let fill = match (self.active, hovered, self.fill) {
            (true, _, _) => Some(colors::SURFACE_HIGH),
            (false, true, _) => Some(colors::SURFACE_HIGH),
            (false, false, fill) => fill,
        };
        if let Some(fill) = fill {
            painter.rect_filled(rect, CornerRadius::same(2), fill);
        }
        let ink = if self.active {
            colors::PRIMARY
        } else if hovered {
            colors::ON_SURFACE
        } else {
            colors::ON_SURFACE_VARIANT
        };
        text::paint_icon(painter, rect.center(), self.glyph, self.icon_size, ink);
        if self.badge {
            let center = Pos2::new(rect.right() - 7.0, rect.top() + 7.0);
            painter.circle_filled(center, 5.0, colors::SURFACE_LOW);
            painter.circle_filled(center, 4.0, colors::PRIMARY);
        }
        if response.clicked() {
            if let Some(dialog) = self.opens {
                crate::commands::request_dialog(ui.ctx(), dialog);
            }
        }
        response
    }
}
