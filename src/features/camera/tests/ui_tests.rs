//! The studio draws without a window and keeps its state consistent.

use crate::config::theme::typography;
use crate::features::camera::components::studio;
use crate::features::camera::models::filter::FilterKind;
use crate::features::camera::models::preset::FilterPreset;
use crate::features::camera::models::state::CameraStudioState;
use eframe::egui::{self, pos2, vec2, Rect};

fn frame(ctx: &egui::Context, state: &mut CameraStudioState) {
    let input = egui::RawInput {
        screen_rect: Some(Rect::from_min_size(pos2(0.0, 0.0), vec2(1333.0, 820.0))),
        ..Default::default()
    };
    let mut output = ctx.run_ui(input, |ctx| {
        egui::CentralPanel::default().show(ctx, |ui| {
            let body = ui.max_rect();
            studio::show(ui, body, state);
        });
    });
    output.textures_delta.clear();
}

#[test]
fn studio_draws_and_builds_the_preview_texture_once() {
    let ctx = egui::Context::default();
    typography::install_fonts(&ctx);
    let mut state = CameraStudioState::default();
    frame(&ctx, &mut state);
    assert!(state.preview.texture.is_some());
    let key = state.preview.key.clone();
    assert!(key.is_some());
    frame(&ctx, &mut state);
    assert_eq!(
        state.preview.key, key,
        "nothing changed, nothing re-rendered"
    );
    state.camera.exposure.compensation_ev = 1.5;
    frame(&ctx, &mut state);
    assert_ne!(state.preview.key, key);
}

#[test]
fn studio_draws_with_an_expanded_filter_for_every_kind() {
    let ctx = egui::Context::default();
    typography::install_fonts(&ctx);
    let mut state = CameraStudioState::default();
    state.stack.entries.clear();
    for kind in FilterKind::ALL {
        state.stack.push(kind);
    }
    for index in 0..state.stack.len() {
        state.selected = Some(index);
        frame(&ctx, &mut state);
    }
    state.stack.entries.clear();
    frame(&ctx, &mut state);
}

#[test]
fn choosing_a_preset_replaces_the_stack_and_editing_marks_it_custom() {
    let mut state = CameraStudioState::default();
    assert_eq!(state.preset, 0);
    state.choose_preset(3);
    assert_eq!(state.stack, FilterPreset::Cinematic.stack());
    assert_eq!(state.preset, 3);
    state.choose_preset(99);
    assert_eq!(state.preset, 3);
    state.mark_custom();
    assert_eq!(state.preset, CameraStudioState::CUSTOM);
}
