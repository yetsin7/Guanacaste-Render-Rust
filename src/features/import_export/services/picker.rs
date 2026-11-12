//! Native file picker (rfd) running on its own thread so the interface keeps drawing.

use crate::features::import_export::logic::formats::PICKER_EXTENSIONS;
use std::path::PathBuf;
use std::sync::mpsc::{channel, Receiver, TryRecvError};

/// A file dialog that may be open.
#[derive(Default)]
pub struct FilePicker {
    receiver: Option<Receiver<Option<PathBuf>>>,
}

impl FilePicker {
    /// Opens the dialog unless one is already open.
    pub fn open(&mut self) {
        if self.receiver.is_some() {
            return;
        }
        let (sender, receiver) = channel();
        self.receiver = Some(receiver);
        let spawned = std::thread::Builder::new()
            .name("selector".into())
            .spawn(move || {
                let picked = rfd::FileDialog::new()
                    .add_filter("Modelos 3D (IFC, OBJ, glTF, FBX)", &PICKER_EXTENSIONS)
                    .add_filter("Todos los archivos", &["*"])
                    .pick_file();
                let _ = sender.send(picked);
            });
        if spawned.is_err() {
            self.receiver = None;
        }
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
