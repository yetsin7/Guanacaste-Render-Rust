//! Export / render mode: output settings on the right and the render queue in the bottom drawer.

use super::{ModeSlots, Panel};
use crate::components::icons;
use crate::features::rendering::components::{output_panel, queue_table};
use crate::models::AppState;
use eframe::egui::Ui;

const BOTTOM_HEIGHT: f32 = 260.0;

/// Slots of export mode.
pub fn slots() -> ModeSlots {
    ModeSlots {
        side: (
            "panel.output",
            icons::IMAGE,
            output_panel::WIDTH,
            output as Panel,
        ),
        bottom: ("panel.queue", icons::QUEUE, BOTTOM_HEIGHT, queue as Panel),
        overlay: |_, _, _| {},
        plays: false,
    }
}

fn output(ui: &mut Ui, app: &mut AppState) {
    output_panel::show(ui, ui.max_rect(), &mut app.render);
}

fn queue(ui: &mut Ui, app: &mut AppState) {
    queue_table::show(ui, ui.max_rect(), &mut app.render);
}
