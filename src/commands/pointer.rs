//! Use case: the mouse over the 3D view drives the build tools. The pointer is converted to a ray from the view's
//! camera; the ray goes to the placed-objects tools (press, drag, release) or to the landscape brushes (a dab per
//! frame while the button is down). The left button is free for tools; the camera uses the right and middle ones
//! (and Alt + left).

use crate::features::build::logic::math::Ray;
use crate::features::landscape::logic::{actions, height_at};
use crate::features::landscape::models::LandscapeState;
use crate::features::viewport::models::Camera;
use crate::models::navigation::{BuildCategory, Mode};
use crate::models::AppState;
use eframe::egui::{Pos2, Rect, Response};

/// The ray through `pointer` for a view covering `rect`.
pub fn ray_through(camera: &Camera, rect: Rect, pointer: Pos2) -> Ray {
    let aspect = f64::from(rect.width() / rect.height().max(1.0));
    let tan = (f64::from(camera.fov_y_deg).to_radians() / 2.0).tan();
    let nx = f64::from((pointer.x - rect.center().x) / (rect.width() / 2.0));
    let ny = f64::from((rect.center().y - pointer.y) / (rect.height() / 2.0));
    let (f, r, u) = (camera.forward(), camera.right(), camera.up());
    let mut direction = [0.0; 3];
    for axis in 0..3 {
        direction[axis] = f64::from(f[axis])
            + f64::from(r[axis]) * nx * tan * aspect
            + f64::from(u[axis]) * ny * tan;
    }
    Ray::new(camera.position.map(f64::from), direction)
}

/// Where `ray` meets the ground of `landscape`, as world (x, y); `None` when it never does.
pub fn terrain_hit(landscape: &LandscapeState, ray: &Ray) -> Option<[f32; 2]> {
    let terrain = &landscape.landscape.terrain;
    let height = |t: f64| {
        let p = ray.at(t);
        p[2] as f32 - height_at(terrain, p[0] as f32, p[1] as f32)
    };
    let (mut near, mut previous) = (0.0_f64, height(0.0));
    let mut t = 0.5;
    while t < 3000.0 {
        let current = height(t);
        if previous > 0.0 && current <= 0.0 {
            let (mut low, mut high) = (near, t);
            for _ in 0..16 {
                let middle = (low + high) / 2.0;
                if height(middle) > 0.0 {
                    low = middle;
                } else {
                    high = middle;
                }
            }
            let point = ray.at(high);
            return Some([point[0] as f32, point[1] as f32]);
        }
        near = t;
        previous = current;
        t += 0.5 + t * 0.01;
    }
    None
}

/// Whether the pointer should drive a tool now (not while the camera uses the left button with Alt).
fn tools_enabled(app: &AppState) -> bool {
    app.mode == Mode::Build
        && matches!(
            app.category,
            BuildCategory::Nature | BuildCategory::WaterTerrain | BuildCategory::Objects
        )
}

/// Feeds the pointer of this frame to the tool of the active category. `rect` is the 3D view, `response` its
/// interaction, `dt` the frame time in seconds. Returns whether the scene changed.
pub fn drive(app: &mut AppState, response: &Response, rect: Rect, dt: f32) -> bool {
    if !tools_enabled(app) || rect.height() < 2.0 {
        return false;
    }
    let ctx = response.ctx.clone();
    let (pressed, down, released, alt, shift, pointer) = ctx.input(|input| {
        (
            input.pointer.primary_pressed(),
            input.pointer.primary_down(),
            input.pointer.primary_released(),
            input.modifiers.alt,
            input.modifiers.shift,
            input.pointer.hover_pos(),
        )
    });
    let camera = *app.viewport.camera();
    let ray = pointer.map(|p| ray_through(&camera, rect, p));
    match app.category {
        BuildCategory::Objects => {
            let Some(ray) = ray else {
                if released {
                    app.objects.pointer_release();
                }
                return false;
            };
            if pressed && response.hovered() && !alt {
                app.objects.pointer_press(&ray, shift);
            } else if down && app.objects.drag.is_some() {
                app.objects.pointer_drag(&ray);
                ctx.request_repaint();
            }
            if released {
                app.objects.pointer_release();
            }
            false
        }
        _ => {
            if !app.landscape.landscape.visible || alt || !down {
                return false;
            }
            let started_here = response.is_pointer_button_down_on() || response.dragged();
            let Some(ray) = ray.filter(|_| started_here) else {
                return false;
            };
            match terrain_hit(&app.landscape, &ray) {
                Some([x, y]) => actions::apply_stroke(&mut app.landscape, x, y, dt.min(0.1)),
                None => false,
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use eframe::egui::{pos2, vec2};

    #[test]
    fn the_center_of_the_view_looks_along_the_camera() {
        let camera = Camera::looking_at([0.0, -10.0, 5.0], [0.0, 0.0, 5.0]);
        let rect = Rect::from_min_size(pos2(0.0, 0.0), vec2(800.0, 600.0));
        let ray = ray_through(&camera, rect, rect.center());
        assert!(ray.direction[1] > 0.99);
        assert_eq!(ray.origin, [0.0, -10.0, 5.0]);
    }

    #[test]
    fn a_ray_aimed_downwards_meets_the_flat_ground_at_the_right_place() {
        let landscape = LandscapeState::default();
        let ray = Ray::new([3.0, 4.0, 10.0], [0.0, 0.0, -1.0]);
        let hit = terrain_hit(&landscape, &ray).expect("hit");
        assert!((hit[0] - 3.0).abs() < 0.05 && (hit[1] - 4.0).abs() < 0.05);
        let up = Ray::new([3.0, 4.0, 10.0], [0.0, 0.0, 1.0]);
        assert!(terrain_hit(&landscape, &up).is_none());
    }

    #[test]
    fn the_right_edge_of_the_view_turns_to_the_right() {
        let camera = Camera::looking_at([0.0, -10.0, 5.0], [0.0, 0.0, 5.0]);
        let rect = Rect::from_min_size(pos2(0.0, 0.0), vec2(800.0, 600.0));
        let ray = ray_through(&camera, rect, pos2(800.0, 300.0));
        assert!(ray.direction[0] > 0.2);
    }
}
