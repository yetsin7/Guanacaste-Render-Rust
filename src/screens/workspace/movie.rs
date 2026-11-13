//! Movie mode: the camera inspector on the right and the clip timeline in the bottom drawer. The studio's live
//! camera is the 3D view's camera: playing or scrubbing flies the view along the animation, and recording a key
//! stores where the view is. The animation is kept in the project document.

use super::{ModeSlots, Panel, Slot};
use crate::commands::camera_link;
use crate::components::icons;
use crate::config::theme::colors;
use crate::features::movie::components::{
    camera_inspector, camera_path, clip_card, timeline_rows, timeline_toolbar,
};
use crate::features::movie::logic;
use crate::features::movie::models::Selection;
use crate::models::AppState;
use eframe::egui::{pos2, vec2, Key, Rect, Ui, UiBuilder};

const BOTTOM_HEIGHT: f32 = 292.0;
const TOOLBAR: f32 = 40.0;

/// Slots of movie mode.
pub fn slots() -> ModeSlots {
    ModeSlots {
        side: Slot {
            title: "panel.camera",
            glyph: icons::VIDEOCAM,
            extent: |_| camera_inspector::WIDTH + 24.0,
            body: inspector as Panel,
        },
        bottom: Slot {
            title: "panel.timeline",
            glyph: icons::TIMELINE,
            extent: |_| BOTTOM_HEIGHT,
            body: timeline as Panel,
        },
        overlay,
        rail: None,
        side_left: false,
        plays: true,
    }
}

/// Per-frame work of the mode: keys, playback and the link between the studio camera and the 3D view.
pub fn follow(ui: &Ui, app: &mut AppState) {
    keyboard(ui, app);
    let dt = f64::from(ui.input(|input| input.stable_dt).min(0.1));
    logic::advance(&mut app.movie, &app.animation, dt);
    camera_link::follow_movie(app);
    app.movie_export.update();
    if app.movie.playing || app.movie_export.is_busy() {
        ui.ctx().request_repaint();
    }
    app.movie_export.show(ui.ctx(), &app.animation);
}

fn keyboard(ui: &Ui, app: &mut AppState) {
    if ui.ctx().egui_wants_keyboard_input() {
        return;
    }
    if ui.input(|input| input.key_pressed(Key::Space)) {
        logic::toggle_play(&mut app.movie, &app.animation);
    }
    if ui.input(|input| input.key_pressed(Key::Delete)) {
        match app.movie.selection {
            Selection::Camera(index) => {
                app.animation.remove_camera_key(index);
            }
            Selection::Value { channel, key } => {
                app.animation.remove_value_key(channel, key);
            }
            Selection::None => {}
        }
        app.movie.selection = Selection::None;
    }
}

fn overlay(ui: &mut Ui, rect: Rect, app: &mut AppState) {
    camera_path::show(ui, rect, &mut app.animation, &mut app.movie);
    clip_card::show(ui, rect.min + vec2(12.0, 12.0), &mut app.animation);
}

fn inspector(ui: &mut Ui, app: &mut AppState) {
    let area = ui.max_rect();
    camera_inspector::show(
        ui,
        area.min + vec2(12.0, 0.0),
        area.height(),
        &mut app.animation,
        &mut app.movie,
    );
}

fn timeline(ui: &mut Ui, app: &mut AppState) {
    let area = ui.max_rect();
    ui.painter().rect_filled(area, 0.0, colors::SURFACE_LOW);
    let mut bar = ui.new_child(
        UiBuilder::new().max_rect(Rect::from_min_size(area.min, vec2(area.width(), TOOLBAR))),
    );
    bar.spacing_mut().item_spacing.y = 0.0;
    let outcome = timeline_toolbar::show(&mut bar, &mut app.animation, &mut app.movie);
    if outcome.export {
        app.movie_export.show_dialog();
    }
    let rows = Rect::from_min_max(pos2(area.left(), area.top() + TOOLBAR), area.max);
    timeline_rows::show(ui, rows, &mut app.animation, &mut app.movie);
}
