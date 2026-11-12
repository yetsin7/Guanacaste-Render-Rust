//! Build screen: the 3D view with the guided step, object properties, tool dock and shortcut hints.

use super::ScreenContext;
use crate::features::build::components::{
    object_properties, shortcuts_hint, step_card, tool_dock, top_bar,
};
use crate::features::build::logic;
use crate::features::build::models::{BuildGuide, BuildState};
use crate::features::viewport::components::{backdrop, reticle};
use crate::models::navigation::Screen;
use crate::services::localization::tr;
use crate::services::sample_data::build as sample;
use eframe::egui::{pos2, Ui};

/// State of the build screen.
pub struct BuildScreen {
    state: BuildState,
    guide: BuildGuide,
}

impl Default for BuildScreen {
    fn default() -> Self {
        Self {
            state: BuildState::default(),
            guide: sample::guide(),
        }
    }
}

impl BuildScreen {
    /// Draws the screen over the whole central area.
    pub fn show(&mut self, ui: &mut Ui, context: &mut ScreenContext) {
        let rect = ui.max_rect();
        backdrop::paint(ui, rect, context.textures);
        let center = pos2(
            rect.left() + rect.width() * 0.42,
            rect.top() + rect.height() * 0.48,
        );
        reticle::show(ui, center, self.guide.asset, tr("build.snap"));

        let inner = rect.shrink(12.0);
        if let Some(top_bar::TopBarAction::Close) = top_bar::show(ui, inner, tr("build.scene")) {
            *context.navigate = Some(Screen::Projects);
        }
        let below_top = pos2(inner.left(), inner.top() + 44.0);
        if step_card::show(ui, below_top, &self.guide, &self.state, context.textures) {
            logic::place_next(&mut self.state, &self.guide);
        }
        let panel_origin = pos2(inner.right() - object_properties::WIDTH, below_top.y);
        let panel_height = (rect.height() - 104.0).max(240.0);
        object_properties::show(
            ui,
            panel_origin,
            panel_height,
            &mut self.state,
            context.textures,
        );

        let dock_corner = pos2(inner.left(), inner.bottom());
        if tool_dock::show(ui, dock_corner, inner.width() - 400.0, &mut self.state) {
            *context.navigate = Some(Screen::MaterialEditor);
        }
        shortcuts_hint::show(ui, inner.right_bottom());
    }
}
