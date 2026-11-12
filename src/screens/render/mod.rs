//! Render studio placeholder: the render engine does not exist yet.

use super::ScreenContext;
use crate::components::{icons, text};
use crate::config::theme::{colors, typography};
use crate::services::localization::tr;
use eframe::egui::{Align, Layout, Ui};

/// Draws an honest empty state.
pub fn show(ui: &mut Ui, _context: &mut ScreenContext) {
    ui.with_layout(Layout::top_down(Align::Center), |ui| {
        ui.add_space((ui.available_height() / 2.0 - 40.0).max(0.0));
        text::icon(ui, icons::CAMERA, 32.0, colors::OUTLINE);
        ui.add_space(8.0);
        text::label(
            ui,
            tr("render.title"),
            typography::headline_md(),
            colors::ON_SURFACE,
        );
        ui.add_space(4.0);
        text::label(
            ui,
            tr("render.empty"),
            typography::body_md(),
            colors::OUTLINE,
        );
    });
}
