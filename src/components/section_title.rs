//! Section headings: the accent bar title with count badge, and the small uppercase caption.

use super::text;
use crate::config::theme::{colors, typography};
use eframe::egui::{CornerRadius, Sense, Ui, Vec2};

/// An accent bar, a `headline-md` title and an optional count badge.
pub fn with_count(ui: &mut Ui, title: &str, count: Option<&str>) {
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 8.0;
        let (bar, _) = ui.allocate_exact_size(Vec2::new(8.0, 16.0), Sense::hover());
        ui.painter()
            .rect_filled(bar, CornerRadius::same(2), colors::PRIMARY);
        text::label(ui, title, typography::headline_md(), colors::ON_SURFACE);
        if let Some(count) = count {
            super::tag::Tag::new(count, colors::PRIMARY)
                .fill(colors::SURFACE_HIGH)
                .padding(4.0, 2.0)
                .show(ui);
        }
    });
}

/// A small uppercase mono caption, e.g. a group heading.
pub fn caption(ui: &mut Ui, title: &str, color: eframe::egui::Color32) {
    text::label(ui, text::caps(title), typography::mono_medium(11.0), color);
}
