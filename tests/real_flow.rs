//! The whole path a user walks, without a window: import a model, show it in the 3D view, fly around with
//! `W`/`A`/`S`/`D`/`Q`/`E`, save a camera, save the project, open it again and find everything as it was.
//!
//! It always runs on a small generated model. With `GR_MODEL` set (for example the Revit IFC of a house) it also
//! runs on that file, read only, and prints how long the per-frame CPU work of the view takes with it.
//!
//! Run: `GR_MODEL="C:/ruta/casa.ifc" cargo test --release --test real_flow -- --nocapture`

use guanacaste_render_rust::commands::camera_link;
use guanacaste_render_rust::commands::sync;
use guanacaste_render_rust::features::import_export::models::{ImportOptions, ImportOutcome};
use guanacaste_render_rust::features::import_export::services::{job, obj, progress::Progress};
use guanacaste_render_rust::features::viewport::logic::frustum::may_be_visible;
use guanacaste_render_rust::features::viewport::models::{MouseButtons, MoveKeys, NavInput};
use guanacaste_render_rust::features::viewport::ui::SceneChange;
use guanacaste_render_rust::models::AppState;
use std::path::{Path, PathBuf};
use std::time::Instant;

const CUBES_OBJ: &str =
    "v -4 -4 0\nv 4 -4 0\nv 4 4 0\nv -4 4 0\nv -4 -4 3\nv 4 -4 3\nv 4 4 3\nv -4 4 3\n\
f 1 2 3 4\nf 5 6 7 8\nf 1 2 6 5\nf 2 3 7 6\nf 3 4 8 7\nf 4 1 5 8\n";

fn generated() -> (ImportOutcome, PathBuf) {
    let outcome = obj::import_reader(
        CUBES_OBJ.as_bytes(),
        CUBES_OBJ.len() as u64,
        &ImportOptions::default(),
        &Progress::default(),
        &|_| None,
    )
    .expect("import of the generated model");
    (outcome, PathBuf::from("Casa generada.obj"))
}

