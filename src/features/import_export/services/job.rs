//! Running an import on a background thread with progress and cancellation; panics become errors.

use super::progress::Progress;
use super::{fbx, gltf, ifc, legacy, obj};
use crate::features::import_export::logic::formats::{detect, sniff, Detected, Signature};
use crate::features::import_export::models::{
    ImportError, ImportOptions, ImportOutcome, SourceFormat,
};
use std::panic::{catch_unwind, AssertUnwindSafe};
use std::path::{Path, PathBuf};
use std::sync::mpsc::{channel, Receiver, TryRecvError};
use std::sync::Arc;

/// The result of a finished import.
pub type JobResult = Result<ImportOutcome, ImportError>;

/// Imports `path` on the calling thread, choosing the reader by extension.
pub fn run(path: &Path, options: &ImportOptions, progress: &Progress) -> JobResult {
    let format = match detect(path) {
        Detected::Supported(format) => format,
        _ => {
            return Err(ImportError::Invalid(
                "Formato de archivo no soportado.".into(),
            ))
        }
    };
    if format == SourceFormat::Legacy {
        match sniff(path)? {
            Signature::Legacy => {}
            Signature::Render => {
                return Err(ImportError::Invalid(
                    "Es un proyecto de Guanacaste Render: ábrelo con Abrir proyecto.".into(),
                ))
            }
            Signature::Unknown => return Err(ImportError::Invalid(
                "Firma desconocida: no es un proyecto de Guanacaste Render ni de Guanacaste 3D."
                    .into(),
            )),
        }
    }
    let outcome = catch_unwind(AssertUnwindSafe(|| match format {
        SourceFormat::Ifc => ifc::import(path, options, progress),
        SourceFormat::Obj => obj::import(path, options, progress),
        SourceFormat::Gltf => gltf::import(path, progress),
        SourceFormat::Fbx => fbx::import(path, progress),
        SourceFormat::Legacy => legacy::import(path, progress),
    }));
    outcome.unwrap_or_else(|payload| {
        let text = payload
            .downcast_ref::<&str>()
            .map(|s| (*s).to_owned())
            .or_else(|| payload.downcast_ref::<String>().cloned())
            .unwrap_or_else(|| "error interno".into());
        Err(ImportError::Crashed(text))
    })
}

/// A running import.
pub struct ImportJob {
    /// The file being imported.
    pub path: PathBuf,
    /// Shared progress and cancel flag.
    pub progress: Arc<Progress>,
    receiver: Receiver<JobResult>,
}

impl ImportJob {
    /// Starts importing `path` in a new thread.
    pub fn spawn(path: PathBuf, options: ImportOptions) -> Self {
        let progress = Arc::new(Progress::default());
        let (sender, receiver) = channel();
        let worker_progress = Arc::clone(&progress);
        let worker_path = path.clone();
        let spawned = std::thread::Builder::new()
            .name("importador".into())
            .spawn(move || {
                let _ = sender.send(run(&worker_path, &options, &worker_progress));
            });
        if let Err(error) = spawned {
            let (failed, receiver) = channel();
            let _ = failed.send(Err(ImportError::Crashed(error.to_string())));
            return Self {
                path,
                progress,
                receiver,
            };
        }
        Self {
            path,
            progress,
            receiver,
        }
    }

    /// The result when the import has finished; `None` while it runs.
    pub fn poll(&self) -> Option<JobResult> {
        match self.receiver.try_recv() {
            Ok(result) => Some(result),
            Err(TryRecvError::Empty) => None,
            Err(TryRecvError::Disconnected) => Some(Err(ImportError::Crashed(
                "el hilo de importación terminó sin resultado".into(),
            ))),
        }
    }
}
