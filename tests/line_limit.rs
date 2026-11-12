//! Architecture rule: no Rust source file may exceed 400 lines.

use std::fs;
use std::path::{Path, PathBuf};

const MAX_LINES: usize = 400;

fn rust_files(dir: &Path, found: &mut Vec<PathBuf>) {
    for entry in fs::read_dir(dir).expect("readable directory") {
        let path = entry.expect("directory entry").path();
        if path.is_dir() {
            rust_files(&path, found);
        } else if path.extension().is_some_and(|extension| extension == "rs") {
            found.push(path);
        }
    }
}

#[test]
fn no_source_file_exceeds_the_line_limit() {
    let mut files = Vec::new();
    for root in ["src", "tests"] {
        rust_files(
            &Path::new(env!("CARGO_MANIFEST_DIR")).join(root),
            &mut files,
        );
    }
    assert!(!files.is_empty(), "no source files found");
    let too_long: Vec<String> = files
        .iter()
        .filter_map(|path| {
            let lines = fs::read_to_string(path)
                .expect("readable source")
                .lines()
                .count();
            (lines > MAX_LINES).then(|| format!("{} has {lines} lines", path.display()))
        })
        .collect();
    assert!(
        too_long.is_empty(),
        "files over {MAX_LINES} lines:\n{}",
        too_long.join("\n")
    );
}
