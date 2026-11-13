//! Recovery copies: autosaved projects in the user's data folder, restorable after an unexpected exit.

use super::store::{self, SaveStats, StoreError};
use super::util::hex;
use crate::features::projects::models::Project;
use std::path::{Path, PathBuf};

/// The per-user data folder of the application (`%LOCALAPPDATA%\GuanacasteRender` on Windows).
pub fn data_dir() -> PathBuf {
    let base = std::env::var_os("LOCALAPPDATA")
        .or_else(|| std::env::var_os("XDG_DATA_HOME"))
        .map(PathBuf::from)
        .or_else(|| {
            std::env::var_os("HOME").map(|home| PathBuf::from(home).join(".local").join("share"))
        })
        .unwrap_or_else(std::env::temp_dir);
    base.join("GuanacasteRender")
}

/// Folder holding the recovery copies under `data`.
pub fn recovery_dir(data: &Path) -> PathBuf {
    data.join("recovery")
}

/// Path of the recovery copy of the document with `id`.
pub fn recovery_path(data: &Path, id: &[u8; 16]) -> PathBuf {
    recovery_dir(data).join(format!("{}.gr", hex(id)))
}

/// A recovery copy found on disk.
#[derive(Debug, Clone, PartialEq)]
pub struct RecoveryEntry {
    /// File path.
    pub path: PathBuf,
    /// Project name stored in the file.
    pub name: String,
    /// Modification time, Unix seconds.
    pub modified: u64,
    /// Size in bytes.
    pub size: u64,
}

/// Writes (atomically) the recovery copy of `project`.
pub fn write(data: &Path, project: &Project) -> Result<SaveStats, StoreError> {
    std::fs::create_dir_all(recovery_dir(data))?;
    store::save(&recovery_path(data, &project.id), project, None)
}

/// Lists readable recovery copies, newest first. Unreadable files are skipped.
pub fn list(data: &Path) -> Vec<RecoveryEntry> {
    let Ok(entries) = std::fs::read_dir(recovery_dir(data)) else {
        return Vec::new();
    };
    let mut found: Vec<RecoveryEntry> = entries
        .flatten()
        .map(|entry| entry.path())
        .filter(|path| path.extension().is_some_and(|e| e == "gr"))
        .filter_map(|path| {
            let meta = std::fs::metadata(&path).ok()?;
            let peek = store::peek(&path).ok()?;
            let modified = meta
                .modified()
                .ok()?
                .duration_since(std::time::UNIX_EPOCH)
                .ok()?
                .as_secs();
            Some(RecoveryEntry {
                path,
                name: peek.name,
                modified,
                size: meta.len(),
            })
        })
        .collect();
    found.sort_by_key(|entry| std::cmp::Reverse(entry.modified));
    found
}

/// Deletes the recovery copy of the document with `id`, if any.
pub fn discard(data: &Path, id: &[u8; 16]) {
    let _ = std::fs::remove_file(recovery_path(data, id));
}

/// Deletes a recovery file by path.
pub fn discard_path(path: &Path) {
    let _ = std::fs::remove_file(path);
}
