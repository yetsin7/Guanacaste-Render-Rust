//! The world a project renders (ground, landscape, objects, vegetation), what the document persists for the
//! render, and the photo renderer as the source of movie frames.

use super::scenes::{cube, paint, scene};
use crate::features::build::models::{PlacedObject, Transform};
use crate::features::camera::models::filter::FilterKind;
use crate::features::environment::models::EnvironmentSettings;
use crate::features::landscape::models::{Landscape, VegetationInstance, WaterBody};
use crate::features::library::models::{AssetRef, ProceduralKind};
use crate::features::lighting::logic::{active_lights, add_light};
use crate::features::lighting::models::LightKind;
use crate::features::movie::models::{Animation, CameraPose};
use crate::features::projects::logic::convert;
use crate::features::projects::models::{Project, ProjectDocument};
use crate::features::rendering::engine::World;
use crate::features::rendering::logic::inputs::RenderInputs;
use crate::features::rendering::logic::world::{build_world, vegetation_kind, GROUND_SINK};
use crate::features::rendering::models::RayTracingQuality;
use crate::features::rendering::services::frame_source::{FrameRequest, FrameSource};
use crate::features::rendering::services::movie_source::PhotoFrameSource;
use crate::features::rendering::services::sequence::{
    export_sequence, ExportError, ExportSettings,
};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

fn project() -> Project {
    let model = scene(
        vec![cube([-2.0, -2.0, 0.0], [2.0, 2.0, 3.0], 0)],
        vec![paint(0.8, 0.8, 0.8)],
    );
    let (document, meshes) = convert::from_scene(&model, "Casa", "casa.obj", 0);
    Project {
        id: [1; 16],
        document,
        meshes,
        path: None,
        legacy_source: None,
        read_only: false,
        dirty: false,
    }
}

#[test]
fn every_world_has_the_ground_and_frames_the_model_without_it() {
    let world = build_world(&project());
    let model_triangles = 12;
    assert!(
        world.scene.triangle_count() > model_triangles + 1000,
        "terrain triangles"
    );
    let focus = world.focus.expect("model bounds");
    assert_eq!(focus.min, [-2.0, -2.0, 0.0]);
    assert_eq!(focus.max, [2.0, 2.0, 3.0]);
    let all = world.scene.bounds().expect("bounds");
    assert!(all.min[2] < -GROUND_SINK + 1e-6 || all.min[2] <= 0.0);
    assert!(
        all.max[0] > 100.0,
        "the ground reaches far beyond the house"
    );
    assert_eq!(world.pbr.len(), world.scene.materials.len());
}

#[test]
fn project_materials_keep_their_pbr_values() {
    let mut project = project();
    project.document.materials[0].pbr = Some({
        let mut m = crate::features::materials::models::Material::new("Concreto");
        m.roughness = 0.12;
        m.metallic = 1.0;
        m
    });
    let world = build_world(&project);
    assert_eq!(world.pbr[0].roughness, 0.12);
    assert_eq!(world.pbr[0].metallic, 1.0);
}

#[test]
fn water_and_vegetation_of_the_landscape_join_the_scene() {
    let base = build_world(&project()).scene.triangle_count();
    let mut with_water = project();
    let mut landscape = Landscape::default();
    landscape.waters.push(WaterBody::plane([0.0, 0.0], 0.5));
    with_water.document.landscape = Some(landscape.clone());
    assert!(build_world(&with_water).scene.triangle_count() > base);
    landscape.vegetation.push(VegetationInstance {
        asset: "Palma".into(),
        position: [20.0, 10.0, 0.0],
        rotation_deg: 30.0,
        scale: 1.5,
    });
    let before = build_world(&with_water).scene.triangle_count();
    with_water.document.landscape = Some(landscape);
    assert!(build_world(&with_water).scene.triangle_count() > before);
    assert_eq!(vegetation_kind("Palma real"), ProceduralKind::Palm);
    assert_eq!(vegetation_kind("Guanacaste"), ProceduralKind::Tree);
}

#[test]
fn placed_objects_appear_at_their_transform() {
    let mut project = project();
    let before = build_world(&project);
    project.document.objects.instances.push(PlacedObject {
        id: 1,
        asset: AssetRef::Procedural {
            kind: ProceduralKind::Cube,
            seed: 0,
        },
        transform: Transform::at([40.0, 0.0, 0.0]),
    });
    let after = build_world(&project);
    assert_eq!(
        after.scene.triangle_count(),
        before.scene.triangle_count() + 12
    );
    let focus = after.focus.expect("focus includes the objects");
    assert!(focus.max[0] > 39.0);
}

