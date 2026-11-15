//! Measures the real frame time of the 3D view with a model loaded, without opening a window: the model is
//! imported (read only), composed exactly like the application does (project geometry plus the landscape), and
//! drawn offscreen with the viewport's own pipelines and frustum culling from several cameras.
//!
//! Run: `cargo run --release --example viewport_bench -- "C:/ruta/modelo.ifc" [width] [height] [frames] [--no-landscape] [--shots=carpeta]`
//! (with `--shots` it saves one PNG per camera instead of timing, to check the look of the sky and horizon).

use guanacaste_render_rust::commands::scene::SceneComposer;
use guanacaste_render_rust::features::build::models::ObjectsState;
use guanacaste_render_rust::features::import_export::models::ImportOptions;
use guanacaste_render_rust::features::import_export::services::{job, progress::Progress};
use guanacaste_render_rust::features::landscape::models::LandscapeState;
use guanacaste_render_rust::features::projects::logic::convert;
use guanacaste_render_rust::features::projects::models::Project;
use guanacaste_render_rust::features::projects::services::util;
use guanacaste_render_rust::features::viewport::models::{Camera, ViewPreset};
use guanacaste_render_rust::features::viewport::services::bench;
use guanacaste_render_rust::features::viewport::ui::ViewportState;
use std::path::PathBuf;
use std::time::Instant;

fn main() {
    let mut numbers = Vec::new();
    let mut path = None;
    let mut landscape_on = true;
    let mut shots: Option<PathBuf> = None;
    for arg in std::env::args().skip(1) {
        if arg == "--no-landscape" {
            landscape_on = false;
        } else if let Some(folder) = arg.strip_prefix("--shots=") {
            shots = Some(PathBuf::from(folder));
        } else if let Ok(n) = arg.parse::<u32>() {
            numbers.push(n);
        } else {
            path = Some(PathBuf::from(arg));
        }
    }
    let Some(path) = path.or_else(|| std::env::var_os("GR_MODEL").map(PathBuf::from)) else {
        eprintln!("Uso: viewport_bench <archivo> [ancho] [alto] [cuadros] [--no-landscape]");
        std::process::exit(2);
    };
    let size = [
        numbers.first().copied().unwrap_or(1600),
        numbers.get(1).copied().unwrap_or(900),
    ];
    let frames = numbers.get(2).copied().unwrap_or(120) as usize;

    let started = Instant::now();
    let outcome = job::run(&path, &ImportOptions::default(), &Progress::default())
        .unwrap_or_else(|error| panic!("importación: {error:?}"));
    println!("Importación: {:.2} s", started.elapsed().as_secs_f64());
    let (document, meshes) =
        convert::from_scene(&outcome.scene, "bench", "bench", util::now_unix());
    let project = Project {
        id: [9; 16],
        document,
        meshes,
        path: None,
        legacy_source: None,
        read_only: false,
        dirty: false,
    };
    let mut landscape = LandscapeState::default();
    landscape.landscape.visible = landscape_on;
    let mut objects = ObjectsState::default();
    let mut composer = SceneComposer::default();
    let mut view = ViewportState::new();
    let built = Instant::now();
    view.sync(composer.compose(Some(&project), &landscape, &mut objects));
    println!(
        "Escena del visor: {:.2} s · {} triángulos dibujables (paisaje: {})",
        built.elapsed().as_secs_f64(),
        view.stats().triangles,
        if landscape_on { "sí" } else { "no" }
    );

    let aspect = size[0] as f32 / size[1] as f32;
    let bounds = *view.bounds().expect("el modelo tiene geometría");
    let lens = Camera::default();
    let mut cameras = Vec::new();
    for (name, preset) in [
        ("exterior isométrica", ViewPreset::Isometric),
        ("planta (cenital)", ViewPreset::Top),
        ("frente", ViewPreset::Front),
    ] {
        let mut controller = view.controller.clone();
        controller.frame_preset(Some(&bounds), preset, aspect, false);
        cameras.push((name.to_string(), controller.camera));
    }
    let center = [
        (bounds.min[0] + bounds.max[0]) * 0.5,
        (bounds.min[1] + bounds.max[1]) * 0.5,
        bounds.min[2] + 1.6,
    ];
    for (k, yaw) in [0.3_f32, 2.1, 4.0].iter().enumerate() {
        let direction = Camera::direction(*yaw, 0.0);
        let target = [
            center[0] + direction[0],
            center[1] + direction[1],
            center[2],
        ];
        let mut camera = Camera::looking_at(center, target);
        camera.near = lens.near;
        camera.far = 400.0;
        cameras.push((format!("interior a 1,6 m, rumbo {}", k + 1), camera));
    }

    // Eye level outside the house, looking at the horizon: where a seam between ground and sky would show.
    let reach = bounds.max[0] - bounds.min[0];
    let mut outside = Camera::looking_at(
        [center[0], bounds.min[1] - reach * 0.6, bounds.min[2] + 1.6],
        [center[0], center[1], bounds.min[2] + 1.6],
    );
    outside.far = 2000.0;
    cameras.push(("exterior a la altura de los ojos".into(), outside));
    let mut level = outside;
    level.target[2] += 6.0;
    cameras.push(("exterior mirando al cielo".into(), level));

    if let Some(folder) = shots {
        std::fs::create_dir_all(&folder).expect("carpeta de capturas");
        let list: Vec<Camera> = cameras.iter().map(|(_, c)| *c).collect();
        let pictures = bench::capture(&view.upload(), &list, size, view.scene_radius())
            .expect("captura sin GPU");
        for (k, picture) in pictures.iter().enumerate() {
            let file = folder.join(format!("visor-{k}.png"));
            image::save_buffer(&file, picture, size[0], size[1], image::ColorType::Rgba8)
                .expect("guardar PNG");
            println!("Captura: {}", file.display());
        }
        return;
    }

    let Some((adapter, timings)) =
        bench::run(&view.upload(), &cameras, size, frames, view.scene_radius())
    else {
        eprintln!("No hay GPU disponible.");
        std::process::exit(1);
    };
    println!(
        "GPU: {adapter} · {}x{} px · {frames} cuadros por cámara",
        size[0], size[1]
    );
    for t in &timings {
        println!(
            "{:<28} {:>7.1} FPS · {:>6.2} ms (p95 {:.2}, peor {:.2}) · bloques {}/{} · {} triángulos",
            t.name,
            t.fps(),
            t.ms.0,
            t.ms.1,
            t.ms.2,
            t.chunks.0,
            t.chunks.1,
            t.triangles
        );
    }
}
