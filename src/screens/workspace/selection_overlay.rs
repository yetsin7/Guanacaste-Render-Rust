//! What marks the selected objects in the 3D view: an outline box around each one and the gizmo of the active
//! tool (arrows to move, a ring to rotate, end handles to scale) at the middle of the selection. Both are drawn
//! over the view by projecting world points with the view's own camera, so they follow every navigation.

use crate::config::theme::colors;
use crate::features::build::logic::math::{to_world, Vec3};
use crate::features::build::models::{AxisLock, GizmoMode};
use crate::features::viewport::models::Camera;
use crate::models::navigation::{BuildCategory, Mode};
use crate::models::AppState;
use eframe::egui::{pos2, Color32, Painter, Pos2, Rect, Stroke, Ui};

/// Length of a gizmo axis on screen, in points.
const AXIS_POINTS: f32 = 78.0;

/// Where `point` falls on a view covering `rect`; `None` behind the camera.
pub fn project(camera: &Camera, rect: Rect, point: Vec3) -> Option<Pos2> {
    let (f, r, u) = (camera.forward(), camera.right(), camera.up());
    let v = [
        point[0] - f64::from(camera.position[0]),
        point[1] - f64::from(camera.position[1]),
        point[2] - f64::from(camera.position[2]),
    ];
    let dot = |a: [f32; 3]| (0..3).map(|i| f64::from(a[i]) * v[i]).sum::<f64>();
    let depth = dot(f);
    if depth < f64::from(camera.near.max(0.01)) {
        return None;
    }
    let tan = (f64::from(camera.fov_y_deg).to_radians() / 2.0).tan();
    let aspect = f64::from(rect.width() / rect.height().max(1.0));
    let x = dot(r) / (depth * tan * aspect);
    let y = dot(u) / (depth * tan);
    Some(pos2(
        rect.center().x + x as f32 * rect.width() / 2.0,
        rect.center().y - y as f32 * rect.height() / 2.0,
    ))
}

/// The corners of the oriented box around an object, in world space.
fn box_corners(app: &mut AppState, id: u64) -> Option<[Vec3; 8]> {
    let object = app.objects.instance(id)?.clone();
    let scene = app.objects.cache.scene(object.asset)?;
    let bounds = scene.bounds()?;
    let mut corners = [[0.0; 3]; 8];
    for (index, corner) in corners.iter_mut().enumerate() {
        let local = [0, 1, 2].map(|axis| {
            let side = if index >> axis & 1 == 0 {
                bounds.min[axis]
            } else {
                bounds.max[axis]
            };
            f64::from(side)
        });
        *corner = to_world(&object.transform, local);
    }
    Some(corners)
}

fn outline(painter: &Painter, camera: &Camera, rect: Rect, corners: &[Vec3; 8]) {
    let screen: Vec<Option<Pos2>> = corners.iter().map(|c| project(camera, rect, *c)).collect();
    let stroke = Stroke::new(1.5, colors::PRIMARY);
    for a in 0..8usize {
        for bit in 0..3 {
            let b = a | (1 << bit);
            if b != a {
                if let (Some(p), Some(q)) = (screen[a], screen[b]) {
                    painter.line_segment([p, q], stroke);
                }
            }
        }
    }
}

fn axis_color(axis: usize, lock: AxisLock) -> Color32 {
    let base = [colors::ERROR, colors::TERTIARY, colors::PRIMARY][axis];
    let locked = matches!(
        (lock, axis),
        (AxisLock::X, 0) | (AxisLock::Y, 1) | (AxisLock::Z, 2)
    );
    if lock == AxisLock::Free || locked {
        base
    } else {
        colors::with_alpha(base, 90)
    }
}