#[test]
fn the_document_persists_environment_filters_and_stays_backward_compatible() {
    let old: ProjectDocument = serde_json::from_str("{}").expect("an old document opens");
    assert!(old.environment.is_none());
    assert!(old.filters.is_empty());
    let text = serde_json::to_string(&old).expect("serializes");
    assert!(!text.contains("environment") && !text.contains("filters"));
    let mut document = old;
    let mut env = EnvironmentSettings {
        hour: 7.25,
        ..EnvironmentSettings::default()
    };
    env.weather.fog_density = 0.3;
    document.environment = Some(env);
    document.filters.push(FilterKind::Vignette);
    let back: ProjectDocument =
        serde_json::from_str(&serde_json::to_string(&document).expect("serializes"))
            .expect("reads");
    assert_eq!(back.environment, Some(env));
    assert_eq!(back.filters.len(), 1);
    let inputs = RenderInputs::from_document(&back);
    assert_eq!(inputs.environment.hour, 7.25);
    assert_eq!(inputs.filters.len(), 1);
}

#[test]
fn only_the_lights_that_are_on_reach_the_renderer() {
    let mut document = ProjectDocument::default();
    add_light(&mut document.lighting, LightKind::Point, "A".into());
    let second = add_light(&mut document.lighting, LightKind::Point, "B".into());
    document.lighting.lights[second].enabled = false;
    let inputs = RenderInputs::from_document(&document);
    assert_eq!(inputs.lights.len(), active_lights(&document.lighting).len());
    assert_eq!(inputs.lights.len(), 1);
}

fn frame(index: u32, time: f64, channels: Vec<(String, f64)>) -> FrameRequest {
    FrameRequest {
        index,
        time,
        pose: CameraPose::from_euler([0.0, -14.0, 2.0], 0.0, 0.0, 0.0, 70.0),
        channels,
        width: 48,
        height: 27,
    }
}

fn source() -> PhotoFrameSource {
    let world = Arc::new(World::new(scene(
        vec![cube([-2.0, -2.0, 0.0], [2.0, 2.0, 3.0], 0)],
        vec![paint(0.8, 0.8, 0.8)],
    )));
    PhotoFrameSource::new(world, RenderInputs::default(), RayTracingQuality::Low)
}

fn brightness(image: &image::RgbaImage) -> u64 {
    image
        .pixels()
        .map(|p| u64::from(p[0]) + u64::from(p[1]) + u64::from(p[2]))
        .sum()
}

#[test]
fn movie_frames_follow_the_pose_and_the_animated_channels() {
    let mut source = source();
    let noon = source
        .render(&frame(0, 0.0, vec![("hora_del_dia".into(), 12.0)]))
        .expect("frame");
    let midnight = source
        .render(&frame(1, 1.0, vec![("hora_del_dia".into(), 0.5)]))
        .expect("frame");
    assert_eq!((noon.width(), noon.height()), (48, 27));
    assert!(brightness(&noon) > brightness(&midnight) * 2);
    let cloudy = source.environment_at(&frame(2, 0.0, vec![("nubosidad".into(), 1.0)]));
    assert_eq!(cloudy.sky.clouds.coverage, 1.0);
    let mut turned = frame(3, 0.0, Vec::new());
    turned.pose = CameraPose::from_euler([0.0, -14.0, 2.0], 180.0, 0.0, 0.0, 70.0);
    let away = source.render(&turned).expect("frame");
    assert_ne!(
        away,
        source.render(&frame(3, 0.0, Vec::new())).expect("frame")
    );
}

#[test]
fn a_movie_exports_through_the_sequence_writer_and_can_be_cancelled() {
    let animation = Animation {
        duration: 0.2,
        fps: 10,
        ..Animation::default()
    };
    let folder = std::env::temp_dir().join(format!("gr-movie-{}", std::process::id()));
    let settings = ExportSettings {
        folder: folder.clone(),
        prefix: "t".into(),
        width: 32,
        height: 18,
        encode_video: false,
    };
    let cancel = Arc::new(AtomicBool::new(false));
    let mut source = source();
    source.set_cancel(Arc::clone(&cancel));
    let mut seen = Vec::new();
    let frames = export_sequence(
        &animation,
        &mut source,
        &settings,
        &cancel,
        &mut |done, total| {
            seen.push((done, total));
        },
    )
    .expect("exports");
    assert_eq!(frames, animation.frame_count());
    assert_eq!(seen.last().copied(), Some((frames, frames)));
    assert!(folder.join("t_000001.png").is_file());
    cancel.store(true, Ordering::Relaxed);
    let again = export_sequence(&animation, &mut source, &settings, &cancel, &mut |_, _| {});
    assert_eq!(again.err(), Some(ExportError::Cancelled));
    let _ = std::fs::remove_dir_all(folder);
}

#[test]
fn a_cancelled_frame_stops_inside_the_renderer() {
    let mut source = source();
    let cancel = Arc::new(AtomicBool::new(true));
    source.set_cancel(cancel);
    assert_eq!(
        source.render(&frame(0, 0.0, Vec::new())).err().as_deref(),
        Some("cancelled")
    );
}
