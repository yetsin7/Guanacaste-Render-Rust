//! Renders a few views of a Guanacaste 3D project (for example the Casa del Cobogó) to PNG files and prints timings.
//!
//! Run: `cargo run --release --example legacy_shots -- casa.g3d out_folder [width]`

use guanacaste_render_rust::features::import_export::models::ImportOptions;
use guanacaste_render_rust::features::import_export::services::{job, progress::Progress};
use guanacaste_render_rust::features::projects::logic::convert;
use guanacaste_render_rust::features::projects::models::{CameraDoc, Project};
use guanacaste_render_rust::features::rendering::engine::{
    render_prepared, RenderControl, RenderScene, ViewPreset,
};
use guanacaste_render_rust::features::rendering::logic::inputs::{CameraChoice, RenderInputs};
use guanacaste_render_rust::features::rendering::logic::world::build_world;
use guanacaste_render_rust::features::rendering::models::RayTracingQuality;
use std::time::Instant;

/// (name, camera position, target, focal length mm, hour)
#[allow(clippy::type_complexity)]
const VIEWS: [(&str, [f64; 3], [f64; 3], f32, f32); 4] = [
    ("street", [7.6, -20.5, 1.6], [7.6, 0.0, 3.0], 24.0, 14.75),
    ("carport", [11.0, -7.0, 1.5], [11.0, 3.0, 1.3], 28.0, 14.0),
    ("cobogo", [14.0, -6.0, 5.0], [14.0, 0.0, 4.2], 35.0, 14.0),
    ("aerial", [-8.0, -22.0, 14.0], [8.0, 4.0, 2.0], 28.0, 15.0),
];

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let (model, out) = (&args[1], std::path::PathBuf::from(&args[2]));
    let width: u32 = args.get(3).and_then(|w| w.parse().ok()).unwrap_or(960);
    let quality = match args.get(4).map(String::as_str) {
        Some("low") => RayTracingQuality::Low,
        Some("high") => RayTracingQuality::High,
        Some("ultra") => RayTracingQuality::Ultra,
        _ => RayTracingQuality::Medium,
    };
    std::fs::create_dir_all(&out).unwrap();
    let started = Instant::now();
    let outcome = job::run(
        std::path::Path::new(model),
        &ImportOptions::default(),
        &Progress::default(),
    )
    .expect("import");
    println!(
        "import {:.1?}: {} triangles, {} materials, {} elements",
        started.elapsed(),
        outcome.scene.triangle_count(),
        outcome.scene.materials.len(),
        outcome.scene.element_count()
    );
    let (document, meshes) = convert::from_scene(&outcome.scene, "Casa", "model", 0);
    let project = Project {
        id: [7; 16],
        document,
        meshes,
        path: None,
        legacy_source: None,
        read_only: false,
        dirty: false,
    };
    let world = build_world(&project);
    let scene = RenderScene::new(&world);
    println!(
        "world+bvh {:.1?}, {} triangles",
        started.elapsed(),
        world.scene.triangle_count()
    );
    for (name, position, target, focal, hour) in VIEWS {
        let mut inputs = RenderInputs::from_document(&project.document);
        inputs.environment.hour = hour;
        inputs.cameras.push(CameraDoc {
            name: name.into(),
            position,
            target,
            focal_mm: focal,
            aperture: 8.0,
            filters: None,
            thumbnail: None,
        });
        let (camera, photo) = inputs
            .camera(
                &world,
                &CameraChoice::Saved(0),
                ViewPreset::Corner,
                (width, width * 9 / 16),
            )
            .expect("camera");
        let request = inputs.request(camera, photo, false, quality);
        let t = Instant::now();
        let image =
            render_prepared(&scene, &request, &RenderControl::new(), &|_| {}).expect("render");
        let file = out.join(format!("{name}.png"));
        image.save(&file).unwrap();
        println!("{name}: {:.1?} -> {}", t.elapsed(), file.display());
    }
}
