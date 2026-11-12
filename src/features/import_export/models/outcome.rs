//! What a finished import returns.

use super::{ImportReport, ImportedScene};

/// A scene together with its import report.
#[derive(Debug, Clone, PartialEq)]
pub struct ImportOutcome {
    /// The imported scene.
    pub scene: ImportedScene,
    /// What happened while importing.
    pub report: ImportReport,
}
