//! Material editor screen: 3D view with overlays and the properties panel.

use super::ScreenContext;
use crate::features::materials::components::{header_bar, properties_panel, viewport_overlays};
use crate::features::materials::models::MaterialState;
use crate::features::viewport::components::backdrop;
use eframe::egui::{pos2, Rect, Ui};

/// State of the material editor screen.
#[derive(Default)]
pub struct MaterialScreen {
    state: MaterialState,
}

impl MaterialScreen {
    /// Draws the screen.
    pub fn show(&mut self, ui: &mut Ui, context: &mut ScreenContext) {
        ui.spacing_mut().item_spacing.y = 0.0;
        header_bar::show(ui, &mut self.state.filter);
        let body = Rect::from_min_max(
            pos2(ui.max_rect().left(), ui.cursor().top()),
            ui.max_rect().max,
        );
        let panel = Rect::from_min_max(
            pos2(body.right() - properties_panel::WIDTH, body.top()),
            body.max,
        );
        let view = Rect::from_min_max(body.min, pos2(panel.left(), body.bottom()));
        backdrop::paint(ui, view, context.textures);
        viewport_overlays::show(ui, view, &mut self.state);
        properties_panel::show(ui, panel, &mut self.state);
    }
}
