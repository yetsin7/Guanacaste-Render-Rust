//! Movie studio screen: 3D view with clip card, path and camera inspector, plus the timeline.

use super::ScreenContext;
use crate::config::theme::colors;
use crate::features::movie::components::header_bar::HeaderAction;
use crate::features::movie::components::{
    camera_inspector, camera_path, clip_card, header_bar, timeline_rows, timeline_toolbar,
};
use crate::features::movie::logic;
use crate::features::movie::models::MovieState;
use crate::features::viewport::components::backdrop;
use crate::models::navigation::Screen;
use crate::services::localization::tr;
use eframe::egui::{pos2, Rect, Ui};

const TIMELINE_HEIGHT: f32 = 240.0;

/// State of the movie studio screen.
#[derive(Default)]
pub struct MovieScreen {
    state: MovieState,
}

impl MovieScreen {
    /// Draws the screen.
    pub fn show(&mut self, ui: &mut Ui, context: &mut ScreenContext) {
        ui.spacing_mut().item_spacing.y = 0.0;
        match header_bar::show(ui, &mut self.state) {
            Some(HeaderAction::Close) => *context.navigate = Some(Screen::Projects),
            Some(HeaderAction::ExportQueue) => *context.navigate = Some(Screen::Render),
            None => {}
        }
        let dt = ui.input(|input| input.stable_dt).min(0.1);
        logic::advance(&mut self.state, dt);
        if self.state.playing {
            ui.ctx().request_repaint();
        }
        let body = Rect::from_min_max(
            pos2(ui.max_rect().left(), ui.cursor().top()),
            ui.max_rect().max,
        );
        let timeline =
            Rect::from_min_max(pos2(body.left(), body.bottom() - TIMELINE_HEIGHT), body.max);
        let view = Rect::from_min_max(body.min, pos2(body.right(), timeline.top()));
        backdrop::paint(ui, view, context.textures);
        camera_path::show(ui, view, &self.state);
        self.overlays(ui, view);
        self.timeline(ui, timeline, context);
    }

    fn overlays(&mut self, ui: &mut Ui, view: Rect) {
        let inner = view.shrink(12.0);
        let mut status = ui.new_child(eframe::egui::UiBuilder::new().max_rect(inner));
        status.spacing_mut().item_spacing.y = 8.0;
        crate::components::tag::Tag::new("37.2 FPS", colors::TERTIARY)
            .part("  |  ", colors::OUTLINE_VARIANT)
            .part("3840×2160 (4K UHD)", colors::ON_SURFACE)
            .part("  |  ", colors::OUTLINE_VARIANT)
            .part("RTX DXR ACCEL", colors::PRIMARY)
            .dot(colors::TERTIARY)
            .gap(6.0)
            .fill(colors::with_alpha(colors::SURFACE_LOW, 230))
            .padding(10.0, 4.0)
            .show(&mut status);
        crate::components::tag::Tag::new(tr("movie.sun"), colors::ON_SURFACE_VARIANT)
            .lead_icon(crate::components::icons::SUN, 13.0, colors::TERTIARY)
            .gap(4.0)
            .fill(colors::with_alpha(colors::SURFACE_LOWEST, 217))
            .padding(8.0, 2.0)
            .show(&mut status);
        clip_card::show(ui, pos2(inner.left(), inner.top() + 56.0));
        camera_inspector::show(
            ui,
            pos2(inner.right() - camera_inspector::WIDTH, inner.top()),
            &mut self.state,
        );
    }

    fn timeline(&mut self, ui: &mut Ui, timeline: Rect, context: &mut ScreenContext) {
        ui.painter().rect_filled(timeline, 0.0, colors::SURFACE_LOW);
        let mut inner = ui.new_child(eframe::egui::UiBuilder::new().max_rect(timeline));
        inner.spacing_mut().item_spacing.y = 0.0;
        if let Some(timeline_toolbar::ToolbarAction::RenderClip) =
            timeline_toolbar::show(&mut inner, &mut self.state)
        {
            *context.navigate = Some(Screen::Render);
        }
        let rows = Rect::from_min_max(pos2(timeline.left(), timeline.top() + 40.0), timeline.max);
        timeline_rows::show(ui, rows, &mut self.state);
    }
}
