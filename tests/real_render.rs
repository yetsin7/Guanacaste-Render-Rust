//! Optional check on a REAL model: imports the file named by `GR_RENDER_MODEL` (for example the Casa del Cobogo
//! `.g3d`), builds the world of a project around it and renders PNG pictures into `GR_RENDER_OUT` (default: the
//! system temp folder). It prints timings and does nothing when the variable is not set.
//!
//! Run: `GR_RENDER_MODEL=casa.g3d GR_RENDER_OUT=out cargo test --release --test real_render -- --nocapture`

use guanacaste_render_rust::features::camera::models::filter::FilterKind;
use guanacaste_render_rust::features::camera::models::stack::FilterStack;
use guanacaste_render_rust::features::import_export::models::ImportOptions;
use guanacaste_render_rust::features::import_export::services::{job, progress::Progress};
use guanacaste_render_rust::features::projects::logic::convert;
use guanacaste_render_rust::features::projects::models::Project;
use guanacaste_render_rust::features::rendering::engine::{
    render_prepared, RenderControl, RenderScene, ViewPreset,
};
use guanacaste_render_rust::features::rendering::logic::inputs::{CameraChoice, RenderInputs};
use guanacaste_render_rust::features::rendering::logic::world::build_world;
use guanacaste_render_rust::features::rendering::models::RayTracingQuality;
use std::path::PathBuf;
use std::time::Instant;

fn project(path: &PathBuf) -> Project {
    let outcome = job::run(path, &ImportOptions::default(), &Progress::default())
        .unwrap_or_else(|error| panic!("{}: {error:?}", path.display()));
    let (document, meshes) = convert::from_scene(&outcome.scene, "Casa", "model", 0);
    Project {
        id: [7; 16],
        document,
        meshes,
        path: None,
        legacy_source: None,
        read_only: false,
        dirty: false,
    }
}

fn render(
    scene: &RenderScene,
    inputs: &RenderInputs,
    world: &guanacaste_render_rust::features::rendering::engine::World,
    preset: ViewPreset,
    size: (u32, u32),
    quality: RayTracingQuality,
    name: &str,
    out: &PathBuf,
) {
    let (camera, photo) = inputs
        .camera(world, &CameraChoice::Preset, preset, size)
        .expect("camera");
    let request = inputs.request(camera, photo, false, quality);
    let started = Instant::now();
    let image = render_prepared(scene, &request, &RenderControl::new(), &|_| {}).expect("render");
    let file = out.join(format!("{name}.png"));
    image.save(&file).expect("save");
    println!(
        "{name}: {}x{} {:?} in {:.1?} -> {}",
        size.0,
        size.1,
        quality,
        started.elapsed(),
        file.display()
    );
}

#[test]
fn renders_the_real_model() {
    let Some(model) = std::env::var_os("GR_RENDER_MODEL").map(PathBuf::from) else {
        eprintln!("GR_RENDER_MODEL no definido: se omite el render de un modelo real.");
        return;
    };
    let out = std::env::var_os("GR_RENDER_OUT").map_or_else(std::env::temp_dir, PathBuf::from);
    std::fs::create_dir_all(&out).expect("output folder");
    let started = Instant::now();
    let project = project(&model);
    println!("import: {:.1?}", started.elapsed());
    let started = Instant::now();
    let world = build_world(&project);
    println!(
        "world: {:.1?}, {} triangles",
        started.elapsed(),
        world.scene.triangle_count()
    );
    let started = Instant::now();
    let scene = RenderScene::new(&world);
    println!("bvh: {:.1?}", started.elapsed());
    let mut inputs = RenderInputs::from_document(&project.document);
    let sizes = std::env::var("GR_RENDER_W")
        .ok()
        .and_then(|w| w.parse::<u32>().ok())
        .unwrap_or(960);
    let size = (sizes, sizes * 9 / 16);
    render(&scene, &inputs, &world, ViewPreset::Corner, size, RayTracingQuality::Low, "corner_low", &out);
    render(&scene, &inputs, &world, ViewPreset::Corner, size, RayTracingQuality::Medium, "corner_medium", &out);
    render(&scene, &inputs, &world, ViewPreset::Front, size, RayTracingQuality::Medium, "front_medium", &out);
    inputs.denoise = false;
    render(&scene, &inputs, &world, ViewPreset::Corner, size, RayTracingQuality::Low, "corner_low_raw", &out);
    inputs.denoise = true;
    let mut stack = FilterStack::new();
    stack.push(FilterKind::Vignette);
    stack.push(FilterKind::Saturation);
    inputs.filters = stack;
    render(&scene, &inputs, &world, ViewPreset::Corner, size, RayTracingQuality::Medium, "corner_filters", &out);
    inputs.filters = FilterStack::new();
    inputs.environment.hour = 18.2;
    render(&scene, &inputs, &world, ViewPreset::Corner, size, RayTracingQuality::Medium, "corner_sunset", &out);
}
