//! Opening any `.gr`: the first eight bytes decide whether it is a Guanacaste Render project (`GRRPROJ1`) or a
//! Guanacaste 3D project (`G3DNATV1`, imported read-only). The extension is never trusted.

use super::container::Limits;
use super::store::{self, LoadWarning, StoreError};
use super::util;
use crate::features::import_export::logic::formats::{sniff, Signature};
use crate::features::import_export::models::{ImportError, ImportReport};
use crate::features::import_export::services::{legacy, progress::Progress};
use crate::features::projects::logic::convert;
use crate::features::projects::models::Project;
use std::path::Path;

/// Why a file could not be opened.
#[derive(Debug, Clone, PartialEq)]
pub enum OpenError {
    /// A Guanacaste Render project that failed to load.
    Store(StoreError),
    /// A Guanacaste 3D project that failed to import.
    Legacy(ImportError),
    /// Neither signature: not a project file.
    UnknownSignature,
}

impl OpenError {
    /// Localization key of the headline.
    pub fn key(&self) -> &'static str {
        match self {
            OpenError::Store(error) => error.key(),
            OpenError::Legacy(error) => error.key(),
            OpenError::UnknownSignature => "store.err_signature",
        }
    }

    /// Technical detail.
    pub fn detail(&self) -> String {
        match self {
            OpenError::Store(error) => error.detail(),
            OpenError::Legacy(error) => error.detail().to_owned(),
            OpenError::UnknownSignature => String::new(),
        }
    }
}

/// An opened project.
#[derive(Debug, Clone, PartialEq)]
pub struct Opened {
    /// The project.
    pub project: Project,
    /// Load notes (newer schema, migration).
    pub warnings: Vec<LoadWarning>,
    /// Stored thumbnail PNG.
    pub thumbnail: Option<Vec<u8>>,
    /// Import report when the file was a Guanacaste 3D project.
    pub report: Option<ImportReport>,
}

/// Opens `path` by signature.
pub fn open(path: &Path) -> Result<Opened, OpenError> {
    let signature = sniff(path).map_err(|e| OpenError::Store(e.into()))?;
    match signature {
        Signature::Render => {
            let loaded = store::load(path, Limits::default()).map_err(OpenError::Store)?;
            Ok(Opened {
                project: loaded.project,
                warnings: loaded.warnings,
                thumbnail: loaded.thumbnail,
                report: None,
            })
        }
        Signature::Legacy => {
            let outcome = legacy::import(path, &Progress::default()).map_err(OpenError::Legacy)?;
            let file_name = path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or_default();
            let name = outcome
                .scene
                .nodes
                .first()
                .map(|n| n.name.clone())
                .filter(|n| !n.is_empty())
                .unwrap_or_else(|| file_name.to_owned());
            let (document, meshes) =
                convert::from_scene(&outcome.scene, &name, file_name, util::now_unix());
            let project = Project {
                id: util::new_document_id(),
                document,
                meshes,
                path: None,
                legacy_source: Some(path.to_path_buf()),
                read_only: false,
                dirty: true,
            };
            Ok(Opened {
                project,
                warnings: Vec::new(),
                thumbnail: None,
                report: Some(outcome.report),
            })
        }
        Signature::Unknown => Err(OpenError::UnknownSignature),
    }
}
