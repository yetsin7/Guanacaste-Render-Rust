//! Imports a model (argument or `GR_MODEL`), saves it as a Guanacaste Render `.gr` project in a temporary folder,
//! opens it again, checks that nothing changed and prints sizes and times. The temporary project is deleted at
//! the end, so nothing derived from the model stays on disk. Example:
//! `cargo run --release --example project_roundtrip -- "C:/ruta/modelo.ifc"`

use guanacaste_render_rust::features::import_export::models::ImportOptions;
use guanacaste_render_rust::features::import_export::services::{job, progress::Progress};
use guanacaste_render_rust::features::projects::logic::convert;
use guanacaste_render_rust::features::projects::models::Project;
use guanacaste_render_rust::features::projects::services::container::Limits;
use guanacaste_render_rust::features::projects::services::{store, util};
use std::path::PathBuf;
use std::time::Instant;

fn main() {
    let Some(path) = std::env::args()
        .nth(1)
        .map(PathBuf::from)
        .or_else(|| std::env::var_os("GR_MODEL").map(PathBuf::from))
    else {
        eprintln!("Uso: project_roundtrip <archivo>");
        std::process::exit(2);
    };
    let started = Instant::now();
    let outcome = job::run(&path, &ImportOptions::default(), &Progress::default())
        .unwrap_or_else(|error| panic!("importación: {error:?}"));
    let import_time = started.elapsed();
    let file_name = path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("modelo");
    let (document, meshes) =
        convert::from_scene(&outcome.scene, file_name, file_name, util::now_unix());
    let project = Project {
        id: util::new_document_id(),
        document,
        meshes,
        path: None,
        legacy_source: None,
        read_only: false,
        dirty: true,
    };
    let folder = std::env::temp_dir().join(format!(
        "gr-roundtrip-{}",
        util::hex(&util::new_document_id())
    ));
    std::fs::create_dir_all(&folder).expect("carpeta temporal");
    let target = folder.join("prueba.gr");
    let stats = store::save(&target, &project, None).expect("guardar");
    let opened_at = Instant::now();
    let loaded = store::load(&target, Limits::default()).expect("abrir");
    let open_time = opened_at.elapsed();
    let same =
        loaded.project.document == project.document && loaded.project.meshes == project.meshes;
    let source_size = std::fs::metadata(&path).map_or(0, |m| m.len());
    let _ = std::fs::remove_dir_all(&folder);
    println!("Origen: {:.1} MB", source_size as f64 / 1_048_576.0);
    println!("Importación: {:.2} s", import_time.as_secs_f64());
    println!(
        "Elementos: {} · mallas: {} · triángulos: {} · vértices: {}",
        outcome.scene.element_count(),
        project.meshes.len(),
        outcome.scene.triangle_count(),
        outcome.scene.vertex_count()
    );
    println!(
        "Proyecto .gr: {:.1} MB · guardado (con verificación) {} ms",
        stats.bytes as f64 / 1_048_576.0,
        stats.millis
    );
    println!("Apertura: {} ms", open_time.as_millis());
    println!("Idéntico tras reabrir: {}", if same { "sí" } else { "NO" });
    if !same {
        std::process::exit(1);
    }
}
