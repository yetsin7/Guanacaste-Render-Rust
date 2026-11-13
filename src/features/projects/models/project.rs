//! A project open in the application.

use super::document::ProjectDocument;
use crate::features::import_export::models::Mesh;
use std::path::PathBuf;

/// The open project: its document, its meshes and its place on disk.
#[derive(Debug, Clone, PartialEq)]
pub struct Project {
    /// Identifier kept across saves (16 bytes).
    pub id: [u8; 16],
    /// JSON document.
    pub document: ProjectDocument,
    /// Mesh data, one entry per `document.meshes`.
    pub meshes: Vec<Mesh>,
    /// File the project was loaded from or saved to. Never a Guanacaste 3D file.
    pub path: Option<PathBuf>,
    /// Guanacaste 3D file the project was imported from; it is never written.
    pub legacy_source: Option<PathBuf>,
    /// True when it came from a newer schema: it can be viewed but never overwritten.
    pub read_only: bool,
    /// True when there are changes not saved to `path`.
    pub dirty: bool,
}
