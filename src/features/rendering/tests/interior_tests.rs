//! Rooms: light portals found in windows, the automatic interior camera, and daylight entering through glass.

use super::scenes::{horizontal_quad, paint, scene, wall_quad};
use crate::features::import_export::models::{ImportedScene, Material, Mesh};
use crate::features::rendering::engine::{
    interior_camera, render_scene, RenderCamera, RenderRequest, RenderScene, World,
};
use crate::features::rendering::models::RayTracingQuality;

/// A wall in the plane `x = x0` facing -X.
fn side_quad(y: [f32; 2], z: [f32; 2], x0: f32, material: usize) -> Mesh {
    Mesh {
        positions: vec![
            [x0, y[0], z[0]],
            [x0, y[1], z[0]],
            [x0, y[1], z[1]],
            [x0, y[0], z[1]],
        ],
        indices: vec![0, 1, 2, 0, 2, 3],
        material: Some(material),
        uvs: Vec::new(),
    }
}

fn glass() -> Material {
    Material {
        name: "glass".into(),
        color: [0.8, 0.9, 0.9, 0.1],
        texture: None,
        pbr: None,
    }
}

/// A 6 x 5 x 2.8 m room; its east wall is `east` (glass or paint).
fn room(east: Material) -> ImportedScene {
    scene(
        vec![
            horizontal_quad([0.0, 6.0], [0.0, 5.0], 0.0, 0),
            horizontal_quad([0.0, 6.0], [0.0, 5.0], 2.8, 0),
            wall_quad([0.0, 6.0], [0.0, 2.8], 0.0, 0),
            wall_quad([0.0, 6.0], [0.0, 2.8], 5.0, 0),
            side_quad([0.0, 5.0], [0.0, 2.8], 0.0, 0),
            side_quad([0.0, 5.0], [0.0, 2.8], 6.0, 1),
        ],
        vec![paint(0.8, 0.8, 0.8), east],
    )
}

fn inside_camera() -> RenderCamera {
    RenderCamera {
        position: [4.5, 2.5, 1.4],
        direction: [-1.0, 0.0, 0.0],
        width: 48,
        height: 32,
        ..RenderCamera::default()
    }
}

fn mean_brightness(scene: &ImportedScene) -> f32 {
    let image = render_scene(
        scene,
        &RenderRequest::new(inside_camera(), RayTracingQuality::Medium),
    );
    let sum: f32 = image
        .pixels()
        .map(|p| (p[0] as f32 + p[1] as f32 + p[2] as f32) / 3.0)
        .sum();
    sum / (image.width() * image.height()) as f32
}

#[test]
fn a_window_to_the_open_air_is_a_portal_and_an_opaque_wall_is_not() {
    let open = RenderScene::new(&World::new(room(glass())));
    assert!(!open.portals.is_empty());
    let closed = RenderScene::new(&World::new(room(paint(0.8, 0.8, 0.8))));
    assert!(closed.portals.is_empty());
}

#[test]
fn the_interior_camera_stands_inside_the_room() {
    let world = World::new(room(glass()));
    let scene = RenderScene::new(&world);
    let focus = world.focus.expect("focus");
    let camera = interior_camera(&scene, &focus, 64, 36).expect("a room");
    let [x, y, z] = camera.position;
    assert!(
        (0.5..5.5).contains(&x) && (0.5..4.5).contains(&y),
        "{x} {y}"
    );
    assert!((1.2..2.0).contains(&z), "eye height {z}");
    assert!(camera.direction[2].abs() < 0.2, "looks level");
}

#[test]
fn no_room_means_no_interior_camera() {
    let open_air = scene(
        vec![horizontal_quad([0.0, 20.0], [0.0, 20.0], 0.0, 0)],
        vec![paint(0.5, 0.5, 0.5)],
    );
    let world = World::new(open_air);
    let scene = RenderScene::new(&world);
    assert!(interior_camera(&scene, &world.focus.expect("focus"), 64, 36).is_none());
}

#[test]
fn daylight_comes_in_through_the_window() {
    let with_window = mean_brightness(&room(glass()));
    let walled = mean_brightness(&room(paint(0.8, 0.8, 0.8)));
    assert!(
        with_window > walled * 1.5 && with_window > 20.0,
        "window {with_window}, wall {walled}"
    );
}
