//! The central stage: the live sphere/cube preview, rotated by dragging.

use super::session::Session;
use crate::components::{segmented::Segmented, text};
use crate::config::theme::{colors, typography};
use crate::features::materials::logic::preview::PreviewShape;
use crate::features::materials::models::MaterialState;
use crate::services::localization::tr;
use eframe::egui::{pos2, vec2, Align2, CursorIcon, Image, Rect, Sense, Ui, UiBuilder};

/// Draws the stage into `area`.
pub fn show(ui: &mut Ui, area: Rect, state: &mut MaterialState, session: &Session) {
    ui.painter().rect_filled(area, 0.0, colors::SURFACE_LOWEST);
    let mut bar = ui.new_child(UiBuilder::new().max_rect(Rect::from_min_size(
        area.min + vec2(12.0, 12.0),
        vec2(180.0, 32.0),
    )));
    let labels = [tr("material.shape_sphere"), tr("material.shape_cube")];
    let mut index = usize::from(state.shape == PreviewShape::Cube);
    if Segmented::text(&labels)
        .height(28.0)
        .show(&mut bar, &mut index)
    {
        state.shape = if index == 1 {
            PreviewShape::Cube
        } else {
            PreviewShape::Sphere
        };
    }
    let Some(texture) = session.preview() else {
        text::paint_text(
            ui.painter(),
            area.center(),
            Align2::CENTER_CENTER,
            tr("material.none_selected"),
            typography::body_md(),
            colors::OUTLINE,
        );
        return;
    };
    let side = (area.width().min(area.height()) - 96.0).clamp(96.0, 560.0);
    let rect = Rect::from_center_size(area.center(), vec2(side, side));
    let response = ui
        .allocate_rect(rect, Sense::drag())
        .on_hover_cursor(CursorIcon::Grab);
    if response.dragged() {
        let delta = response.drag_delta();
        state.yaw += delta.x * 0.012;
        state.pitch = (state.pitch + delta.y * 0.012).clamp(-1.4, 1.4);
    }
    Image::new((texture.id(), rect.size())).paint_at(ui, rect);
    if let Some(material) = state.current() {
        text::paint_text(
            ui.painter(),
            pos2(area.center().x, rect.bottom() + 24.0),
            Align2::CENTER_CENTER,
            &material.name,
            typography::label_lg(),
            colors::ON_SURFACE,
        );
    }
    text::paint_text(
        ui.painter(),
        pos2(area.center().x, area.bottom() - 16.0),
        Align2::CENTER_CENTER,
        tr("material.drag_hint"),
        typography::mono(10.0),
        colors::OUTLINE,
    );
}
