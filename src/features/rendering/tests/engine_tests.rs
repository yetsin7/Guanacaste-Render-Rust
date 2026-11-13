//! The CPU renderer on tiny scenes: sky, shadows, depth order, antialiasing, tiles, cancellation and framing.

use super::scenes::{cube, empty, horizontal_quad, paint, scene, wall_quad};
use crate::features::import_export::models::ImportedScene;
use crate::features::rendering::engine::{
    frame_bounds, render_prepared, render_scene, RenderCamera, RenderControl, RenderError,
    RenderRequest, RenderScene, TileUpdate, ViewPreset, World,
};
use crate::features::rendering::models::RayTracingQuality;
use image::RgbaImage;
use std::sync::atomic::{AtomicUsize, Ordering};

fn camera(width: u32, height: u32) -> RenderCamera {
    RenderCamera {
        width,
        height,
        ..RenderCamera::default()
    }
}

fn request(camera: RenderCamera, quality: RayTracingQuality) -> RenderRequest {
    RenderRequest::new(camera, quality)
}

fn pixel(image: &RgbaImage, x: u32, y: u32) -> [i32; 4] {
    let p = image.get_pixel(x, y).0;
    [p[0] as i32, p[1] as i32, p[2] as i32, p[3] as i32]
}

/// Mean brightness of the 3x3 block around `(x, y)`.
fn brightness(image: &RgbaImage, x: u32, y: u32) -> f32 {
    let mut sum = 0.0;
    for dy in 0..3 {
        for dx in 0..3 {
            let p = pixel(image, x - 1 + dx, y - 1 + dy);
            sum += (p[0] + p[1] + p[2]) as f32 / 3.0;
        }
    }
    sum / 9.0
}

#[test]
fn sky_pixels_are_blue_and_bluer_toward_the_zenith() {
    let image = render_scene(&empty(), &request(camera(64, 48), RayTracingQuality::Low));
    let horizon = pixel(&image, 32, 24);
    let high = pixel(&image, 32, 0);
    assert!(
        horizon[2] > horizon[1] && horizon[1] > horizon[0],
        "{horizon:?}"
    );
    assert_eq!(horizon[3], 255);
    assert!(high[2] - high[0] > horizon[2] - horizon[0]);
}

#[test]
fn a_transparent_background_leaves_the_sky_clear_and_the_geometry_solid() {
    let red = scene(
        vec![wall_quad([-2.0, 2.0], [-2.0, 2.0], 0.0, 0)],
        vec![paint(1.0, 0.0, 0.0)],
    );
    let mut req = request(camera(64, 48), RayTracingQuality::Low);
    req.camera.position = [0.0, -10.0, 0.0];
    req.transparent_background = true;
    let image = render_scene(&red, &req);
    assert_eq!(pixel(&image, 32, 24)[3], 255);
    assert_eq!(pixel(&image, 1, 1)[3], 0);
}

#[test]
fn the_sun_casts_a_hard_shadow_of_a_cube_on_the_ground() {
    let scene = super::scenes::cube_on_ground();
    let mut req = request(
        RenderCamera {
            position: [0.0, 0.0, 30.0],
            direction: [0.0, 0.0, -1.0],
            up: [0.0, 1.0, 0.0],
            fov_y_degrees: 40.0,
            width: 64,
            height: 48,
            ..RenderCamera::default()
        },
        RayTracingQuality::Medium,
    );
    req.environment.hour = 9.0;
    let image = render_scene(&scene, &req);
    // The light travels toward -X, so the shadow falls to the left of the cube; the right side is lit.
    let shadow = brightness(&image, 28, 24);
    let lit = brightness(&image, 45, 24);
    assert!(lit > shadow + 40.0, "lit {lit} vs shadow {shadow}");
    // Sky light still reaches the shadow: it is dim, not black.
    assert!(shadow > 5.0, "shadow {shadow}");
}

fn depth_scene(near_first: bool) -> ImportedScene {
    let near = wall_quad([-5.0, 5.0], [-5.0, 5.0], -3.0, 0);
    let far = wall_quad([-5.0, 5.0], [-5.0, 5.0], 5.0, 1);
    let meshes = if near_first {
        vec![near, far]
    } else {
        vec![far, near]
    };
    scene(meshes, vec![paint(1.0, 0.0, 0.0), paint(0.0, 0.0, 1.0)])
}

#[test]
fn the_nearest_surface_wins_whatever_the_mesh_order() {
    for near_first in [true, false] {
        let mut req = request(camera(32, 24), RayTracingQuality::Low);
        req.camera.position = [0.0, -10.0, 0.0];
        let image = render_scene(&depth_scene(near_first), &req);
        let center = pixel(&image, 16, 12);
        assert!(
            center[0] > center[2] + 40,
            "near_first={near_first}: {center:?}"
        );
    }
}

