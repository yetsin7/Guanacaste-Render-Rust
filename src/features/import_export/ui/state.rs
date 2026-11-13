//! State of the import dialog.

use crate::features::import_export::logic::formats::{detect, Detected};
use crate::features::import_export::models::ImportOptions;
use crate::features::import_export::services::job::{ImportJob, JobResult};
use crate::features::import_export::services::last_model;
use crate::features::import_export::services::picker::FilePicker;
use std::path::{Path, PathBuf};

/// Meters per unit of the unit choices offered for files without declared units.
pub const UNIT_CHOICES: [(&str, f64); 5] = [
    ("m", 1.0),
    ("cm", 0.01),
    ("mm", 0.001),
    ("ft", 0.3048),
    ("in", 0.0254),
];

/// Everything the import dialog remembers between frames.
#[derive(Default)]
pub struct ImportDialogState {
    /// Chosen file.
    pub path: Option<PathBuf>,
    /// What the chosen file is.
    pub detected: Option<Detected>,
    /// Options for formats without declared units or axes.
    pub options: ImportOptions,
    /// Selected index of [`UNIT_CHOICES`].
    pub unit_index: usize,
    /// The import running now.
    pub job: Option<ImportJob>,
    /// The last finished import.
    pub result: Option<Box<JobResult>>,
    /// Preview rotation around the vertical axis.
    pub yaw: f32,
    /// Preview elevation angle.
    pub pitch: f32,
    /// The native file dialog.
    pub picker: FilePicker,
    /// The user asked to use the finished import as the open project.
    pub adopt: bool,
    /// Where the last imported model is remembered.
    pub data_dir: Option<PathBuf>,
    /// The last model imported successfully, offered for reopening.
    pub last_model: Option<PathBuf>,
}

impl ImportDialogState {
    /// A state that remembers the last imported model inside `dir`.
    pub fn with_data_dir(dir: &Path) -> Self {
        Self {
            last_model: last_model::load(dir),
            data_dir: Some(dir.to_path_buf()),
            ..Self::default()
        }
    }

    /// Chooses `path`, classifies it and starts importing when the format is supported.
    pub fn choose(&mut self, path: PathBuf) {
        self.detected = Some(detect(&path));
        self.path = Some(path);
        self.result = None;
        if let Some(Detected::Supported(format)) = self.detected {
            self.options.y_up = format.default_y_up();
            self.start();
        }
    }

    /// (Re)starts the import of the chosen file with the current options.
    pub fn start(&mut self) {
        if let Some(job) = self.job.take() {
            job.progress.cancel();
        }
        self.options.unit_scale = UNIT_CHOICES[self.unit_index.min(UNIT_CHOICES.len() - 1)].1;
        if let Some(path) = self.path.clone() {
            self.result = None;
            self.job = Some(ImportJob::spawn(path, self.options));
        }
    }

    /// Cancels the running import.
    pub fn cancel(&mut self) {
        if let Some(job) = &self.job {
            job.progress.cancel();
        }
    }

    /// Collects finished work from the picker and the import thread; call once per frame.
    pub fn tick(&mut self) {
        if let Some(path) = self.picker.poll() {
            self.choose(path);
        }
        if let Some(result) = self.job.as_ref().and_then(ImportJob::poll) {
            self.job = None;
            if let (Ok(_), Some(path)) = (&result, &self.path) {
                if let Some(dir) = &self.data_dir {
                    let _ = last_model::save(dir, path);
                }
                self.last_model = Some(path.clone());
            }
            self.result = Some(Box::new(result));
        }
    }

    /// Whether something is still running (a thread or the native dialog).
    pub fn busy(&self) -> bool {
        self.job.is_some() || self.picker.is_open()
    }
}
