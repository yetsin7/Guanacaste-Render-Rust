//! Use case: the example model of the start screen, the "Casa del Cobogó" of Guanacaste 3D. The file is too big to
//! ship with the repository, so it is looked up on this computer: first the path in `GR_SAMPLE_MODEL`, then a
//! shallow search for a `.g3d` or `.gr` file whose name contains "cobog" under the folders where the developer
//! keeps projects (`C:\Dev` and the user's Documents). The import itself is done by the normal importer.

use std::path::{Path, PathBuf};

/// Environment variable that names the example file explicitly.
pub const ENV_PATH: &str = "GR_SAMPLE_MODEL";
/// Text the file name must contain (lower case).
const NAME_PART: &str = "cobog";
/// Folders searched in depth.
const MAX_DEPTH: usize = 6;
/// Folders never entered.
const SKIPPED: [&str; 6] = ["node_modules", "target", ".git", "bin", "obj", ".claude"];

fn is_model(path: &Path) -> bool {
    let name = path
        .file_name()
        .map(|n| n.to_string_lossy().to_lowercase())
        .unwrap_or_default();
    let extension = path
        .extension()
        .map(|e| e.to_string_lossy().to_lowercase())
        .unwrap_or_default();
    name.contains(NAME_PART) && (extension == "g3d" || extension == "gr")
}

/// The first example file under `folder`, searching `depth` levels down.
pub fn search(folder: &Path, depth: usize) -> Option<PathBuf> {
    let entries = std::fs::read_dir(folder).ok()?;
    let mut folders = Vec::new();
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            let skip = path
                .file_name()
                .is_some_and(|n| SKIPPED.contains(&n.to_string_lossy().as_ref()));
            if !skip {
                folders.push(path);
            }
        } else if is_model(&path) {
            return Some(path);
        }
    }
    if depth == 0 {
        return None;
    }
    folders.into_iter().find_map(|f| search(&f, depth - 1))
}

/// Looks for the example model on this computer.
pub fn locate() -> Option<PathBuf> {
    if let Some(path) = std::env::var_os(ENV_PATH).map(PathBuf::from) {
        if path.is_file() {
            return Some(path);
        }
    }
    let mut roots = vec![PathBuf::from("C:/Dev")];
    if let Some(home) = std::env::var_os("USERPROFILE").or_else(|| std::env::var_os("HOME")) {
        roots.push(PathBuf::from(home).join("Documents"));
    }
    roots.iter().find_map(|root| search(root, MAX_DEPTH))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_cobogo_models_match() {
        assert!(
            is_model(Path::new("C:/x/Casa del Cobogó.g3d"))
                || is_model(Path::new("casa-cobogo.g3d"))
        );
        assert!(!is_model(Path::new("casa.g3d")));
        assert!(!is_model(Path::new("cobogo.png")));
    }

    #[test]
    fn the_search_finds_a_file_in_a_subfolder() {
        let root = std::env::temp_dir().join("gr-sample-search");
        let inner = root.join("a").join("b");
        std::fs::create_dir_all(&inner).unwrap();
        std::fs::write(inner.join("Casa cobogo.g3d"), b"x").unwrap();
        assert_eq!(search(&root, 3), Some(inner.join("Casa cobogo.g3d")));
        assert_eq!(search(&root, 0), None);
        let _ = std::fs::remove_dir_all(&root);
    }
}
