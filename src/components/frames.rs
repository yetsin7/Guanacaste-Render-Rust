//! Frames for the surface tiers of the design system.

use crate::config::theme::colors;
use eframe::egui::{Color32, CornerRadius, Frame, Margin, Shadow, Stroke};

/// A flat card on the given fill with uniform padding and 2 px corners.
pub fn card(fill: Color32, padding: i8) -> Frame {
    Frame::new()
        .fill(fill)
        .corner_radius(CornerRadius::same(2))
        .inner_margin(Margin::same(padding))
}

/// A translucent panel floating over the 3D view (`surface-container-low` at 95 %).
pub fn overlay(padding: i8) -> Frame {
    card(colors::with_alpha(colors::SURFACE_LOW, 242), padding)
}

/// A floating panel with a bright hairline and the ambient shadow of the design system.
pub fn floating(padding: i8) -> Frame {
    Frame::new()
        .fill(colors::with_alpha(colors::SURFACE_LOW, 242))
        .stroke(Stroke::new(1.0, colors::FLOATING_BORDER))
        .corner_radius(CornerRadius::same(4))
        .inner_margin(Margin::same(padding))
        .shadow(Shadow {
            offset: [0, 8],
            blur: 24,
            spread: 0,
            color: Color32::from_black_alpha(165),
        })
}
