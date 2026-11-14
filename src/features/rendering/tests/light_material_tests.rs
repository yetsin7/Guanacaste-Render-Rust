//! Environment, artificial lights and PBR materials on tiny scenes.

use super::scenes::{cube, horizontal_quad, paint, scene, wall_quad};
use crate::features::import_export::models::{ImportedScene, Mesh};
use crate::features::lighting::models::{EmissiveMaterial, Light, LightKind, Power};
use crate::features::materials::models::{Material, Procedural, ProceduralKind, TextureRef};
use crate::features::rendering::engine::{
    render_prepared, RenderCamera, RenderControl, RenderRequest, RenderScene, World,
};
use crate::features::rendering::models::RayTracingQuality;
use image::RgbaImage;
use std::sync::Arc;

fn render_world(world: &World, request: &RenderRequest) -> RgbaImage {
    let prepared = RenderScene::new(world);
    render_prepared(&prepared, request, &RenderControl::new(), &|_| {}).expect("render")
}

fn lum(image: &RgbaImage, x: u32, y: u32) -> i32 {
    let p = image.get_pixel(x, y).0;
    p[0] as i32 + p[1] as i32 + p[2] as i32
}

/// A camera above a 40 m ground looking straight down.
fn top_camera() -> RenderCamera {
    RenderCamera {
        position: [0.0, 0.0, 25.0],
        direction: [0.0, 0.0, -1.0],
        up: [0.0, 1.0, 0.0],
        fov_y_degrees: 50.0,
        width: 64,
        height: 48,
        ..RenderCamera::default()
    }
}

fn ground() -> ImportedScene {
    scene(
        vec![horizontal_quad([-30.0, 30.0], [-30.0, 30.0], 0.0, 0)],
        vec![paint(0.8, 0.8, 0.8)],
    )
}

fn night(camera: RenderCamera) -> RenderRequest {
    // High: the sample count and bounces that Medium had before the presets were retuned for faster renders.
    let mut request = RenderRequest::new(camera, RayTracingQuality::High);
    request.environment.hour = 0.5;
    request
}

/// A point light 3 m above the scene origin (the lighting world is Y up).
fn lamp(lumens: f32) -> Light {
    Light {
        position: [0.0, 3.0, 0.0],
        power: Power::Lumens { value: lumens },
        kelvin: 4000.0,
        ..Light::default()
    }
}

#[test]
fn the_sun_follows_the_time_of_day() {
    let world = World::new(ground());
    let mut request = RenderRequest::new(top_camera(), RayTracingQuality::Low);
    request.environment.hour = 12.0;
    let noon = render_world(&world, &request);
    request.environment.hour = 1.0;
    let midnight = render_world(&world, &request);
    assert!(
        lum(&noon, 32, 24) > lum(&midnight, 32, 24) * 3,
        "noon is much brighter"
    );
}

#[test]
fn fog_hides_distant_geometry() {
    let wall = scene(
        vec![wall_quad([-50.0, 50.0], [-50.0, 50.0], 400.0, 0)],
        vec![paint(0.05, 0.05, 0.05)],
    );
    let world = World::new(wall);
    let camera = RenderCamera {
        position: [0.0, 0.0, 0.0],
        width: 32,
        height: 24,
        far: 2000.0,
        ..RenderCamera::default()
    };
    let mut request = RenderRequest::new(camera, RayTracingQuality::Low);
    let clear = render_world(&world, &request);
    request.environment.weather.fog_density = 0.6;
    let foggy = render_world(&world, &request);
    assert!(
        lum(&foggy, 16, 12) > lum(&clear, 16, 12) + 150,
        "a dark wall turns into fog"
    );
}

