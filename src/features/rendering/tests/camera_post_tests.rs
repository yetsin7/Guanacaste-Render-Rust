//! Cameras of the other domains, exposure, filters, depth of field and denoising on tiny scenes.

use super::scenes::{empty, horizontal_quad, paint, scene, wall_quad};
use crate::features::camera::models::camera::PhotoCamera;
use crate::features::camera::models::filter::FilterKind;
use crate::features::camera::models::stack::FilterStack;
use crate::features::projects::models::CameraDoc;
use crate::features::rendering::engine::{render_scene, RenderCamera, RenderRequest, World};
use crate::features::rendering::logic::inputs::{CameraChoice, RenderInputs};
use crate::features::rendering::logic::plan::{build_spec, plan_from_studio};
use crate::features::rendering::models::{RayTracingQuality, RenderStudioState};
use crate::features::viewport::models::Camera;
use image::RgbaImage;
use std::sync::Arc;

fn px(image: &RgbaImage, x: u32, y: u32) -> [i32; 3] {
    let p = image.get_pixel(x, y).0;
    [p[0] as i32, p[1] as i32, p[2] as i32]
}

fn sum(p: [i32; 3]) -> i32 {
    p[0] + p[1] + p[2]
}

fn small_camera() -> RenderCamera {
    RenderCamera {
        width: 48,
        height: 32,
        position: [0.0, -10.0, 0.0],
        ..RenderCamera::default()
    }
}

fn red_wall() -> crate::features::import_export::models::ImportedScene {
    scene(
        vec![wall_quad([-3.0, 3.0], [-3.0, 3.0], 0.0, 0)],
        vec![paint(0.9, 0.2, 0.2)],
    )
}

#[test]
fn the_viewport_camera_becomes_a_render_camera() {
    let view = Camera::looking_at([0.0, -10.0, 2.0], [0.0, 0.0, 2.0]);
    let camera = RenderCamera::from_viewport(&view, 640, 360);
    assert_eq!(camera.position, [0.0, -10.0, 2.0]);
    assert!((camera.direction[1] - 1.0).abs() < 1e-5);
    assert_eq!(camera.fov_y_degrees, view.fov_y_deg);
    assert_eq!((camera.width, camera.height), (640, 360));
    assert_eq!((camera.near, camera.far), (view.near, view.far));
}

#[test]
fn a_photographic_camera_sets_field_of_view_framing_and_clip_planes() {
    let mut photo = PhotoCamera::default();
    photo.set_focal_length(24.0);
    photo.set_clip(0.3, 800.0);
    let camera = RenderCamera::from_photo(&photo, [0.0, -5.0, 1.5], [0.0, 0.0, 1.5], 900);
    assert!((camera.fov_y_degrees - photo.vertical_fov().to_degrees()).abs() < 1e-3);
    assert_eq!(camera.width, 900);
    assert_eq!(camera.height, 600); // 3:2 framing
    assert_eq!((camera.near, camera.far), (0.3, 800.0));
    // A wider lens sees more.
    let mut tele = photo;
    tele.set_focal_length(85.0);
    let narrow = RenderCamera::from_photo(&tele, [0.0; 3], [0.0, 1.0, 0.0], 900);
    assert!(narrow.fov_y_degrees < camera.fov_y_degrees);
}

#[test]
fn saved_cameras_keep_their_lens_and_become_queue_jobs() {
    let saved = CameraDoc {
        name: "Fachada".into(),
        position: [0.0, -10.0, 1.6],
        target: [0.0, 0.0, 1.6],
        focal_mm: 35.0,
        aperture: 5.6,
    };
    let camera = RenderCamera::from_saved(&saved, 1280, 720);
    // 35 mm on a 36 mm wide sensor and a 16:9 picture.
    let expected = 2.0 * (36.0f32 / (16.0 / 9.0) / 2.0 / 35.0).atan().to_degrees();
    assert!((camera.fov_y_degrees - expected).abs() < 1e-3);
    let inputs = RenderInputs {
        cameras: vec![saved],
        ..RenderInputs::default()
    };
    let world = Arc::new(World::new(red_wall()));
    let mut plan = plan_from_studio(&RenderStudioState::default());
    plan.camera = CameraChoice::Saved(0);
    let spec = build_spec(&world, &inputs, &plan, false).expect("saved camera job");
    assert!(spec.name.contains("Fachada"), "{}", spec.name);
    assert!(spec.request.post.photo.is_some());
    assert_eq!(spec.request.camera.position, [0.0, -10.0, 1.6]);
    plan.camera = CameraChoice::Saved(7);
    assert!(build_spec(&world, &inputs, &plan, false).is_none());
}

