//! Localization coverage: every key referenced from the code exists in the catalog and every catalog key is used.

use guanacaste_render_rust::services::localization::entries;
use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};

fn sources(dir: &Path, found: &mut Vec<PathBuf>) {
    for entry in fs::read_dir(dir).expect("readable directory") {
        let path = entry.expect("directory entry").path();
        if path.is_dir() {
            if path.ends_with("localization") {
                continue;
            }
            sources(&path, found);
        } else if path.extension().is_some_and(|extension| extension == "rs") {
            found.push(path);
        }
    }
}

/// Every double-quoted literal of a source file (escapes are not interpreted: keys never contain them).
fn literals(text: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut parts = text.split('"');
    parts.next();
    for (index, part) in parts.enumerate() {
        if index % 2 == 0 {
            out.push(part.to_owned());
        }
    }
    out
}

/// Literal first arguments of `tr("...")` and `trf("...")` calls (not of functions that merely end in `tr`).
fn direct_calls(text: &str) -> Vec<&str> {
    let mut found = Vec::new();
    for call in ["tr(\"", "trf(\""] {
        let mut from = 0;
        while let Some(at) = text[from..].find(call).map(|i| i + from) {
            let before = text[..at].chars().next_back();
            from = at + call.len();
            if before.is_some_and(|c| c.is_alphanumeric() || c == '_') {
                continue;
            }
            found.push(text[from..].split('"').next().unwrap_or_default());
        }
    }
    found
}

#[test]
fn keys_used_in_code_exist_and_catalog_has_no_dead_keys() {
    let keys: HashSet<&str> = entries().map(|(key, _)| *key).collect();
    let prefixes: HashSet<&str> = keys
        .iter()
        .filter_map(|key| key.split('.').next())
        .collect();
    let mut files = Vec::new();
    sources(
        &Path::new(env!("CARGO_MANIFEST_DIR")).join("src"),
        &mut files,
    );
    let mut used = HashSet::new();
    let mut missing = Vec::new();
    for file in files {
        let text = fs::read_to_string(&file).expect("readable source");
        // Skip test modules: they may probe unknown keys on purpose.
        let text = text.split("#[cfg(test)]").next().unwrap_or("");
        for literal in direct_calls(text) {
            if !keys.contains(literal) {
                missing.push(format!("{} -> {literal} (tr/trf)", file.display()));
            }
        }
        for literal in literals(text) {
            let Some((prefix, rest)) = literal.split_once('.') else {
                continue;
            };
            let looks_like_key = prefixes.contains(prefix)
                && !rest.is_empty()
                && rest
                    .chars()
                    .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_' || c == '.');
            if looks_like_key {
                if keys.contains(literal.as_str()) {
                    used.insert(literal);
                } else {
                    missing.push(format!("{} -> {literal}", file.display()));
                }
            }
        }
    }
    assert!(
        missing.is_empty(),
        "unknown localization keys:\n{}",
        missing.join("\n")
    );
    let mut dead: Vec<&&str> = keys.iter().filter(|key| !used.contains(**key)).collect();
    dead.sort();
    assert!(dead.is_empty(), "catalog keys never used: {dead:?}");
}

#[test]
fn texts_are_not_empty() {
    for (key, text) in entries() {
        assert!(!text.trim().is_empty(), "empty text for {key}");
    }
}
