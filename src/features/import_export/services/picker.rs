//! Native file pickers (rfd) running on their own thread so the interface keeps drawing.

use crate::features::import_export::logic::formats::PICKER_EXTENSIONS;
use std::path::{Path, PathBuf};
use std::sync::mpsc::{channel, Receiver, TryRecvError};

/// A file dialog that may be open.
#[derive(Default)]
pub struct FilePicker {
    receiver: Option<Receiver<Option<PathBuf>>>,
}

fn named(dialog: rfd::FileDialog, suggested: &Path) -> rfd::FileDialog {
    let dialog = match suggested.parent().filter(|p| p.is_dir()) {
        Some(folder) => dialog.set_directory(folder),
        None => dialog,
    };
    match suggested.file_name().and_then(|n| n.to_str()) {
        Some(name) => dialog.set_file_name(name),
        None => dialog,
    }
}

impl FilePicker {
    fn spawn(&mut self, run: impl FnOnce() -> Option<PathBuf> + Send + 'static) {
        if self.receiver.is_some() {
            return;
        }
        let (sender, receiver) = channel();
        self.receiver = Some(receiver);
        let spawned = std::thread::Builder::new()
            .name("selector".into())
            .spawn(move || {
                let _ = sender.send(run());
            });
        if spawned.is_err() {
            self.receiver = None;
        }
    }

    /// Opens the model picker (IFC, OBJ, glTF, FBX and Guanacaste 3D projects).
    pub fn open(&mut self) {
        self.spawn(|| {
            rfd::FileDialog::new()
                .add_filter(
                    "Modelos 3D (IFC, OBJ, glTF, FBX, Guanacaste 3D)",
                    &PICKER_EXTENSIONS,
                )
                .add_filter("Todos los archivos", &["*"])
                .pick_file()
        });
    }

    /// Opens the project picker (`.gr`; the signature decides the format).
    pub fn open_project(&mut self) {
        self.spawn(|| {
            rfd::FileDialog::new()
                .add_filter("Proyectos de Guanacaste (.gr)", &["gr"])
                .add_filter("Proyectos de Guanacaste 3D (.g3d)", &["g3d"])
                .add_filter("Todos los archivos", &["*"])
                .pick_file()
        });
    }

    /// Opens a "save as" dialog for a project, proposing `suggested`.
    pub fn save_project(&mut self, suggested: &Path) {
        let suggested = suggested.to_path_buf();
        self.spawn(move || {
            named(rfd::FileDialog::new(), &suggested)
                .add_filter("Proyecto de Guanacaste Render (.gr)", &["gr"])
                .save_file()
        });
    }

    /// Opens a "save as" dialog for a PNG image, proposing `suggested`.
    pub fn save_image(&mut self, suggested: &Path) {
        let suggested = suggested.to_path_buf();
        self.spawn(move || {
            named(rfd::FileDialog::new(), &suggested)
                .add_filter("Imagen PNG", &["png"])
                .save_file()
        });
    }

    /// Whether the dialog is open.
    pub fn is_open(&self) -> bool {
        self.receiver.is_some()
    }

    /// The chosen file once the dialog closed (`None` while open or when cancelled).
    pub fn poll(&mut self) -> Option<PathBuf> {
        let receiver = self.receiver.as_ref()?;
        match receiver.try_recv() {
            Ok(picked) => {
                self.receiver = None;
                picked
            }
            Err(TryRecvError::Empty) => None,
            Err(TryRecvError::Disconnected) => {
                self.receiver = None;
                None
            }
        }
    }
}
