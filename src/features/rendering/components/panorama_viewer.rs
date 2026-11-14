//! The simple panorama viewer: a window with the picture seen from the middle of the sphere. Dragging looks
//! around, the wheel zooms, and the buttons recenter or open the folder of the file.

use crate::components::button::{Button, ButtonStyle};
use crate::features::rendering::logic::panorama_view::reproject;
use crate::features::rendering::models::{PanoramaViewer, ViewKey};
use crate::features::rendering::services::open_folder;
use crate::services::localization::tr;
use eframe::egui::{self, vec2, Context, Sense, TextureOptions, Window};

/// Resolution the view is resampled at; it is stretched to the window.
const VIEW: [usize; 2] = [800, 450];

/// Draws the viewer window while it is open.
pub fn show(ctx: &Context, viewer: &mut PanoramaViewer) {
    if !viewer.open || viewer.source.is_none() {
        return;
    }
    let mut open = true;
    let mut close = false;
    Window::new(tr("pano.viewer_title"))
        .id(egui::Id::new("panorama-viewer"))
        .open(&mut open)
        .collapsible(false)
        .default_size([840.0, 520.0])
        .show(ctx, |ui| {
            let size = vec2(
                ui.available_width().max(320.0),
                ui.available_width().max(320.0) * 9.0 / 16.0,
            );
            let (rect, response) = ui.allocate_exact_size(size, Sense::drag());
            if response.dragged() {
                let delta = response.drag_delta();
                let per_point = viewer.fov / rect.width().max(1.0);
                viewer.yaw = (viewer.yaw - delta.x * per_point + 180.0).rem_euclid(360.0) - 180.0;
                viewer.pitch = (viewer.pitch + delta.y * per_point).clamp(-89.0, 89.0);
            }
            if response.hovered() {
                let wheel = ui.input(|input| input.smooth_scroll_delta.y);
                viewer.fov = (viewer.fov - wheel * 0.1).clamp(20.0, 120.0);
            }
            let key = ViewKey {
                yaw: viewer.yaw,
                pitch: viewer.pitch,
                fov: viewer.fov,
            };
            if viewer.key != Some(key) || viewer.texture.is_none() {
                if let Some(source) = &viewer.source {
                    let image = reproject(source, key.yaw, key.pitch, key.fov, VIEW);
                    viewer.texture =
                        Some(ctx.load_texture("panorama-view", image, TextureOptions::LINEAR));
                    viewer.key = Some(key);
                }
            }
            if let Some(texture) = &viewer.texture {
                ui.painter().image(
                    texture.id(),
                    rect,
                    egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)),
                    egui::Color32::WHITE,
                );
            }
            ui.add_space(6.0);
            ui.horizontal(|ui| {
                if Button::new(tr("pano.recenter"), ButtonStyle::Container)
                    .show(ui)
                    .clicked()
                {
                    viewer.yaw = 0.0;
                    viewer.pitch = 0.0;
                    viewer.fov = 90.0;
                }
                if let Some(folder) = viewer.path.as_deref().and_then(std::path::Path::parent) {
                    if Button::new(tr("pano.open_folder"), ButtonStyle::Container)
                        .show(ui)
                        .clicked()
                    {
                        open_folder(folder);
                    }
                }
                if Button::new(tr("done.close"), ButtonStyle::Quiet)
                    .show(ui)
                    .clicked()
                {
                    close = true;
                }
            });
            ui.label(tr("pano.viewer_hint"));
        });
    viewer.open = open && !close;
}
