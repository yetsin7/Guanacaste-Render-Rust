//! Movie mode: camera inspector on the right and the clip timeline in the bottom drawer.

use super::{ModeSlots, Panel};
use crate::components::icons;
use crate::features::movie::components::timeline_toolbar::ToolbarAction;
use crate::features::movie::components::{
    camera_inspector, camera_path, timeline_rows, timeline_toolbar,
};
use crate::features::movie::logic;
use crate::models::navigation::{Mode, Screen};
use crate::models::AppState;
use eframe::egui::{pos2, vec2, Rect, Ui, UiBuilder};

const BOTTOM_HEIGHT: f32 = 292.0;
const TOOLBAR: f32 = 40.0;

/// Slots of movie mode.
pub fn slots() -> ModeSlots {
    ModeSlots {
        side: (
            "panel.camera",
            icons::VIDEOCAM,
            camera_inspector::WIDTH,
            inspector as Panel,
        ),
        bottom: (
            "panel.timeline",
            icons::TIMELINE,
            BOTTOM_HEIGHT,
            timeline as Panel,
        ),
        overlay: |ui, rect, app| camera_path::show(ui, rect, &app.movie),
        plays: true,
    }
}

/// Moves the playhead while playing (runs even with the timeline collapsed).
pub fn advance(ui: &Ui, app: &mut AppState) {
    let dt = ui.input(|input| input.stable_dt).min(0.1);
    logic::advance(&mut app.movie, dt);
    if app.movie.playing {
        ui.ctx().request_repaint();
    }
}

fn inspector(ui: &mut Ui, app: &mut AppState) {
    camera_inspector::show(ui, ui.max_rect().min, &mut app.movie);
}

fn timeline(ui: &mut Ui, app: &mut AppState) {
    let area = ui.max_rect();
    let mut bar = ui.new_child(
        UiBuilder::new().max_rect(Rect::from_min_size(area.min, vec2(area.width(), TOOLBAR))),
    );
    bar.spacing_mut().item_spacing.y = 0.0;
    if let Some(ToolbarAction::RenderClip) = timeline_toolbar::show(&mut bar, &mut app.movie) {
        app.navigate(Screen::Workspace(Mode::Export));
    }
    let rows = Rect::from_min_max(pos2(area.left(), area.top() + TOOLBAR), area.max);
    timeline_rows::show(ui, rows, &mut app.movie);
}
