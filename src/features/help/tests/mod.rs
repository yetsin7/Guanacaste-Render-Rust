//! Unit tests of the help domain.

use super::logic::controls;
use super::logic::*;

#[test]
fn shortcut_search_checks_action_and_keys() {
    let list = controls::objects();
    assert_eq!(filter_shortcuts(&list, "deshacer").len(), 1);
    assert_eq!(filter_shortcuts(&list, "supr").len(), 1);
    assert_eq!(filter_shortcuts(&list, "").len(), list.len());
    assert!(filter_shortcuts(&list, "zzz").is_empty());
}

#[test]
fn the_controls_name_the_keys_the_app_really_uses() {
    let all: Vec<_> = controls::navigation()
        .into_iter()
        .chain(controls::cameras())
        .chain(controls::application())
        .collect();
    for needle in [
        "W A S D",
        "E / Q",
        "Clic derecho",
        "Mayús",
        "F / Inicio",
        "1 - 9",
        "F1",
        "Ctrl + S",
    ] {
        assert!(
            all.iter().any(|s| s.keys.contains(needle)),
            "no row for {needle}"
        );
    }
}
