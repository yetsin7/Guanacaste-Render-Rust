//! The open project and every use case around it: new, open, import, save, save as, export, autosave,
//! recovery and the recent-files list. Dialogs ask; this type does the work and keeps the state.

use crate::features::import_export::models::ImportOutcome;
use crate::features::import_export::services::preview_image;
use crate::features::projects::logic::{convert, save_plan};
use crate::features::projects::models::Project;
use crate::features::projects::services::recent::{self, RecentEntry, RecentInfo};
use crate::features::projects::services::recovery::{self, RecoveryEntry};
use crate::features::projects::services::store::{self, LoadWarning};
use crate::features::projects::services::{open, util};
use crate::services::localization::{tr, trf};
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

/// Seconds between recovery copies of a project with unsaved changes.
pub const AUTOSAVE_EVERY: Duration = Duration::from_secs(60);

/// Something that must wait until the user decides about unsaved changes.
#[derive(Debug, Clone, PartialEq)]
pub enum Pending {
    /// Close the application.
    Close,
    /// Start a new empty project with this name.
    New(String),
    /// Open this file.
    Open(PathBuf),
    /// Use the finished import as the project.
    Import,
}

/// A one-line message for the user, with whether it is an error.
#[derive(Debug, Clone, PartialEq)]
pub struct Status {
    /// Text in Spanish.
    pub text: String,
    /// Whether it reports a failure.
    pub error: bool,
    /// When it was raised.
    pub at: Instant,
}

/// The application's project state.
pub struct Workspace {
    /// The open project.
    pub project: Option<Project>,
    /// Per-user data folder (recent list, recovery copies).
    pub data_dir: PathBuf,
    /// Recent project files that still exist, newest first.
    pub recent: Vec<RecentInfo>,
    /// Recovery copies found at start-up.
    pub recovery: Vec<RecoveryEntry>,
    /// Last result of an operation.
    pub status: Option<Status>,
    /// The user accepted to close the window.
    pub close_confirmed: bool,
    last_autosave: Instant,
}

impl Workspace {
    /// Loads the recent list and the recovery copies from `data_dir`.
    pub fn new(data_dir: PathBuf) -> Self {
        let recent = recent::describe(&recent::prune(&data_dir));
        let recovery = recovery::list(&data_dir);
        Self {
            project: None,
            data_dir,
            recent,
            recovery,
            status: None,
            close_confirmed: false,
            last_autosave: Instant::now(),
        }
    }

    /// Whether the open project has unsaved changes.
    pub fn dirty(&self) -> bool {
        self.project.as_ref().is_some_and(|p| p.dirty)
    }

    /// Name of the open project.
    pub fn name(&self) -> Option<&str> {
        self.project.as_ref().map(|p| p.document.meta.name.as_str())
    }

    /// Window title: `Casa • — Guanacaste Render` (the dot marks unsaved changes).
    pub fn window_title(&self) -> String {
        match &self.project {
            Some(project) => format!(
                "{}{} — Guanacaste Render",
                project.document.meta.name,
                if project.dirty { " •" } else { "" }
            ),
            None => "Guanacaste Render".into(),
        }
    }

    /// Shows `text` as the status message.
    pub fn notify(&mut self, text: String, error: bool) {
        self.status = Some(Status {
            text,
            error,
            at: Instant::now(),
        });
    }

    fn remember(&mut self, path: &Path, name: &str) {
        let _ = recent::remember(
            &self.data_dir,
            RecentEntry {
                path: path.to_path_buf(),
                name: name.to_owned(),
            },
        );
        self.recent = recent::describe(&recent::load(&self.data_dir));
    }

    fn replace(&mut self, project: Project) {
        if let Some(old) = &self.project {
            recovery::discard(&self.data_dir, &old.id);
        }
        self.project = Some(project);
        self.last_autosave = Instant::now();
    }

    /// Replaces the open project by a new empty one called `name`.
    pub fn start_new(&mut self, name: &str) {
        self.replace(store::new_project(name));
        self.notify(trf("workspace.created", &[name]), false);
    }

    /// Discards the recovery copy of the open project (its changes were dropped on purpose).
    pub fn forget_changes(&mut self) {
        if let Some(project) = &self.project {
            recovery::discard(&self.data_dir, &project.id);
        }
    }

    /// Opens `path` by signature, replacing the open project. Returns false on failure.
    pub fn open_file(&mut self, path: &Path) -> bool {
        match open::open(path) {
            Ok(opened) => {
                let name = opened.project.document.meta.name.clone();
                let text = if opened.project.legacy_source.is_some() {
                    tr("workspace.opened_legacy").to_owned()
                } else if opened
                    .warnings
                    .iter()
                    .any(|w| matches!(w, LoadWarning::NewerSchema { .. }))
                {
                    tr("workspace.opened_read_only").to_owned()
                } else {
                    trf("workspace.opened", &[&name])
                };
                if opened.project.path.is_some() {
                    self.remember(path, &name);
                }
                self.replace(opened.project);
                self.notify(text, false);
                true
            }
            Err(error) => {
                self.notify(format!("{} {}", tr(error.key()), error.detail()), true);
                false
            }
        }
    }

