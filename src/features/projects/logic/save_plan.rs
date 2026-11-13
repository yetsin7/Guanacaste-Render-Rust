//! Where "Save" writes: the project's own `.gr`, or a new name when the project came from Guanacaste 3D.

use crate::features::projects::models::Project;
use std::path::{Path, PathBuf};

/// Extension of every project file (Guanacaste Render and Guanacaste 3D alike; the signature tells them apart).
pub const EXTENSION: &str = "gr";

/// What "Save" must do.
#[derive(Debug, Clone, PartialEq)]
pub enum SaveTarget {
    /// Write over the project's own file.
    Overwrite(PathBuf),
    /// Ask for a name, proposing `suggested`; `reason_key` explains why when it is not a first save.
    AskAs {
        /// Proposed path.
        suggested: PathBuf,
        /// Localization key of the one-line explanation, if any.
        reason_key: Option<&'static str>,
    },
}

/// `<folder>/<stem> - render.gr` next to a Guanacaste 3D file, so the original is never overwritten.
pub fn render_copy_path(legacy: &Path) -> PathBuf {
    let stem = legacy
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("proyecto");
    legacy.with_file_name(format!("{stem} - render.{EXTENSION}"))
}

/// Adds `.gr` when the chosen name has no extension or another one.
pub fn with_extension(path: &Path) -> PathBuf {
    let has = path
        .extension()
        .and_then(|e| e.to_str())
        .is_some_and(|e| e.eq_ignore_ascii_case(EXTENSION));
    if has {
        path.to_path_buf()
    } else {
        let mut name = path.as_os_str().to_os_string();
        name.push(".");
        name.push(EXTENSION);
        PathBuf::from(name)
    }
}

/// Decides what "Save" does for `project`.
pub fn save_target(project: &Project) -> SaveTarget {
    match (&project.path, &project.legacy_source) {
        (Some(path), _) if !project.read_only => SaveTarget::Overwrite(path.clone()),
        (_, Some(legacy)) => SaveTarget::AskAs {
            suggested: render_copy_path(legacy),
            reason_key: Some("save.legacy_reason"),
        },
        (Some(path), None) => SaveTarget::AskAs {
            suggested: render_copy_path(path),
            reason_key: Some("save.read_only_reason"),
        },
        (None, None) => SaveTarget::AskAs {
            suggested: PathBuf::from(format!("{}.{EXTENSION}", project.document.meta.name)),
            reason_key: None,
        },
    }
}
