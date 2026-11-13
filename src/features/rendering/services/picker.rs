//! A folder picker that runs on its own thread so the interface keeps drawing, and "open folder".

use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::mpsc::{channel, Receiver, TryRecvError};

/// A folder dialog that may be open.
#[derive(Default)]
pub struct FolderPicker {
    receiver: Option<Receiver<Option<PathBuf>>>,
}

impl FolderPicker {
    /// Opens the dialog starting at `start` when that folder exists. Does nothing while one is already open.
    pub fn open(&mut self, start: &str) {
        if self.receiver.is_some() {
            return;
        }
        let start = PathBuf::from(start.trim());
        let (sender, receiver) = channel();
        let spawned = std::thread::Builder::new()
            .name("folder-picker".into())
            .spawn(move || {
                let mut dialog = rfd::FileDialog::new();
                if start.is_dir() {
                    dialog = dialog.set_directory(&start);
                }
                let _ = sender.send(dialog.pick_folder());
            });
        if spawned.is_ok() {
            self.receiver = Some(receiver);
        }
    }

    /// Whether the dialog is open.
    pub fn is_open(&self) -> bool {
        self.receiver.is_some()
    }

    /// The folder the user chose, once, after the dialog closes. `None` while open or when cancelled.
    pub fn poll(&mut self) -> Option<PathBuf> {
        let receiver = self.receiver.as_ref()?;
        match receiver.try_recv() {
            Ok(choice) => {
                self.receiver = None;
                choice
            }
            Err(TryRecvError::Empty) => None,
            Err(TryRecvError::Disconnected) => {
                self.receiver = None;
                None
            }
        }
    }
}

/// Shows `folder` in the system file manager. Returns false when it does not exist or cannot be launched.
pub fn open_folder(folder: &Path) -> bool {
    if !folder.is_dir() {
        return false;
    }
    let program = if cfg!(target_os = "windows") {
        "explorer"
    } else if cfg!(target_os = "macos") {
        "open"
    } else {
        "xdg-open"
    };
    Command::new(program).arg(folder).spawn().is_ok()
}