fn folder(label: &str) -> PathBuf {
    let path = std::env::temp_dir().join(format!("gr-flow-{label}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&path);
    std::fs::create_dir_all(&path).expect("temporary folder");
    path
}

fn frame(app: &mut AppState) -> SceneChange {
    sync::step(app);
    let source = app.composer.compose(
        app.workspace.project.as_ref(),
        &app.landscape,
        &mut app.objects,
    );
    app.viewport.sync(source)
}

/// Holds `keys` for `frames` frames of 1/60 s, like a user holding the keys over the 3D view.
fn fly(app: &mut AppState, keys: MoveKeys, frames: usize) {
    let input = NavInput {
        dt: 1.0 / 60.0,
        view_size: [1280.0, 720.0],
        keys,
        buttons: MouseButtons::default(),
        ..Default::default()
    };
    for _ in 0..frames {
        app.viewport.controller.update(&input);
    }
}

fn walk_the_whole_flow(outcome: &ImportOutcome, source: &Path, label: &str) {
    let dir = folder(label);
    let mut app = AppState::with_data_dir(dir.join("datos"));
    app.viewport.gpu_available = true;

    // Import: the model becomes an unsaved project and the view loads it.
    app.workspace.adopt_import(outcome, source);
    assert!(app.workspace.dirty(), "a fresh import is unsaved");
    assert_eq!(frame(&mut app), SceneChange::Loaded);
    assert!(app.viewport.has_geometry());
    let triangles = app.viewport.stats().triangles;
    assert!(triangles > 0);
    assert_eq!(
        app.viewport.stats().skipped_meshes,
        0,
        "{label}: every mesh is drawable"
    );
    let bounds = *app.viewport.bounds().expect("bounds of the model");
    assert!(bounds.min.iter().chain(&bounds.max).all(|v| v.is_finite()));

    // The first view frames the whole model and sees it.
    app.viewport.frame_scene(16.0 / 9.0);
    let camera = *app.viewport.camera();
    assert!(camera.is_valid(), "{label}: the framed camera is valid");
    let looked_at = camera.view_projection(16.0 / 9.0);
    assert!(
        may_be_visible(&looked_at, &bounds),
        "{label}: model in view"
    );

    // Navigate: every movement key does what it says, relative to where the camera looks.
    app.viewport.controller.go_to(camera, false);
    let start = app.viewport.camera().position;
    let forward = app.viewport.camera().forward();
    let right = app.viewport.camera().right();
    let along = |from: [f32; 3], to: [f32; 3], axis: [f32; 3]| {
        (0..3).map(|i| (to[i] - from[i]) * axis[i]).sum::<f32>()
    };
    let keys = |f: fn(&mut MoveKeys)| {
        let mut k = MoveKeys::default();
        f(&mut k);
        k
    };
    fly(&mut app, keys(|k| k.forward = true), 30);
    let after_w = app.viewport.camera().position;
    assert!(along(start, after_w, forward) > 0.0, "W moves forward");
    fly(&mut app, keys(|k| k.back = true), 60);
    let after_s = app.viewport.camera().position;
    assert!(along(after_w, after_s, forward) < 0.0, "S moves backward");
    fly(&mut app, keys(|k| k.right = true), 30);
    let after_d = app.viewport.camera().position;
    assert!(along(after_s, after_d, right) > 0.0, "D moves right");
    fly(&mut app, keys(|k| k.left = true), 60);
    let after_a = app.viewport.camera().position;
    assert!(along(after_d, after_a, right) < 0.0, "A moves left");
    fly(&mut app, keys(|k| k.up = true), 30);
    let after_e = app.viewport.camera().position;
    assert!(after_e[2] > after_a[2], "E goes up");
    fly(&mut app, keys(|k| k.down = true), 60);
    assert!(
        app.viewport.camera().position[2] < after_e[2],
        "Q goes down"
    );
    assert!(app.viewport.camera().is_valid(), "{label}: still valid");

    // Save a camera and the project, then open the file in a fresh application.
    camera_link::save_current(&mut app);
    // The application brings the states and the document in step once per frame, before any save shortcut runs.
    frame(&mut app);
    assert!(app.workspace.dirty());
    let file = dir.join("Casa.gr");
    let saving = Instant::now();
    assert!(app.workspace.save_to(&file, true), "{label}: save");
    println!(
        "{label}: guardar con miniatura {:.2} s",
        saving.elapsed().as_secs_f64()
    );
    assert!(!app.workspace.dirty());
    let saved = app.workspace.project.clone().expect("project");

    let mut other = AppState::with_data_dir(dir.join("otros"));
    other.viewport.gpu_available = true;
    assert!(other.workspace.open_file(&file), "{label}: reopen");
    let reopened = other.workspace.project.clone().expect("reopened project");
    assert_eq!(reopened.meshes, saved.meshes, "{label}: meshes identical");
    assert_eq!(reopened.document, saved.document, "{label}: document equal");
    assert!(!other.workspace.dirty(), "an opened file has no changes");
    assert_eq!(frame(&mut other), SceneChange::Loaded);
    assert_eq!(other.viewport.stats().triangles, triangles);
    assert_eq!(other.cameras.len(), 1, "{label}: the saved camera is back");
    assert!(other.viewport.camera().is_valid());
    // And it is shown again: nothing to rebuild on the next frames.
    assert_eq!(frame(&mut other), SceneChange::Unchanged);

    // The per-frame CPU work of the view must stay small even for a big model.
    let started = Instant::now();
    for _ in 0..100 {
        assert_eq!(frame(&mut other), SceneChange::Unchanged);
    }
    let per_frame = started.elapsed().as_secs_f64() * 10.0;
    println!(
        "{label}: {triangles} triángulos · trabajo de CPU por cuadro del visor: {per_frame:.3} ms"
    );
    assert!(per_frame < 8.0, "{label}: {per_frame} ms of CPU per frame");
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn import_navigate_save_and_reopen_a_generated_model() {
    let (outcome, source) = generated();
    walk_the_whole_flow(&outcome, &source, "generado");
}

#[test]
fn import_navigate_save_and_reopen_the_real_model() {
    let Some(list) = std::env::var_os("GR_MODEL") else {
        eprintln!("Sin GR_MODEL: se omite el modelo real.");
        return;
    };
    for path in std::env::split_paths(&list) {
        let started = Instant::now();
        let outcome = job::run(&path, &ImportOptions::default(), &Progress::default())
            .unwrap_or_else(|error| panic!("{}: {error:?}", path.display()));
        println!(
            "{}: importado en {:.1} s",
            path.display(),
            started.elapsed().as_secs_f64()
        );
        walk_the_whole_flow(&outcome, &path, "real");
    }
}
