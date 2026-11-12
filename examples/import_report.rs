//! Imports a model from the path given as argument (or `GR_MODEL`), prints the import report and, with a second
//! argument, writes a plan + isometric preview PNG. Example:
//! `cargo run --release --example import_report -- "C:/ruta/modelo.ifc" preview.png`

use guanacaste_render_rust::features::import_export::logic::report_text;
use guanacaste_render_rust::features::import_export::models::ImportOptions;
use guanacaste_render_rust::features::import_export::services::{
    job, preview_image, progress::Progress,
};
use std::path::PathBuf;

fn main() {
    let mut args = std::env::args().skip(1);
    let Some(path) = args
        .next()
        .map(PathBuf::from)
        .or_else(|| std::env::var_os("GR_MODEL").map(PathBuf::from))
    else {
        eprintln!("Uso: import_report <archivo> [vista.png]");
        std::process::exit(2);
    };
    let started = std::time::Instant::now();
    match job::run(&path, &ImportOptions::default(), &Progress::default()) {
        Ok(outcome) => {
            println!(
                "{}",
                report_text::full_text(&outcome.scene, &outcome.report)
            );
            println!("Tiempo total: {:.1} s", started.elapsed().as_secs_f32());
            if let Some(out) = args.next() {
                let image = preview_image::render(&outcome.scene, 900, 700);
                match image.save(&out) {
                    Ok(()) => println!("Vista previa escrita en {out}"),
                    Err(error) => eprintln!("No se pudo escribir la vista previa: {error}"),
                }
            }
        }
        Err(error) => {
            eprintln!(
                "Error de importación: {} {}",
                guanacaste_render_rust::services::localization::tr(error.key()),
                error.detail()
            );
            std::process::exit(1);
        }
    }
}