#[test]
fn the_far_plane_hides_distant_geometry() {
    let mut req = request(camera(32, 24), RayTracingQuality::Low);
    req.camera.position = [0.0, -10.0, 0.0];
    req.camera.far = 5.0;
    req.transparent_background = true;
    let image = render_scene(&depth_scene(true), &req);
    assert_eq!(pixel(&image, 16, 12)[3], 0);
}

#[test]
fn edges_are_antialiased() {
    let wall = scene(
        vec![wall_quad([-30.0, 0.43], [-30.0, 30.0], 0.0, 0)],
        vec![paint(1.0, 0.0, 0.0)],
    );
    let mut req = request(camera(64, 48), RayTracingQuality::High);
    req.camera.position = [0.0, -10.0, 0.0];
    let image = render_scene(&wall, &req);
    let wall_green = pixel(&image, 20, 24)[1];
    let sky_green = pixel(&image, 50, 24)[1];
    let edge_green = pixel(&image, 34, 24)[1];
    assert!(sky_green > wall_green + 80, "{wall_green} vs {sky_green}");
    assert!(
        edge_green > wall_green + 20 && edge_green < sky_green - 20,
        "edge {edge_green} between {wall_green} and {sky_green}"
    );
}

#[test]
fn the_same_request_gives_the_same_picture_and_a_new_seed_a_different_noise() {
    let scene = super::scenes::cube_on_ground();
    let bounds = scene.bounds().expect("bounds");
    let mut req = request(
        frame_bounds(&bounds, ViewPreset::Corner, 48, 32),
        RayTracingQuality::Low,
    );
    let first = render_scene(&scene, &req);
    assert_eq!(first, render_scene(&scene, &req));
    req.seed = 99;
    assert_ne!(first, render_scene(&scene, &req));
}

#[test]
fn tiles_cover_the_picture_and_progress_reaches_one() {
    let prepared = RenderScene::new(&World::new(super::scenes::cube_on_ground()));
    let req = request(camera(70, 40), RayTracingQuality::Low);
    let control = RenderControl::new();
    let tiles = AtomicUsize::new(0);
    let pixels = AtomicUsize::new(0);
    let on_tile = |tile: &TileUpdate| {
        tiles.fetch_add(1, Ordering::SeqCst);
        pixels.fetch_add((tile.width * tile.height) as usize, Ordering::SeqCst);
        assert_eq!(tile.rgba.len(), (tile.width * tile.height * 4) as usize);
    };
    let image = render_prepared(&prepared, &req, &control, &on_tile).expect("not cancelled");
    assert_eq!((image.width(), image.height()), (70, 40));
    assert_eq!(tiles.load(Ordering::SeqCst), 3 * 2);
    assert_eq!(pixels.load(Ordering::SeqCst), 70 * 40);
    assert!((control.progress() - 1.0).abs() < 1e-6);
}

#[test]
fn a_cancelled_render_returns_the_cancel_error() {
    let prepared = RenderScene::new(&World::new(super::scenes::cube_on_ground()));
    let req = request(camera(64, 64), RayTracingQuality::Ultra);
    let control = RenderControl::new();
    control.cancel();
    let result = render_prepared(&prepared, &req, &control, &|_| {});
    assert_eq!(result.err(), Some(RenderError::Cancelled));
}

#[test]
fn the_corner_view_frames_the_whole_model() {
    let model = scene(
        vec![cube([-1.0, -1.0, 0.0], [1.0, 1.0, 2.0], 0)],
        vec![paint(0.8, 0.8, 0.8)],
    );
    let bounds = model.bounds().expect("bounds");
    for preset in ViewPreset::ALL {
        let mut req = request(
            frame_bounds(&bounds, preset, 64, 40),
            RayTracingQuality::Low,
        );
        req.transparent_background = true;
        let image = render_scene(&model, &req);
        assert_eq!(pixel(&image, 32, 20)[3], 255, "{preset:?} center");
        for (x, y) in [(0, 0), (63, 0), (0, 39), (63, 39)] {
            assert_eq!(pixel(&image, x, y)[3], 0, "{preset:?} corner {x},{y}");
        }
    }
}

#[test]
fn invalid_triangles_are_skipped_instead_of_breaking_the_render() {
    let mut broken = horizontal_quad([-1.0, 1.0], [-1.0, 1.0], 0.0, 0);
    broken.indices.extend([0, 1, 99]);
    broken.positions[2] = [f32::NAN, 0.0, 0.0];
    let model = scene(vec![broken], vec![paint(0.5, 0.5, 0.5)]);
    let image = render_scene(&model, &request(camera(16, 12), RayTracingQuality::Low));
    assert_eq!((image.width(), image.height()), (16, 12));
}
