//! Imports a model (a Revit IFC for example) and renders several views of it to PNG files with the project's own
//! CPU renderer, printing the time of every stage. The model file is only read.
//!
//! Run: `cargo run --release --example ifc_shots -- model.ifc out_folder [width] [views] [--garden] [--quality=low|medium|high]`
//!
//! `views` is a comma-separated list of `front`, `corner`, `side`, `aerial`, `plan`, `interior` (default: all but
//! `plan` and `side`).

use guanacaste_render_rust::features::import_export::models::ImportOptions;
use guanacaste_render_rust::features::import_export::services::{job, progress::Progress};
use guanacaste_render_rust::features::projects::logic::convert;
use guanacaste_render_rust::features::projects::models::Project;
use guanacaste_render_rust::features::rendering::engine::{
    interior_camera, render_prepared, RenderControl, RenderScene, ViewPreset,
};
use guanacaste_render_rust::features::rendering::logic::garden;
use guanacaste_render_rust::features::rendering::logic::inputs::{CameraChoice, RenderInputs};
use guanacaste_render_rust::features::rendering::logic::world::build_world;
use guanacaste_render_rust::features::rendering::models::RayTracingQuality;
use std::time::Instant;

fn preset(name: &str) -> Option<ViewPreset> {
    Some(match name {
        "front" => ViewPreset::Front,
        "corner" => ViewPreset::Corner,
        "side" => ViewPreset::Side,
        "aerial" => ViewPreset::Aerial,
        "plan" => ViewPreset::Plan,
        _ => return None,
    })
}

fn main() {
    let mut positional = Vec::new();
    let (mut with_garden, mut quality) = (false, RayTracingQuality::Medium);
    for arg in std::env::args().skip(1) {
        match arg.as_str() {
            "--garden" => with_garden = true,
            "--quality=low" => quality = RayTracingQuality::Low,
            "--quality=medium" => quality = RayTracingQuality::Medium,
            "--quality=high" => quality = RayTracingQuality::High,
            _ => positional.push(arg),
        }
    }
    let [model, out, rest @ ..] = positional.as_slice() else {
        eprintln!("usage: ifc_shots model out_folder [width] [views] [--garden] [--quality=...]");
        std::process::exit(2);
    };
    let out = std::path::PathBuf::from(out);
    let width: u32 = rest.first().and_then(|w| w.parse().ok()).unwrap_or(1280);
    let views: Vec<String> = rest
        .get(1)
        .map_or("front,corner,aerial,interior", String::as_str)
        .split(',')
        .map(str::to_owned)
        .collect();
    std::fs::create_dir_all(&out).expect("output folder");
    let started = Instant::now();
    let outcome = job::run(
        std::path::Path::new(model),
        &ImportOptions::default(),
        &Progress::default(),
    )
    .expect("import");
    println!(
        "import {:.1?}: {} triangles, {} materials",
        started.elapsed(),
        outcome.scene.triangle_count(),
        outcome.scene.materials.len()
    );
    let (document, meshes) = convert::from_scene(&outcome.scene, "Casa", "model", 0);
    let mut project = Project {
        id: [7; 16],
        document,
        meshes,
        path: None,
        legacy_source: None,
        read_only: false,
        dirty: false,
    };
    if with_garden {
        println!("garden: {} plants", garden::plant_around(&mut project));
    }
    let t = Instant::now();
    let world = build_world(&project);
    println!(
        "world {:.1?}: {} triangles, focus {:?}, ground {:.2}, facade {:.0} deg",
        t.elapsed(),
        world.scene.triangle_count(),
        world.focus,
        world.ground_z,
        world.facade.to_degrees()
    );
    let t = Instant::now();
    let scene = RenderScene::new(&world);
    println!(
        "bvh {:.1?}: {} traced triangles",
        t.elapsed(),
        scene.triangle_count()
    );
    let (w, h) = (width, width * 9 / 16);
    for name in &views {
        let mut inputs = RenderInputs::from_document(&project.document);
        inputs.environment.hour = 15.0;
        let camera = if name == "interior" {
            world
                .focus
                .and_then(|focus| interior_camera(&scene, &focus, w, h))
        } else {
            let view = preset(name).expect("unknown view");
            inputs
                .camera(&world, &CameraChoice::Preset, view, (w, h))
                .map(|(camera, _)| camera)
        };
        let Some(camera) = camera else {
            println!("{name}: no camera");
            continue;
        };
        println!(
            "{name} camera at {:?} looking {:?}",
            camera.position, camera.direction
        );
        let mut request = inputs.request(camera, None, false, quality);
        request.post.auto_exposure = name == "interior";
        let t = Instant::now();
        let image =
            render_prepared(&scene, &request, &RenderControl::new(), &|_| {}).expect("render");
        let file = out.join(format!("{name}.png"));
        image.save(&file).expect("save png");
        println!("{name}: {:.1?} -> {}", t.elapsed(), file.display());
    }
}
