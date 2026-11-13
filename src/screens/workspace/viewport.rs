//! Slot of the 3D view. Today it paints the placeholder render; the viewport feature replaces `panel` with the
//! real scene and camera controls. The shell already routes the `F` key here through `requests.frame_view`.

use crate::features::viewport::components::backdrop;
use crate::models::camera::CameraPose;
use crate::models::AppState;
use eframe::egui::{Rect, Ui};

/// Paints the 3D view into `rect` and serves view requests.
pub fn panel(ui: &mut Ui, rect: Rect, app: &mut AppState) {
    if std::mem::take(&mut app.requests.frame_view) {
        // Until the real view exists, framing returns to the default pose.
        app.view = CameraPose::default();
    }
    backdrop::paint(ui, rect, &mut app.textures);
}
