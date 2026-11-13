//! The sample-image preview: shows what the camera framing and the filter chain do to a test picture.

use super::{controls, focus_panel};
use crate::components::tag::Tag;
use crate::config::theme::{colors, typography};
use crate::features::camera::logic::{exposure, preview};
use crate::features::camera::models::exposure::PREVIEW_SCENE_EV100;
use crate::features::camera::models::state::{CameraStudioState, PreviewKey};
use crate::services::localization::{tr, trf};
use eframe::egui::{self, pos2, vec2, Color32, ColorImage, Rect, TextureOptions, Ui, UiBuilder};

/// Re-renders the preview texture when its inputs changed since the last frame.
fn refresh(ctx: &egui::Context, state: &mut CameraStudioState) {
    let camera = state.camera;
    let key = PreviewKey {
        aspect: camera.aspect(),
        stops: exposure::exposure_stops(
            PREVIEW_SCENE_EV100,
            camera.ev100(),
            camera.exposure.compensation_ev,
        ),
        stack: state.stack.clone(),
    };
    let cache = &mut state.preview;
    if cache.texture.is_some() && cache.key.as_ref() == Some(&key) {
        return;
    }
    let image = preview::render_preview(&cache.source, &camera, &state.stack);
    let color =
        ColorImage::from_rgba_unmultiplied([image.width(), image.height()], &image.to_srgb8());
    match &mut cache.texture {
        Some(texture) => texture.set(color, TextureOptions::LINEAR),
        None => {
            cache.texture = Some(ctx.load_texture("camera-preview", color, TextureOptions::LINEAR))
        }
    }
    cache.key = Some(key);
}

/// Paints the preview centered inside `area` with its readout.
pub fn show(ui: &mut Ui, area: Rect, state: &mut CameraStudioState) {
    refresh(ui.ctx(), state);
    ui.painter().rect_filled(area, 0.0, colors::SURFACE_LOWEST);
    if let Some(texture) = &state.preview.texture {
        let frame = area.shrink2(vec2(24.0, 56.0));
        let aspect = state.camera.aspect();
        let size = if frame.width() / frame.height() > aspect {
            vec2(frame.height() * aspect, frame.height())
        } else {
            vec2(frame.width(), frame.width() / aspect)
        };
        let rect = Rect::from_center_size(frame.center(), size);
        let full = Rect::from_min_max(pos2(0.0, 0.0), pos2(1.0, 1.0));
        ui.painter().image(texture.id(), rect, full, Color32::WHITE);
        ui.painter().rect_stroke(
            rect,
            0.0,
            egui::Stroke::new(1.0, colors::OUTLINE_VARIANT),
            egui::StrokeKind::Outside,
        );
    }
    let mut badge = ui.new_child(UiBuilder::new().max_rect(area.shrink(12.0)));
    Tag::new(tr("camera.sample_preview"), colors::WARNING)
        .font(typography::mono_medium(11.0))
        .dot(colors::WARNING)
        .gap(8.0)
        .fill(colors::with_alpha(colors::SURFACE_LOW, 230))
        .padding(12.0, 4.0)
        .show(&mut badge);
    let camera = &state.camera;
    let readout = trf(
        "camera.hud",
        &[
            &controls::mm(camera.lens.focal_length_mm),
            &controls::trim(camera.lens.aperture, 1),
            &controls::degrees(camera.horizontal_fov()),
            &focus_panel::sharp_zone(camera),
        ],
    );
    let hud = Rect::from_min_size(
        pos2(area.left() + 12.0, area.bottom() - 32.0),
        vec2(area.width() - 24.0, 24.0),
    );
    let mut hud_ui = ui.new_child(UiBuilder::new().max_rect(hud));
    crate::components::text::label(
        &mut hud_ui,
        readout,
        typography::label_md(),
        colors::ON_SURFACE,
    );
}
