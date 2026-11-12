//! IFC (STEP) importer: units, hierarchy, materials, styles and geometry into the common scene model.

mod brep;
mod build;
mod context;
mod finish;
mod items;
mod piece;
mod profile;
mod representation;
mod solids;
mod styles;
mod units;

use crate::features::import_export::models::{ImportError, ImportOptions, ImportOutcome};
use crate::features::import_export::services::progress::Progress;
use crate::features::import_export::services::step;
use std::io::Read;
use std::path::Path;

/// Entity types that never matter for the 3D model and are dropped while reading (properties, history...).
fn keep(ty: &str) -> bool {
    const DROPPED: [&str; 14] = [
        "IFCPROPERTY",
        "IFCQUANTITY",
        "IFCRELDEFINESBYPROPERTIES",
        "IFCPROPERTYSET",
        "IFCELEMENTQUANTITY",
        "IFCOWNERHISTORY",
        "IFCPERSON",
        "IFCORGANIZATION",
        "IFCAPPLICATION",
        "IFCCLASSIFICATION",
        "IFCRELASSOCIATESCLASSIFICATION",
        "IFCDOCUMENT",
        "IFCRELASSOCIATESDOCUMENT",
        "IFCRELASSIGNS",
    ];
    !DROPPED.iter().any(|prefix| ty.starts_with(prefix))
}

/// Imports an IFC stream of `size` bytes.
pub fn import_reader<R: Read>(
    reader: R,
    size: u64,
    _options: &ImportOptions,
    progress: &Progress,
) -> Result<ImportOutcome, ImportError> {
    let started = std::time::Instant::now();
    let file = step::parse_stream(reader, size, &keep, progress, (0.0, 0.6))?;
    let mut outcome = build::scene(&file, size, progress)?;
    outcome.report.duration_ms = started.elapsed().as_millis();
    progress.set(1.0);
    Ok(outcome)
}

/// Imports the IFC file at `path`.
pub fn import(
    path: &Path,
    options: &ImportOptions,
    progress: &Progress,
) -> Result<ImportOutcome, ImportError> {
    let file = std::fs::File::open(path)?;
    let size = file.metadata()?.len();
    import_reader(file, size, options, progress)
}

/// Text label of a length scale (meters per unit), shared by the other importers.
pub fn label_for(scale: f64) -> String {
    units::length_label(scale)
}
