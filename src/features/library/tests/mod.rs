//! Unit tests of the library domain.

use super::logic::*;
use super::models::*;
use crate::services::sample_data::library as sample;

#[test]
fn default_view_shows_every_sample_asset() {
    let assets = sample::assets();
    assert_eq!(
        visible(&assets, &LibraryState::default()).len(),
        assets.len()
    );
}

#[test]
fn applied_taxa_filter_excludes_other_groups() {
    let assets = sample::assets();
    let mut state = LibraryState::default();
    toggle_taxa(&mut state, Taxa::Conifers);
    let shown = visible(&assets, &state);
    assert!(!shown.is_empty());
    assert!(shown
        .iter()
        .all(|&index| assets[index].taxa == Taxa::Conifers));
    toggle_taxa(&mut state, Taxa::Conifers);
    assert_eq!(visible(&assets, &state).len(), assets.len());
}

#[test]
fn search_matches_name_and_species() {
    let assets = sample::assets();
    let state = LibraryState {
        search: "quercus".into(),
        ..Default::default()
    };
    let shown = visible(&assets, &state);
    assert_eq!(shown.len(), 1);
    assert_eq!(assets[shown[0]].name, "Roble Blanco 003");
}

#[test]
fn sorting_orders_the_cards() {
    let assets = sample::assets();
    let mut state = LibraryState {
        sort: SortKey::Height,
        ..Default::default()
    };
    let heights: Vec<f32> = visible(&assets, &state)
        .iter()
        .map(|&i| assets[i].height_m)
        .collect();
    assert!(heights.windows(2).all(|pair| pair[0] >= pair[1]));
    state.sort = SortKey::Name;
    let names: Vec<_> = visible(&assets, &state)
        .iter()
        .map(|&i| assets[i].name)
        .collect();
    assert!(names.windows(2).all(|pair| pair[0] <= pair[1]));
}

#[test]
fn sort_cycle_returns_to_start() {
    let mut key = SortKey::default();
    for _ in 0..4 {
        key = key.next();
    }
    assert_eq!(key, SortKey::default());
}
