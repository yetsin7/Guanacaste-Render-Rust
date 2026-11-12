//! Spanish text catalog. Every user-visible string is looked up by key with [`tr`]; per-domain tables live in
//! sibling files so no table grows too large.

mod common;
mod dialogs;
mod help;
mod import_export;
mod library;
mod materials;
mod movie;
mod projects;
mod rendering;
mod settings;
mod viewport;

use std::collections::HashMap;
use std::sync::OnceLock;

/// Every table of the catalog.
const TABLES: &[&[(&str, &str)]] = &[
    common::ENTRIES,
    dialogs::ENTRIES,
    import_export::ENTRIES,
    projects::ENTRIES,
    viewport::ENTRIES,
    library::ENTRIES,
    materials::ENTRIES,
    movie::ENTRIES,
    rendering::ENTRIES,
    settings::ENTRIES,
    help::ENTRIES,
];

/// All `(key, text)` pairs of the catalog.
pub fn entries() -> impl Iterator<Item = &'static (&'static str, &'static str)> {
    TABLES.iter().flat_map(|table| table.iter())
}

fn catalog() -> &'static HashMap<&'static str, &'static str> {
    static CATALOG: OnceLock<HashMap<&'static str, &'static str>> = OnceLock::new();
    CATALOG.get_or_init(|| entries().map(|(key, text)| (*key, *text)).collect())
}

/// Returns the Spanish text for `key`; an unknown key yields the key itself so the gap is visible.
pub fn tr(key: &'static str) -> &'static str {
    catalog().get(key).copied().unwrap_or(key)
}

/// Like [`tr`], substituting `{0}`, `{1}`... with the given arguments.
pub fn trf(key: &'static str, args: &[&str]) -> String {
    let mut text = tr(key).to_owned();
    for (index, value) in args.iter().enumerate() {
        text = text.replace(&format!("{{{index}}}"), value);
    }
    text
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keys_are_unique() {
        let mut seen = std::collections::HashSet::new();
        for (key, _) in entries() {
            assert!(seen.insert(*key), "duplicate localization key {key}");
        }
    }

    #[test]
    fn substitutes_arguments() {
        assert_eq!(
            trf("projects.modified", &["14 min"]),
            "Modificado hace 14 min"
        );
    }

    #[test]
    fn unknown_key_is_returned_verbatim() {
        assert_eq!(tr("no.such.key"), "no.such.key");
    }
}
