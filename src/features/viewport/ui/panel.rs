//! The interactive 3D view: reads input, moves the camera and queues the GPU paint callback.

use super::input::{self, ViewAction};
use super::state::{Notice, ViewportState};
use crate::components::text;
use crate::config::theme::{colors, typography};
use crate::features::viewport::services::callback::ViewportCallback;
use crate::features::viewport::services::globals::Globals;
use crate::services::localization::{tr, trf};
use eframe::egui::{Align2, Context, CursorIcon, Id, Rect, Response, Sense, Ui};
use std::time::{Duration, Instant};

/// How long a notice stays visible.
pub const NOTICE_TIME: Duration = Duration::from_secs(2);

fn say(state: &mut ViewportState, ctx: &Context, text: String) {
    state.notice = Some(Notice {
        text,
        at: Instant::now(),
    });
    ctx.request_repaint_after(NOTICE_TIME);
}

fn apply(state: &mut ViewportState, ctx: &Context, action: ViewAction, aspect: f32) {
    match action {
        ViewAction::Frame => state.frame_scene(aspect),
        ViewAction::SaveSlot(slot) => {
            let camera = *state.camera();
            if state.slots.save(slot, camera) {
                let text = trf("view3d.slot_saved", &[&(slot + 1).to_string()]);
                say(state, ctx, text);
            }
        }
        ViewAction::RecallSlot(slot) => match state.slots.recall(slot) {
            Some(camera) => {
                state.controller.go_to(camera, true);
                let text = trf("view3d.slot_recalled", &[&(slot + 1).to_string()]);
                say(state, ctx, text);
            }
            None => {
                let text = trf("view3d.slot_empty", &[&(slot + 1).to_string()]);
                say(state, ctx, text);
            }
        },
    }
}

/// Draws the 3D view over `rect` and lets the user navigate it. Returns the response of the view area.
pub fn show(ui: &mut Ui, rect: Rect, state: &mut ViewportState) -> Response {
    let response = ui.interact(rect, Id::new("viewport-3d"), Sense::click_and_drag());
    if rect.width() < 2.0 || rect.height() < 2.0 {
        return response;
    }
    if !state.gpu_available {
        let painter = ui.painter_at(rect);
        painter.rect_filled(rect, 0.0, colors::SURFACE_LOWEST);
        text::paint_text(
            &painter,
            rect.center(),
            Align2::CENTER_CENTER,
            tr("view3d.no_gpu"),
            typography::label_md(),
            colors::OUTLINE,
        );
        return response;
    }
    let ctx = ui.ctx().clone();
    let aspect = rect.width() / rect.height();
    state.set_aspect(aspect);
    if state.take_pending_frame() {
        state.frame_now(aspect);
    }
    let frame_input = input::read(&ctx, &response, rect);
    for action in &frame_input.actions {
        apply(state, &ctx, *action, aspect);
    }
    if state.controller.update(&frame_input.nav) {
        ctx.request_repaint();
    }
    if frame_input.grabbed {
        ctx.set_cursor_icon(CursorIcon::Grabbing);
    }
    let pixels = ctx.pixels_per_point();
    let size_px = [
        (rect.width() * pixels).round().max(1.0) as u32,
        (rect.height() * pixels).round().max(1.0) as u32,
    ];
    let callback = ViewportCallback {
        globals: Globals::new(state.camera(), aspect, state.scene_radius()),
        size_px,
        scene: state.upload(),
        options: state.options,
    };
    ui.painter_at(rect).add(callback.shape(rect));
    response
}