#[test]
fn a_point_light_lights_the_ground_at_night() {
    let world = World::new(ground());
    let mut request = night(top_camera());
    let dark = render_world(&world, &request);
    request.lights = Arc::new(vec![lamp(40_000.0)]);
    let lit = render_world(&world, &request);
    assert!(
        lum(&lit, 32, 24) > lum(&dark, 32, 24) + 60,
        "under the lamp"
    );
    assert!(
        lum(&lit, 32, 24) > lum(&lit, 3, 3),
        "light falls off with distance"
    );
}

#[test]
fn shadows_can_be_switched_off_per_light() {
    // A wall at x = 2 hides the lamp from the ground at x = 4 only when the light casts shadows.
    let blocker = scene(
        vec![
            horizontal_quad([-30.0, 30.0], [-30.0, 30.0], 0.0, 0),
            cube([1.9, -5.0, 0.0], [2.1, 5.0, 6.0], 0),
        ],
        vec![paint(0.8, 0.8, 0.8)],
    );
    let world = World::new(blocker);
    let pixel = |shadows: bool| {
        let mut request = night(top_camera());
        request.lights = Arc::new(vec![Light {
            cast_shadows: shadows,
            ..lamp(60_000.0)
        }]);
        let image = render_world(&world, &request);
        // x = 4 m maps to column 32 + 4 / (2 * 25 * tan 25deg / 32) ~ 32 + 5.5; row at the center.
        lum(&image, 39, 24)
    };
    assert!(
        pixel(false) > pixel(true) + 20,
        "shadow on {} off {}",
        pixel(true),
        pixel(false)
    );
}

#[test]
fn an_area_light_lights_the_scene_with_soft_samples() {
    let world = World::new(ground());
    let mut request = night(top_camera());
    request.lights = Arc::new(vec![Light {
        kind: LightKind::Area {
            width: 2.0,
            height: 2.0,
            two_sided: true,
        },
        ..lamp(80_000.0)
    }]);
    let image = render_world(&world, &request);
    assert!(lum(&image, 32, 24) > lum(&image, 4, 4));
}

#[test]
fn emissive_materials_glow_in_the_dark() {
    let panel = scene(
        vec![wall_quad([-1.0, 1.0], [-1.0, 1.0], 5.0, 0)],
        vec![paint(0.2, 0.2, 0.2)],
    );
    let mut world = World::new(panel.clone());
    world.pbr[0].emission = [1.0, 0.9, 0.7];
    world.pbr[0].emission_strength = 4.0;
    let camera = RenderCamera {
        position: [0.0, 0.0, 0.0],
        width: 32,
        height: 24,
        ..RenderCamera::default()
    };
    let request = night(camera);
    let glowing = render_world(&world, &request);
    let plain = render_world(&World::new(panel.clone()), &request);
    assert!(lum(&glowing, 16, 12) > lum(&plain, 16, 12) + 100);
    // The lighting domain's emissive list works by material name.
    let mut by_name = World::new(panel);
    by_name.emissive.push(EmissiveMaterial {
        material: "paint".into(),
        luminance: 800.0,
        ..EmissiveMaterial::default()
    });
    assert!(lum(&render_world(&by_name, &request), 16, 12) > lum(&plain, 16, 12) + 100);
}

fn textured_wall(texture: Option<TextureRef>) -> World {
    let mut mesh = wall_quad([-3.0, 3.0], [-3.0, 3.0], 0.0, 0);
    mesh.uvs = vec![[0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0]];
    let mut world = World::new(scene(vec![mesh], vec![paint(0.9, 0.9, 0.9)]));
    world.pbr[0] = Material {
        base_color: [0.9, 0.9, 0.9, 1.0],
        roughness: 0.8,
        ..Material::new("paint")
    };
    world.pbr[0].uv.scale = [6.0, 6.0];
    world.pbr[0].maps.albedo = texture;
    world
}

fn wall_request() -> RenderRequest {
    let camera = RenderCamera {
        position: [0.0, -8.0, 0.0],
        width: 64,
        height: 48,
        ..RenderCamera::default()
    };
    RenderRequest::new(camera, RayTracingQuality::Medium)
}

