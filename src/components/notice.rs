//! Inline notice banners: information, warning or error.

use super::{icons, text};
use crate::config::theme::{colors, typography};
use eframe::egui::{self, Color32, CornerRadius, Margin, Ui};

/// Severity of a notice.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Level {
    /// Neutral information.
    Info,
    /// Something to double-check.
    Warning,
    /// Something failed.
    Error,
}

impl Level {
    /// Accent color and icon of the level.
    pub fn look(self) -> (Color32, char) {
        match self {
            Level::Info => (colors::SECONDARY, icons::INFO),
            Level::Warning => (colors::WARNING, icons::WARNING),
            Level::Error => (colors::ERROR, icons::ERROR),
        }
    }
}

/// Draws a banner with a title and an explanation.
pub fn show(ui: &mut Ui, level: Level, title: &str, message: &str) {
    let (accent, glyph) = level.look();
    let frame = egui::Frame::new()
        .fill(colors::with_alpha(accent, 24))
        .corner_radius(CornerRadius::same(2))
        .inner_margin(Margin::symmetric(12, 8))
        .show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.horizontal_top(|ui| {
                ui.spacing_mut().item_spacing.x = 10.0;
                text::icon(ui, glyph, 18.0, accent);
                ui.vertical(|ui| {
                    ui.spacing_mut().item_spacing.y = 2.0;
                    text::label(ui, title, typography::label_md(), colors::ON_SURFACE);
                    text::paragraph(
                        ui,
                        message,
                        typography::body_sm(),
                        colors::ON_SURFACE_VARIANT,
                        18.0,
                    );
                });
            });
        });
    let strip = eframe::egui::Rect::from_min_size(
        frame.response.rect.min,
        eframe::egui::vec2(3.0, frame.response.rect.height()),
    );
    ui.painter()
        .rect_filled(strip, CornerRadius::same(2), accent);
}
