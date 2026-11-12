//! The 3D view backdrop: a placeholder render covering the area, with the prototype's darkening gradient.

use crate::components::paint;
use crate::config::theme::colors;
use crate::services::textures::{Placeholder, Textures};
use eframe::egui::{self, vec2, Color32, Rect, Ui};

/// Paints the placeholder render over `rect` and the gradient scrims that keep overlays legible.
pub fn paint(ui: &Ui, rect: Rect, textures: &mut Textures) {
    let texture = textures.get(ui.ctx(), Placeholder::VillaRender).clone();
    let painter = ui.painter().with_clip_rect(rect);
    painter.rect_filled(rect, 0.0, colors::SURFACE_LOWEST);
    paint::image_cover(&painter, rect, &texture, vec2(0.5, 0.55), Color32::WHITE);
    let half = rect.height() / 2.0;
    paint::vertical_gradient(
        &painter,
        Rect::from_min_size(rect.min, vec2(rect.width(), half)),
        colors::with_alpha(colors::SURFACE_LOWEST, 102),
        egui::Color32::TRANSPARENT,
    );
    paint::vertical_gradient(
        &painter,
        Rect::from_min_max(egui::pos2(rect.left(), rect.top() + half), rect.max),
        egui::Color32::TRANSPARENT,
        colors::with_alpha(colors::SURFACE_LOWEST, 204),
    );
    painter.rect_filled(rect, 0.0, colors::with_alpha(colors::SURFACE, 26));
}