fn spread(image: &RgbaImage) -> i32 {
    let values: Vec<i32> = (20..44)
        .flat_map(|x| (14..34).map(move |y| (x, y)))
        .map(|(x, y)| lum(image, x, y))
        .collect();
    values.iter().max().unwrap() - values.iter().min().unwrap()
}

#[test]
fn textured_materials_use_the_uv_coordinates() {
    let brick = TextureRef::procedural(Procedural::new(ProceduralKind::Brick));
    let flat = render_world(&textured_wall(None), &wall_request());
    let bricks = render_world(&textured_wall(Some(brick)), &wall_request());
    assert!(
        spread(&bricks) > spread(&flat) + 60,
        "{} vs {}",
        spread(&bricks),
        spread(&flat)
    );
}

#[test]
fn a_normal_map_changes_the_shading_of_a_flat_wall() {
    let bump = TextureRef {
        is_height: true,
        ..TextureRef::procedural(Procedural::new(ProceduralKind::Brick))
    };
    let mut world = textured_wall(None);
    world.pbr[0].maps.normal = Some(bump);
    world.pbr[0].normal_strength = 2.0;
    // A lamp at the side of the wall in the dark lights it at a grazing angle, where bumps show.
    let mut request = night(wall_request().camera);
    request.lights = Arc::new(vec![Light {
        position: [2.5, 0.0, 0.8],
        ..lamp(30_000.0)
    }]);
    let flat = render_world(&textured_wall(None), &request);
    let bumped = render_world(&world, &request);
    request.seed = 77;
    let flat_again = render_world(&textured_wall(None), &request);
    // The bumps change the picture far more than a different noise seed does.
    let difference = |a: &RgbaImage, b: &RgbaImage| -> i32 {
        (20..44)
            .flat_map(|x| (14..34).map(move |y| (x, y)))
            .map(|(x, y)| (lum(a, x, y) - lum(b, x, y)).abs())
            .sum()
    };
    let (by_bumps, by_noise) = (difference(&bumped, &flat), difference(&flat_again, &flat));
    assert!(by_bumps > by_noise * 3 / 2, "{by_bumps} vs {by_noise}");
}

#[test]
fn transparent_materials_show_what_is_behind_and_glass_reflects() {
    let mut glass = Material::new("vidrio");
    glass.transparency = 0.9;
    glass.roughness = 0.02;
    let mut world = World::new(scene(
        vec![
            wall_quad([-5.0, 5.0], [-5.0, 5.0], 4.0, 1),
            wall_quad([-5.0, 5.0], [-5.0, 5.0], 2.0, 0),
        ],
        vec![paint(0.5, 0.5, 0.5), paint(1.0, 0.0, 0.0)],
    ));
    world.pbr[0] = glass;
    let image = render_world(&world, &wall_request_at(-4.0));
    let p = image.get_pixel(32, 24).0;
    assert!(
        p[0] as i32 > p[2] as i32 + 30,
        "the red wall shows through the glass: {p:?}"
    );
}

fn wall_request_at(y: f32) -> RenderRequest {
    let mut request = wall_request();
    request.camera.position = [0.0, y, 0.0];
    request
}

#[test]
fn meshes_without_uvs_are_textured_in_meters() {
    let mut world = textured_wall(Some(TextureRef::procedural(Procedural::new(
        ProceduralKind::Brick,
    ))));
    world.scene.meshes[0] = Mesh {
        uvs: Vec::new(),
        ..world.scene.meshes[0].clone()
    };
    // One tile every 2.5 m of wall: the bricks are as big as in the UV-mapped wall.
    world.pbr[0].uv.scale = [0.4, 0.4];
    let image = render_world(&world, &wall_request());
    let flat = render_world(&textured_wall(None), &wall_request());
    assert!(spread(&image) > spread(&flat) + 60);
}