fn gizmo(
    painter: &Painter,
    camera: &Camera,
    rect: Rect,
    pivot: Vec3,
    mode: GizmoMode,
    lock: AxisLock,
) {
    let Some(center) = project(camera, rect, pivot) else {
        return;
    };
    let distance = (0..3)
        .map(|i| (pivot[i] - f64::from(camera.position[i])).powi(2))
        .sum::<f64>()
        .sqrt()
        .max(0.1);
    let tan = (f64::from(camera.fov_y_deg).to_radians() / 2.0).tan();
    // World length that spans `AXIS_POINTS` points whatever the distance.
    let length = distance * tan * 2.0 * f64::from(AXIS_POINTS / rect.height().max(1.0));
    painter.circle_filled(center, 4.0, colors::ON_SURFACE);
    if mode == GizmoMode::Rotate {
        let ink = Stroke::new(2.5, colors::TERTIARY);
        let mut last = None;
        for step in 0..=48 {
            let angle = step as f64 / 48.0 * std::f64::consts::TAU;
            let point = [
                pivot[0] + angle.cos() * length,
                pivot[1] + angle.sin() * length,
                pivot[2],
            ];
            let now = project(camera, rect, point);
            if let (Some(a), Some(b)) = (last, now) {
                painter.line_segment([a, b], ink);
            }
            last = now;
        }
        return;
    }
    for axis in 0..3 {
        let mut end = pivot;
        end[axis] += length;
        let Some(tip) = project(camera, rect, end) else {
            continue;
        };
        let color = axis_color(axis, lock);
        painter.line_segment([center, tip], Stroke::new(3.0, color));
        if mode == GizmoMode::Scale {
            painter.rect_filled(
                Rect::from_center_size(tip, eframe::egui::vec2(10.0, 10.0)),
                1.0,
                color,
            );
        } else {
            let direction = (tip - center).normalized();
            let side = eframe::egui::vec2(-direction.y, direction.x);
            let base = tip - direction * 9.0;
            painter.add(eframe::egui::Shape::convex_polygon(
                vec![tip + direction * 7.0, base + side * 5.0, base - side * 5.0],
                color,
                Stroke::NONE,
            ));
        }
    }
}

/// Draws the outlines and the gizmo of the selection over the 3D view `rect` (only in the objects tools).
pub fn show(ui: &mut Ui, rect: Rect, app: &mut AppState) {
    if app.mode != Mode::Build
        || app.category != BuildCategory::Objects
        || app.objects.selection.is_empty()
        || rect.height() < 2.0
    {
        return;
    }
    let camera = *app.viewport.camera();
    let painter = ui.painter_at(rect);
    let ids: Vec<u64> = app.objects.selection.iter().copied().collect();
    let mut sum = [0.0; 3];
    let mut count = 0.0;
    for id in ids {
        if let Some(corners) = box_corners(app, id) {
            outline(&painter, &camera, rect, &corners);
        }
        if let Some(object) = app.objects.instance(id) {
            for axis in 0..3 {
                sum[axis] += object.transform.position[axis];
            }
            count += 1.0;
        }
    }
    if count > 0.0 {
        let pivot = sum.map(|v| v / count);
        gizmo(
            &painter,
            &camera,
            rect,
            pivot,
            app.objects.mode,
            app.objects.axis,
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use eframe::egui::vec2;

    #[test]
    fn the_point_the_camera_looks_at_projects_to_the_middle() {
        let camera = Camera::looking_at([0.0, -10.0, 2.0], [0.0, 0.0, 2.0]);
        let rect = Rect::from_min_size(pos2(0.0, 0.0), vec2(800.0, 600.0));
        let p = project(&camera, rect, [0.0, 0.0, 2.0]).expect("in front");
        assert!((p.x - 400.0).abs() < 0.5 && (p.y - 300.0).abs() < 0.5);
        assert!(
            project(&camera, rect, [0.0, -20.0, 2.0]).is_none(),
            "behind"
        );
        let right = project(&camera, rect, [3.0, 0.0, 2.0]).expect("in front");
        assert!(right.x > 400.0);
    }

    #[test]
    fn projection_and_picking_rays_agree() {
        let camera = Camera::looking_at([5.0, -8.0, 3.0], [1.0, 2.0, 1.0]);
        let rect = Rect::from_min_size(pos2(10.0, 20.0), vec2(900.0, 500.0));
        let target = [2.0, 1.0, 0.5];
        let screen = project(&camera, rect, target).expect("in front");
        let ray = crate::commands::pointer::ray_through(&camera, rect, screen);
        let along: Vec3 = [
            target[0] - ray.origin[0],
            target[1] - ray.origin[1],
            target[2] - ray.origin[2],
        ];
        let t = (0..3).map(|i| along[i] * ray.direction[i]).sum::<f64>();
        for i in 0..3 {
            assert!((ray.origin[i] + ray.direction[i] * t - target[i]).abs() < 1e-3);
        }
    }
}