#[test]
fn camera_exposure_brightens_and_darkens_the_picture() {
    let render = |compensation: f32| {
        let mut request = RenderRequest::new(small_camera(), RayTracingQuality::Low);
        let mut photo = PhotoCamera::default();
        photo.exposure.compensation_ev = compensation;
        request.post.photo = Some(photo);
        render_scene(&red_wall(), &request)
    };
    let (dark, normal, bright) = (render(-2.0), render(0.0), render(2.0));
    let (d, n, b) = (
        sum(px(&dark, 24, 16)),
        sum(px(&normal, 24, 16)),
        sum(px(&bright, 24, 16)),
    );
    assert!(d < n && n < b, "{d} {n} {b}");
}

#[test]
fn the_filter_stack_is_applied_to_the_finished_picture() {
    let mut request = RenderRequest::new(small_camera(), RayTracingQuality::Low);
    let plain = render_scene(&red_wall(), &request);
    let p = px(&plain, 24, 16);
    assert!(p[0] > p[2] + 30, "the wall is red: {p:?}");
    let mut stack = FilterStack::new();
    stack.push(FilterKind::BlackWhite);
    request.post.filters = stack;
    let gray = render_scene(&red_wall(), &request);
    let g = px(&gray, 24, 16);
    assert!(
        (g[0] - g[1]).abs() <= 2 && (g[1] - g[2]).abs() <= 2,
        "{g:?}"
    );
}

/// A red wall edge 60 m away seen with a lens focused 3 m away.
fn far_edge_request(depth_of_field: bool) -> RenderRequest {
    let mut request = RenderRequest::new(
        RenderCamera {
            width: 320,
            height: 12,
            position: [0.0, -60.0, 0.0],
            ..RenderCamera::default()
        },
        RayTracingQuality::High,
    );
    let mut photo = PhotoCamera::default();
    photo.set_aperture(1.4);
    photo.set_focus_distance(3.0);
    photo.exposure.shutter_s =
        crate::features::camera::logic::exposure::shutter_for_ev(1.4, 100.0, 12.0);
    request.post.photo = Some(photo);
    request.post.depth_of_field = depth_of_field;
    request
}

#[test]
fn depth_of_field_blurs_what_is_out_of_focus() {
    let wall = scene(
        vec![wall_quad([-60.0, 0.0], [-60.0, 60.0], 0.0, 0)],
        vec![paint(1.0, 0.0, 0.0)],
    );
    let transitions = |image: &RgbaImage| {
        (0..320)
            .filter(|x| {
                let g = px(image, *x, 6)[1];
                g > 60 && g < 235
            })
            .count()
    };
    let sharp = render_scene(&wall, &far_edge_request(false));
    let blurred = render_scene(&wall, &far_edge_request(true));
    assert!(
        transitions(&blurred) > transitions(&sharp) + 2,
        "{} vs {}",
        transitions(&blurred),
        transitions(&sharp)
    );
}

#[test]
fn denoising_smooths_the_noise_but_keeps_the_edges() {
    let ground = scene(
        vec![
            horizontal_quad([-20.0, 20.0], [-20.0, 20.0], 0.0, 0),
            wall_quad([-30.0, 0.0], [0.0, 30.0], 5.0, 0),
        ],
        vec![paint(0.6, 0.6, 0.6)],
    );
    let camera = RenderCamera {
        width: 64,
        height: 48,
        position: [0.0, -12.0, 4.0],
        direction: [0.0, 1.0, -0.2],
        ..RenderCamera::default()
    };
    let mut request = RenderRequest::new(camera, RayTracingQuality::Low);
    let noisy = render_scene(&ground, &request);
    request.post.denoise = true;
    let clean = render_scene(&ground, &request);
    let variance = |image: &RgbaImage| {
        let values: Vec<f32> = (30..60)
            .flat_map(|x| (36..44).map(move |y| (x, y)))
            .map(|(x, y)| sum(px(image, x, y)) as f32)
            .collect();
        let mean = values.iter().sum::<f32>() / values.len() as f32;
        values.iter().map(|v| (v - mean).powi(2)).sum::<f32>() / values.len() as f32
    };
    assert!(variance(&clean) < variance(&noisy) * 0.7);
    // The wall edge (left of x = 32 at the horizon) stays a step, not a ramp.
    let edge = |image: &RgbaImage| (sum(px(image, 10, 20)) - sum(px(image, 50, 20))).abs();
    assert!(edge(&clean) > edge(&noisy) / 2);
}

#[test]
fn an_empty_scene_still_renders_with_every_post_effect_on() {
    let mut request = RenderRequest::new(small_camera(), RayTracingQuality::Low);
    request.post.denoise = true;
    request.post.photo = Some(PhotoCamera::default());
    request.post.depth_of_field = true;
    let image = render_scene(&empty(), &request);
    assert_eq!((image.width(), image.height()), (48, 32));
}
