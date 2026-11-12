//! Optional integration check on REAL exported models (for example from Revit). It imports every supported file
//! found in `tests/data/` (ignored by git) plus the files listed in the `GR_MODEL` environment variable
//! (separated by `;`), prints the import report and fails only if an import returns an error.
//!
//! Run: `cargo test --release --test real_files -- --nocapture`

use guanacaste_render_rust::features::import_export::logic::formats::{detect, Detected};
use guanacaste_render_rust::features::import_export::logic::report_text;
use guanacaste_render_rust::features::import_export::models::ImportOptions;
use guanacaste_render_rust::features::import_export::services::{job, progress::Progress};
use std::path::PathBuf;

fn candidates() -> Vec<PathBuf> {
    let mut files = Vec::new();
    if let Ok(entries) = std::fs::read_dir(
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("tests")
            .join("data"),
    ) {
        files.extend(entries.flatten().map(|entry| entry.path()));
    }
    if let Some(list) = std::env::var_os("GR_MODEL") {
        files.extend(std::env::split_paths(&list));
    }
    files
        .into_iter()
        .filter(|path| matches!(detect(path), Detected::Supported(_)))
        .collect()
}

#[test]
fn imports_every_real_file_without_errors() {
    let files = candidates();
    if files.is_empty() {
        eprintln!(
            "Sin archivos reales: coloca .ifc/.obj/.glb/.fbx en tests/data/ o define GR_MODEL."
        );
    }
    for path in files {
        let started = std::time::Instant::now();
        let outcome = job::run(&path, &ImportOptions::default(), &Progress::default())
            .unwrap_or_else(|error| panic!("{}: {error:?}", path.display()));
        println!(
            "== {}\n{}\nTiempo: {:?}",
            path.display(),
            report_text::full_text(&outcome.scene, &outcome.report),
            started.elapsed()
        );
        assert!(
            outcome.scene.element_count() > 0,
            "el archivo no produjo elementos"
        );
    }
}
