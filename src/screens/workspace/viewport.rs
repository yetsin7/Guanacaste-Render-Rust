//! The 3D view, shared by every mode: the real scene with its navigation (ASDW/QE, right button to look, wheel,
//! `F` to frame), the navigation toolbar, and the pointer tools of build mode. Nothing but the scene is ever drawn
//! as the background: with no geometry the view shows the grid and the base ground.

use crate::commands::pointer;
use crate::features::viewport::components::nav_toolbar;
use crate::features::viewport::ui::panel;
use crate::models::AppState;
use eframe::egui::{pos2, Rect, Ui};

/// Draws the 3D view into `rect` and serves its pointer tools; the navigation toolbar hangs from `toolbar_x`.
pub fn panel(ui: &mut Ui, rect: Rect, toolbar_x: f32, app: &mut AppState) {
    let response = panel::show(ui, rect, &mut app.viewport);
    let dt = ui.input(|input| input.stable_dt);
    pointer::drive(app, &response, rect, dt);
    if app.viewport.gpu_available {
        nav_toolbar::show(ui, pos2(toolbar_x, rect.top() + 10.0), &mut app.viewport);
    }
}
