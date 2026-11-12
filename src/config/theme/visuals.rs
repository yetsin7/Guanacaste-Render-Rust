//! Global egui visuals and spacing derived from the color tokens.

use super::{colors::*, typography};
use eframe::egui::{self, CornerRadius, Stroke, Vec2};

/// Installs fonts, colors and spacing for the whole application.
pub fn apply(context: &egui::Context) {
    typography::install_fonts(context);
    context.set_theme(egui::Theme::Dark);
    let hairline = Stroke::new(1.0, SURFACE_HIGHEST);
    let mut visuals = egui::Visuals::dark();
    visuals.panel_fill = SURFACE;
    visuals.window_fill = SURFACE_LOW;
    visuals.extreme_bg_color = SURFACE_LOWEST;
    visuals.faint_bg_color = SURFACE_CONTAINER;
    visuals.override_text_color = Some(ON_SURFACE);
    visuals.selection.bg_fill = with_alpha(SECONDARY_CONTAINER, 120);
    visuals.selection.stroke = Stroke::new(1.0, PRIMARY);
    visuals.widgets.noninteractive.bg_fill = SURFACE_LOW;
    visuals.widgets.noninteractive.bg_stroke = hairline;
    visuals.widgets.noninteractive.fg_stroke = Stroke::new(1.0, ON_SURFACE_VARIANT);
    visuals.widgets.inactive.bg_fill = SURFACE_CONTAINER;
    visuals.widgets.inactive.weak_bg_fill = SURFACE_CONTAINER;
    visuals.widgets.inactive.bg_stroke = hairline;
    visuals.widgets.hovered.bg_fill = SURFACE_HIGH;
    visuals.widgets.hovered.weak_bg_fill = SURFACE_HIGH;
    visuals.widgets.hovered.bg_stroke = Stroke::new(1.0, FLOATING_BORDER);
    visuals.widgets.active.bg_fill = SECONDARY_CONTAINER;
    visuals.widgets.active.weak_bg_fill = SECONDARY_CONTAINER;
    visuals.widgets.active.bg_stroke = Stroke::new(1.0, PRIMARY);
    visuals.window_corner_radius = CornerRadius::same(4);
    visuals.window_stroke = Stroke::new(1.0, FLOATING_BORDER);
    context.set_visuals(visuals);
    context.all_styles_mut(|style| {
        style.spacing.item_spacing = Vec2::new(8.0, 4.0);
        style.spacing.button_padding = Vec2::new(8.0, 4.0);
        style.spacing.scroll.bar_width = 6.0;
        style.spacing.scroll.floating = true;
        style.text_styles = [
            (egui::TextStyle::Body, typography::body_md()),
            (egui::TextStyle::Button, typography::body_md()),
            (egui::TextStyle::Small, typography::body_sm()),
            (egui::TextStyle::Heading, typography::headline_md()),
            (egui::TextStyle::Monospace, typography::label_sm()),
        ]
        .into();
    });
}