    /// Adopts a finished import as the open project (the caller already settled unsaved changes).
    pub fn adopt_import(&mut self, outcome: &ImportOutcome, source: &Path) {
        let file_name = source
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or_default();
        let name = source
            .file_stem()
            .and_then(|n| n.to_str())
            .unwrap_or("Proyecto");
        let (document, meshes) =
            convert::from_scene(&outcome.scene, name, file_name, util::now_unix());
        let legacy_source = (outcome.scene.source.format
            == crate::features::import_export::models::SourceFormat::Legacy)
            .then(|| source.to_path_buf());
        self.replace(Project {
            id: util::new_document_id(),
            document,
            meshes,
            path: None,
            legacy_source,
            read_only: false,
            dirty: true,
        });
        self.notify(trf("workspace.imported", &[file_name]), false);
    }

    /// PNG thumbnail of the open project (plan and isometric preview), when it has geometry.
    pub fn thumbnail(&self) -> Option<Vec<u8>> {
        let project = self.project.as_ref().filter(|p| !p.meshes.is_empty())?;
        let scene = convert::to_scene(&project.document, &project.meshes, 0);
        png(&preview_image::render(&scene, 192, 240))
    }

    /// Saves to `path` (adding `.gr`), never over a Guanacaste 3D file. Returns false on failure.
    pub fn save_to(&mut self, path: &Path, with_thumbnail: bool) -> bool {
        let path = save_plan::with_extension(path);
        let thumbnail = if with_thumbnail {
            self.thumbnail()
        } else {
            None
        };
        let Some(project) = self.project.as_mut() else {
            return false;
        };
        project.document.meta.modified = util::now_unix();
        match store::save(&path, project, thumbnail.as_deref()) {
            Ok(stats) => {
                project.path = Some(path.clone());
                project.dirty = false;
                let (id, name) = (project.id, project.document.meta.name.clone());
                recovery::discard(&self.data_dir, &id);
                self.remember(&path, &name);
                self.notify(
                    trf(
                        "workspace.saved",
                        &[
                            &path.display().to_string(),
                            &crate::features::projects::logic::describe::size_label(stats.bytes),
                        ],
                    ),
                    false,
                );
                true
            }
            Err(error) => {
                self.notify(format!("{} {}", tr(error.key()), error.detail()), true);
                false
            }
        }
    }

    /// Writes a recovery copy when there are unsaved changes and the interval has passed.
    pub fn autosave(&mut self) {
        if !self.dirty() || self.last_autosave.elapsed() < AUTOSAVE_EVERY {
            return;
        }
        self.last_autosave = Instant::now();
        if let Some(project) = &self.project {
            if recovery::write(&self.data_dir, project).is_err() {
                self.notify(tr("workspace.autosave_failed").to_owned(), true);
            }
        }
    }

    /// Restores recovery copy `index` as an unsaved project and removes it from the list.
    pub fn restore(&mut self, index: usize) {
        if index >= self.recovery.len() {
            return;
        }
        let entry = self.recovery.remove(index);
        match store::load(&entry.path, Default::default()) {
            Ok(loaded) => {
                let mut project = loaded.project;
                project.path = None;
                project.dirty = true;
                self.project = Some(project);
                self.notify(trf("workspace.restored", &[&entry.name]), false);
            }
            Err(error) => self.notify(format!("{} {}", tr(error.key()), error.detail()), true),
        }
    }

    /// Deletes every remaining recovery copy.
    pub fn discard_recovery(&mut self) {
        for entry in self.recovery.drain(..) {
            recovery::discard_path(&entry.path);
        }
    }

    /// Writes a `width`×`height` PNG preview (plan and isometric) of the open project to `path`.
    pub fn export_png(&mut self, path: &Path, width: u32, height: u32) {
        let Some(project) = self.project.as_ref().filter(|p| !p.meshes.is_empty()) else {
            self.notify(tr("workspace.export_empty").to_owned(), true);
            return;
        };
        let scene = convert::to_scene(&project.document, &project.meshes, 0);
        let image = preview_image::render(&scene, (width / 2).max(16), height.max(16));
        match image.save(path) {
            Ok(()) => self.notify(
                trf("workspace.exported", &[&path.display().to_string()]),
                false,
            ),
            Err(error) => self.notify(format!("{} {error}", tr("workspace.export_failed")), true),
        }
    }
}

/// Encodes an RGB image as PNG bytes.
fn png(image: &image::RgbImage) -> Option<Vec<u8>> {
    let mut bytes = std::io::Cursor::new(Vec::new());
    image.write_to(&mut bytes, image::ImageFormat::Png).ok()?;
    Some(bytes.into_inner())
}
