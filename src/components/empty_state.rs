//! An empty-state block: icon, title, explanation and an optional action.

use super::{
    button::{Button, ButtonStyle},
    text,
};
use crate::config::theme::{colors, typography};
use eframe::egui::{Align, Layout, Ui};

/// Draws the block centered in the available width; returns true when the action button was clicked.
pub fn show(ui: &mut Ui, glyph: char, title: &str, message: &str, action: Option<&str>) -> bool {
    let mut clicked = false;
    ui.vertical_centered(|ui| {
        ui.spacing_mut().item_spacing.y = 6.0;
        ui.add_space(24.0);
        text::icon(ui, glyph, 32.0, colors::OUTLINE);
        text::label(ui, title, typography::headline_sm(), colors::ON_SURFACE);
        ui.allocate_ui_with_layout(
            eframe::egui::vec2(420.0, 0.0),
            Layout::top_down(Align::Center),
            |ui| {
                text::paragraph(ui, message, typography::body_sm(), colors::OUTLINE, 18.0);
            },
        );
        if let Some(label) = action {
            ui.add_space(6.0);
            clicked = Button::new(label, ButtonStyle::Container)
                .font(typography::label_md())
                .padding(12.0, 6.0)
                .show(ui)
                .clicked();
        }
        ui.add_space(24.0);
    });
    clicked
}
