//! Material editor screen: material list, live preview and the properties panel.

use super::ScreenContext;
use crate::features::materials::components::session::Session;
use crate::features::materials::components::{
    header_bar, material_list, preview_stage, properties_panel,
};
use crate::features::materials::logic::sync;
use crate::features::materials::models::MaterialState;
use eframe::egui::{pos2, Rect, Ui};

/// State of the material editor screen.
#[derive(Default)]
pub struct MaterialScreen {
    state: MaterialState,
    session: Session,
}

impl MaterialScreen {
    /// Draws the screen. The open project's materials are edited in place and the project is marked modified.
    pub fn show(&mut self, ui: &mut Ui, context: &mut ScreenContext) {
        sync::adopt(&mut self.state, context.project.as_deref());
        ui.spacing_mut().item_spacing.y = 0.0;
        let name = context
            .project
            .as_ref()
            .map(|p| p.document.meta.name.clone());
        header_bar::show(
            ui,
            name.as_deref(),
            self.state.library.len(),
            &mut self.state.filter,
        );
        let body = Rect::from_min_max(
            pos2(ui.max_rect().left(), ui.cursor().top()),
            ui.max_rect().max,
        );
        let list = Rect::from_min_max(
            body.min,
            pos2(body.left() + material_list::WIDTH, body.bottom()),
        );
        let panel = Rect::from_min_max(
            pos2(body.right() - properties_panel::WIDTH, body.top()),
            body.max,
        );
        let stage = Rect::from_min_max(
            pos2(list.right(), body.top()),
            pos2(panel.left(), body.bottom()),
        );
        material_list::show(ui, list, &mut self.state);
        properties_panel::show(ui, panel, &mut self.state, &mut self.session);
        if self.session.refresh(ui.ctx(), &mut self.state) {
            ui.ctx().request_repaint();
        }
        preview_stage::show(ui, stage, &mut self.state, &self.session);
        if self.session.picker.is_open() {
            ui.ctx()
                .request_repaint_after(std::time::Duration::from_millis(100));
        }
        sync::flush(&mut self.state, context.project.as_deref_mut());
    }
}
