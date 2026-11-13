//! The recent-projects list, stored as JSON in the user's data folder.

use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

const MAX_RECENT: usize = 12;

/// A recent project as stored on disk.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RecentEntry {
    /// Path of the `.gr` file.
    pub path: PathBuf,
    /// Project name when it was last opened or saved.
    pub name: String,
}

fn list_path(data: &Path) -> PathBuf {
    data.join("recent.json")
}

/// Reads the list; a missing or unreadable file gives an empty list.
pub fn load(data: &Path) -> Vec<RecentEntry> {
    std::fs::read(list_path(data))
        .ok()
        .and_then(|bytes| serde_json::from_slice(&bytes).ok())
        .unwrap_or_default()
}

/// Moves `entry` to the front of the list (replacing an older entry for the same path) and saves it.
pub fn remember(data: &Path, entry: RecentEntry) -> std::io::Result<Vec<RecentEntry>> {
    let mut list = load(data);
    list.retain(|existing| existing.path != entry.path);
    list.insert(0, entry);
    list.truncate(MAX_RECENT);
    std::fs::create_dir_all(data)?;
    std::fs::write(
        list_path(data),
        serde_json::to_vec_pretty(&list).map_err(std::io::Error::other)?,
    )?;
    Ok(list)
}

/// Removes entries whose files no longer exist and saves the result.
pub fn prune(data: &Path) -> Vec<RecentEntry> {
    let mut list = load(data);
    list.retain(|entry| entry.path.exists());
    let _ = std::fs::write(
        list_path(data),
        serde_json::to_vec_pretty(&list).unwrap_or_default(),
    );
    list
}

/// Real file information of a recent entry.
#[derive(Debug, Clone, PartialEq)]
pub struct RecentInfo {
    /// The stored entry.
    pub entry: RecentEntry,
    /// Size in bytes.
    pub size: u64,
    /// Modification time, Unix seconds.
    pub modified: u64,
}

/// Stat-s every entry, skipping files that no longer exist.
pub fn describe(list: &[RecentEntry]) -> Vec<RecentInfo> {
    list.iter()
        .filter_map(|entry| {
            let meta = std::fs::metadata(&entry.path).ok()?;
            let modified = meta
                .modified()
                .ok()?
                .duration_since(std::time::UNIX_EPOCH)
                .ok()?
                .as_secs();
            Some(RecentInfo {
                entry: entry.clone(),
                size: meta.len(),
                modified,
            })
        })
        .collect()
}
