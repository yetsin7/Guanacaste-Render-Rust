//! Native image file dialog (rfd) on its own thread so the interface keeps drawing while it is open.

use crate::features::materials::models::MapKind;
use crate::services::localization::tr;
use std::path::PathBuf;
use std::sync::mpsc::{channel, Receiver, TryRecvError};

/// File extensions the texture loader understands.
pub const IMAGE_EXTENSIONS: [&str; 3] = ["png", "jpg", "jpeg"];

/// A dialog that may be open, remembering which texture slot it is choosing a file for.
#[derive(Default)]
pub struct ImagePicker {
    pending: Option<(MapKind, Receiver<Option<PathBuf>>)>,
}

impl ImagePicker {
    /// Opens the dialog for slot `kind`; ignored while another dialog is open.
    pub fn open(&mut self, kind: MapKind) {
        if self.pending.is_some() {
            return;
        }
        let (sender, receiver) = channel();
        let spawned = std::thread::Builder::new()
            .name("selector-imagen".into())
            .spawn(move || {
                let picked = rfd::FileDialog::new()
                    .add_filter(tr("material.picker_images"), &IMAGE_EXTENSIONS)
                    .add_filter(tr("material.picker_all"), &["*"])
                    .pick_file();
                let _ = sender.send(picked);
            });
        if spawned.is_ok() {
            self.pending = Some((kind, receiver));
        }
    }

    /// Whether the dialog is open.
    pub fn is_open(&self) -> bool {
        self.pending.is_some()
    }

    /// The slot and file chosen once the dialog closed (`None` while open or when cancelled).
    pub fn poll(&mut self) -> Option<(MapKind, PathBuf)> {
        let (kind, receiver) = self.pending.as_ref()?;
        let kind = *kind;
        match receiver.try_recv() {
            Ok(picked) => {
                self.pending = None;
                picked.map(|path| (kind, path))
            }
            Err(TryRecvError::Empty) => None,
            Err(TryRecvError::Disconnected) => {
                self.pending = None;
                None
            }
        }
    }
}
