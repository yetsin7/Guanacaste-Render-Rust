//! The 3D view, shared by every mode: the real scene with its navigation (ASDW/QE, right button to look, wheel,
//! `F` to frame), the navigation toolbar, and the pointer tools of build mode. Nothing but the scene is ever drawn
//! as the background: with no geometry the view shows the grid and the base ground.

use crate::commands::pointer;
use crate::features::viewport::ui::panel;
use crate::models::AppState;
use eframe::egui::{Rect, Ui};

/// Draws the 3D view into `rect` and serves its pointer tools.
pub fn panel(ui: &mut Ui, rect: Rect, app: &mut AppState) {
    let over_view = ui
        .input(|input| input.pointer.hover_pos())
        .is_some_and(|pos| rect.contains(pos));
    crate::commands::camera_link::number_keys(ui.ctx(), app, over_view);
    let response = panel::show(ui, rect, &mut app.viewport);
    let dt = ui.input(|input| input.stable_dt);
    pointer::drive(app, &response, rect, dt);
}
