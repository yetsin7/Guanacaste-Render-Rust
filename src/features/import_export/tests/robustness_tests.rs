//! Errors must never crash the application or stay silent.

use super::fixtures::IFC2X3_MM;
use crate::features::import_export::models::{ImportError, ImportOptions};
use crate::features::import_export::services::{ifc, job, progress::Progress};
use std::path::Path;

#[test]
fn garbage_is_an_error_not_a_panic() {
    let result = ifc::import_reader(
        &b"esto no es IFC"[..],
        14,
        &ImportOptions::default(),
        &Progress::default(),
    );
    assert!(matches!(result, Err(ImportError::Invalid(_))));
}

#[test]
fn truncated_files_do_not_panic() {
    let cut = &IFC2X3_MM[..IFC2X3_MM.len() * 2 / 3];
    let _ = ifc::import_reader(
        cut.as_bytes(),
        cut.len() as u64,
        &ImportOptions::default(),
        &Progress::default(),
    );
}

#[test]
fn cancellation_returns_cancelled() {
    let progress = Progress::default();
    progress.cancel();
    let result = ifc::import_reader(
        IFC2X3_MM.as_bytes(),
        IFC2X3_MM.len() as u64,
        &ImportOptions::default(),
        &progress,
    );
    assert_eq!(result.err(), Some(ImportError::Cancelled));
}

#[test]
fn missing_files_report_an_io_error() {
    let result = job::run(
        Path::new("no_existe_en_ningun_lado.ifc"),
        &ImportOptions::default(),
        &Progress::default(),
    );
    assert!(matches!(result, Err(ImportError::Io(_))));
}

#[test]
fn proprietary_files_are_refused_with_a_clear_error() {
    let result = job::run(
        Path::new("casa.rvt"),
        &ImportOptions::default(),
        &Progress::default(),
    );
    assert!(matches!(result, Err(ImportError::Invalid(_))));
}

#[test]
fn a_background_job_delivers_its_result() {
    let dir = std::env::temp_dir().join("gr_import_job_test");
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join("fixture.ifc");
    std::fs::write(&path, IFC2X3_MM).unwrap();
    let job = job::ImportJob::spawn(path.clone(), ImportOptions::default());
    let result = loop {
        if let Some(result) = job.poll() {
            break result;
        }
        std::thread::sleep(std::time::Duration::from_millis(10));
    };
    assert_eq!(result.unwrap().scene.element_count(), 4);
    let _ = std::fs::remove_file(path);
}
