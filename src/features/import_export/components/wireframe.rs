//! A rotatable wireframe preview of the imported geometry (a stand-in for the 3D view until the renderer exists).

use crate::config::theme::colors;
use crate::features::import_export::models::ImportedScene;
use eframe::egui::{pos2, Color32, CursorIcon, Sense, Stroke, Ui, Vec2};

const MAX_TRIANGLES: usize = 15_000;

/// Projects `point` orthographically after rotating by `yaw` (around Z) and `pitch`.
pub fn project(point: [f32; 3], yaw: f32, pitch: f32) -> [f32; 2] {
    let (sy, cy) = yaw.sin_cos();
    let (sp, cp) = pitch.sin_cos();
    let x = point[0] * cy - point[1] * sy;
    let depth = point[0] * sy + point[1] * cy;
    [x, point[2] * cp - depth * sp]
}

/// Draws the preview of `scene` in a rectangle of `size`; dragging rotates the view.
pub fn show(ui: &mut Ui, scene: &ImportedScene, yaw: &mut f32, pitch: &mut f32, size: Vec2) {
    let (rect, response) = ui.allocate_exact_size(size, Sense::drag());
    let response = response.on_hover_cursor(CursorIcon::Grab);
    if response.dragged() {
        *yaw += response.drag_delta().x * 0.01;
        *pitch = (*pitch + response.drag_delta().y * 0.01).clamp(-1.4, 1.4);
    }
    ui.painter().rect_filled(rect, 2.0, colors::SURFACE_LOWEST);
    let Some(bounds) = scene.bounds() else { return };
    let center = [
        (bounds.min[0] + bounds.max[0]) / 2.0,
        (bounds.min[1] + bounds.max[1]) / 2.0,
        (bounds.min[2] + bounds.max[2]) / 2.0,
    ];
    let corners: Vec<[f32; 2]> = (0..8)
        .map(|i| {
            project(
                [
                    if i & 1 == 0 {
                        bounds.min[0]
                    } else {
                        bounds.max[0]
                    } - center[0],
                    if i & 2 == 0 {
                        bounds.min[1]
                    } else {
                        bounds.max[1]
                    } - center[1],
                    if i & 4 == 0 {
                        bounds.min[2]
                    } else {
                        bounds.max[2]
                    } - center[2],
                ],
                *yaw,
                *pitch,
            )
        })
        .collect();
    let extent = corners
        .iter()
        .fold(1e-6f32, |m, c| m.max(c[0].abs()).max(c[1].abs()));
    let scale = (rect.width().min(rect.height()) / 2.0 - 12.0) / extent;
    let to_screen = |p: [f32; 3]| {
        let q = project(
            [p[0] - center[0], p[1] - center[1], p[2] - center[2]],
            *yaw,
            *pitch,
        );
        pos2(
            rect.center().x + q[0] * scale,
            rect.center().y - q[1] * scale,
        )
    };
    let painter = ui.painter().with_clip_rect(rect);
    let total = scene.triangle_count().max(1);
    let stride = total.div_ceil(MAX_TRIANGLES);
    let mut counter = 0usize;
    for mesh in &scene.meshes {
        let color =
            mesh.material
                .and_then(|m| scene.materials.get(m))
                .map_or(colors::PRIMARY, |m| {
                    Color32::from_rgb(
                        (m.color[0] * 255.0) as u8,
                        (m.color[1] * 255.0) as u8,
                        (m.color[2] * 255.0) as u8,
                    )
                });
        let stroke = Stroke::new(1.0, colors::with_alpha(colors::with_alpha(color, 255), 170));
        for triangle in mesh.indices.chunks_exact(3) {
            counter += 1;
            if !counter.is_multiple_of(stride) {
                continue;
            }
            let points: Vec<_> = triangle
                .iter()
                .filter_map(|i| mesh.positions.get(*i as usize))
                .map(|p| to_screen(*p))
                .collect();
            if points.len() == 3 {
                painter.line_segment([points[0], points[1]], stroke);
                painter.line_segment([points[1], points[2]], stroke);
                painter.line_segment([points[2], points[0]], stroke);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn up_axis_maps_to_screen_up_without_pitch() {
        let p = project([0.0, 0.0, 2.0], 0.0, 0.0);
        assert_eq!(p, [0.0, 2.0]);
    }

    #[test]
    fn yaw_quarter_turn_swaps_x_and_y() {
        let p = project([1.0, 0.0, 0.0], std::f32::consts::FRAC_PI_2, 0.0);
        assert!(p[0].abs() < 1e-6);
    }
}
