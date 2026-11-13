//! The camera studio body: the sample preview on the left and the camera and filter controls on the right.

use super::{filter_list, focus_panel, lens_panel, preview};
use crate::components::text;
use crate::config::theme::{colors, typography};
use crate::features::camera::models::state::CameraStudioState;
use crate::services::localization::tr;
use eframe::egui::{self, pos2, Margin, Rect, ScrollArea, Ui, UiBuilder};

/// Width of the control panel in points.
pub const PANEL_WIDTH: f32 = 320.0;

/// Draws the studio into `body` (the area under the screen header).
pub fn show(ui: &mut Ui, body: Rect, state: &mut CameraStudioState) {
    let panel = Rect::from_min_max(pos2(body.right() - PANEL_WIDTH, body.top()), body.max);
    let view = Rect::from_min_max(body.min, pos2(panel.left(), body.bottom()));
    preview::show(ui, view, state);
    ui.painter().rect_filled(panel, 0.0, colors::SURFACE_LOW);
    let mut inner = ui.new_child(UiBuilder::new().max_rect(panel));
    ScrollArea::vertical()
        .auto_shrink([false, false])
        .show(&mut inner, |ui| {
            egui::Frame::new()
                .inner_margin(Margin::same(16))
                .show(ui, |ui| {
                    ui.set_width(PANEL_WIDTH - 32.0);
                    ui.spacing_mut().item_spacing.y = 10.0;
                    text::label(
                        ui,
                        tr("camera.title"),
                        typography::headline_sm(),
                        colors::ON_SURFACE,
                    );
                    lens_panel::show_lens(ui, &mut state.camera);
                    lens_panel::show_distance(ui, &mut state.camera);
                    focus_panel::show_focus(ui, &mut state.camera);
                    focus_panel::show_exposure(ui, &mut state.camera);
                    ui.add_space(6.0);
                    filter_list::show(ui, state);
                });
        });
}
