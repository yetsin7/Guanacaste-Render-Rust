//! Render studio screen: output settings with a sample preview, and the render queue.

use super::ScreenContext;
use crate::components::{tag::Tag, text};
use crate::config::theme::{colors, typography};
use crate::features::rendering::components::{output_panel, queue_table, studio_header};
use crate::features::rendering::models::RenderStudioState;
use crate::features::viewport::components::backdrop;
use crate::services::localization::tr;
use crate::services::sample_data::rendering as sample;
use eframe::egui::{pos2, vec2, Rect, Ui, UiBuilder};

/// State of the render studio screen.
pub struct RenderScreen {
    state: RenderStudioState,
}

impl Default for RenderScreen {
    fn default() -> Self {
        Self {
            state: RenderStudioState {
                tab: 0,
                jobs: sample::jobs(),
                resolution: 2,
                samples: 0.25,
                denoise: true,
                output: "C:/Proyectos/Guanacaste/Renders".into(),
            },
        }
    }
}

impl RenderScreen {
    /// Draws the screen.
    pub fn show(&mut self, ui: &mut Ui, context: &mut ScreenContext) {
        ui.spacing_mut().item_spacing.y = 0.0;
        studio_header::show(ui, &mut self.state);
        let body = Rect::from_min_max(
            pos2(ui.max_rect().left(), ui.cursor().top()),
            ui.max_rect().max,
        );
        if self.state.tab == 1 {
            queue_table::show(ui, body, &mut self.state);
            return;
        }
        let panel = Rect::from_min_max(
            pos2(body.right() - output_panel::WIDTH, body.top()),
            body.max,
        );
        let view = Rect::from_min_max(body.min, pos2(panel.left(), body.bottom()));
        backdrop::paint(ui, view, context.textures);
        let mut badge = ui.new_child(UiBuilder::new().max_rect(view.shrink(12.0)));
        Tag::new(tr("render.sample_preview"), colors::WARNING)
            .font(typography::mono_medium(11.0))
            .dot(colors::WARNING)
            .gap(8.0)
            .fill(colors::with_alpha(colors::SURFACE_LOW, 230))
            .padding(12.0, 4.0)
            .show(&mut badge);
        let hud = Rect::from_min_size(
            pos2(view.left() + 12.0, view.bottom() - 36.0),
            vec2(360.0, 24.0),
        );
        let mut hud_ui = ui.new_child(UiBuilder::new().max_rect(hud));
        let resolution = crate::features::rendering::logic::resolution_label(self.state.resolution);
        text::label(
            &mut hud_ui,
            resolution,
            typography::label_md(),
            colors::ON_SURFACE,
        );
        output_panel::show(ui, panel, &mut self.state);
    }
}
