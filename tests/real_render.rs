//! Optional check on a REAL model: imports the file named by `GR_RENDER_MODEL` (for example the Casa del Cobogo
//! `.g3d`), builds the world of a project around it and renders PNG pictures into `GR_RENDER_OUT` (default: the
//! system temp folder). It prints timings and does nothing when the variable is not set.
//!
//! Run: `GR_RENDER_MODEL=casa.g3d GR_RENDER_OUT=out cargo test --release --test real_render -- --nocapture`

use guanacaste_render_rust::features::camera::models::filter::FilterKind;
use guanacaste_render_rust::features::camera::models::stack::FilterStack;
use guanacaste_render_rust::features::import_export::models::ImportOptions;
use guanacaste_render_rust::features::import_export::services::{job, progress::Progress};
use guanacaste_render_rust::features::lighting::models::{Light, Power};
use guanacaste_render_rust::features::projects::logic::convert;
use guanacaste_render_rust::features::projects::models::{CameraDoc, Project};
use guanacaste_render_rust::features::rendering::engine::{
    render_prepared, RenderControl, RenderScene, ViewPreset, World,
};
use guanacaste_render_rust::features::rendering::logic::inputs::{CameraChoice, RenderInputs};
use guanacaste_render_rust::features::rendering::logic::world::build_world;
use guanacaste_render_rust::features::rendering::models::RayTracingQuality;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Instant;

fn project(path: &Path) -> Project {
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

/// Everything shared by the pictures of one run.
struct Studio {
    world: World,
    scene: RenderScene,
    out: PathBuf,
    size: (u32, u32),
}

impl Studio {
    fn shot(
        &self,
        inputs: &RenderInputs,
        choice: &CameraChoice,
        dof: bool,
        quality: RayTracingQuality,
        name: &str,
    ) {
        let (camera, photo) = inputs
            .camera(&self.world, choice, ViewPreset::Corner, self.size)
            .expect("camera");
        let request = inputs.request(camera, photo, dof, quality);
        let started = Instant::now();
        let image =
            render_prepared(&self.scene, &request, &RenderControl::new(), &|_| {}).expect("render");
        let file = self.out.join(format!("{name}.png"));
        image.save(&file).expect("save");
        println!(
            "{name}: {}x{} {quality:?} in {:.1?} -> {}",
            self.size.0,
            self.size.1,
            started.elapsed(),
            file.display()
        );
    }
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
    let width = std::env::var("GR_RENDER_W")
        .ok()
        .and_then(|w| w.parse::<u32>().ok())
        .unwrap_or(960);
    let focus = world.focus.expect("model bounds");
    let studio = Studio {
        world,
        scene,
        out,
        size: (width, width * 9 / 16),
    };
    let preset = CameraChoice::Preset;
    let mut inputs = RenderInputs::from_document(&project.document);
    let (low, medium) = (RayTracingQuality::Low, RayTracingQuality::Medium);
    studio.shot(&inputs, &preset, false, low, "corner_low");
    studio.shot(&inputs, &preset, false, medium, "corner_medium");
    inputs.denoise = false;
    studio.shot(&inputs, &preset, false, low, "corner_low_raw");
    inputs.denoise = true;
    // Filters of the camera domain.
    let mut stack = FilterStack::new();
    stack.push(FilterKind::Vignette);
    stack.push(FilterKind::Sepia);
    inputs.filters = stack;
    studio.shot(&inputs, &preset, false, medium, "corner_filters");
    inputs.filters = FilterStack::new();
    // A saved camera at eye height with depth of field.
    let [cx, cy] = [(focus.min[0] + focus.max[0]) / 2.0, focus.min[1] - 30.0];
    inputs.cameras.push(CameraDoc {
        name: "Acceso".into(),
        position: [f64::from(cx), f64::from(cy), 1.7],
        target: [f64::from(cx), f64::from(focus.min[1]), 2.0],
        focal_mm: 35.0,
        aperture: 2.8,
        filters: None,
        thumbnail: None,
    });
    studio.shot(&inputs, &CameraChoice::Saved(0), true, medium, "saved_dof");
    // Late afternoon, and fog with rain.
    inputs.environment.hour = 17.2;
    studio.shot(&inputs, &preset, false, medium, "corner_afternoon");
    inputs.environment.hour = 10.0;
    inputs.environment.weather.fog_density = 0.35;
    studio.shot(&inputs, &preset, false, medium, "corner_fog");
    inputs.environment.weather.fog_density = 0.0;
    // Night with lamps around the house (the lighting world is Y up: `(x, y, z)` is scene `(x, -z, y)`).
    inputs.environment.hour = 20.5;
    let lamp = |x: f32, y: f32| Light {
        position: [x, 3.0, -y],
        power: Power::Lumens { value: 60_000.0 },
        kelvin: 3000.0,
        ..Light::default()
    };
    inputs.lights = Arc::new(vec![
        lamp(focus.min[0], focus.min[1]),
        lamp(focus.max[0], focus.min[1]),
        lamp(focus.max[0], focus.max[1]),
    ]);
    studio.shot(&inputs, &preset, false, medium, "corner_night_lamps");
}
