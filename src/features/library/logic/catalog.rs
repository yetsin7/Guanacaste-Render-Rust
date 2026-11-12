//! Catalog rules: filtering and sorting.

use crate::features::library::models::{LibraryAsset, LibraryState, SortKey, Taxa};

/// Whether `asset` passes the search text and, when applied, the taxa filter.
pub fn matches(asset: &LibraryAsset, state: &LibraryState) -> bool {
    let needle = state.search.trim().to_lowercase();
    let text_ok = needle.is_empty()
        || asset.name.to_lowercase().contains(&needle)
        || asset.species.to_lowercase().contains(&needle);
    let taxa_ok =
        !state.taxa_applied || state.taxa == Taxa::NorthAmerica || asset.taxa == state.taxa;
    text_ok && taxa_ok
}

/// Indices into `assets` of the visible cards, in the selected sort order.
pub fn visible(assets: &[LibraryAsset], state: &LibraryState) -> Vec<usize> {
    let mut indices: Vec<usize> = (0..assets.len())
        .filter(|&index| matches(&assets[index], state))
        .collect();
    match state.sort {
        SortKey::CanopyVolume => {
            indices.sort_by_key(|&index| std::cmp::Reverse(assets[index].volume))
        }
        SortKey::Name => indices.sort_by_key(|&index| assets[index].name),
        SortKey::Rating => indices.sort_by(|&a, &b| assets[b].rating.total_cmp(&assets[a].rating)),
        SortKey::Height => {
            indices.sort_by(|&a, &b| assets[b].height_m.total_cmp(&assets[a].height_m))
        }
    }
    indices
}

/// Selects `taxa`, applying the filter; picking the applied filter again clears it.
pub fn toggle_taxa(state: &mut LibraryState, taxa: Taxa) {
    if state.taxa_applied && state.taxa == taxa {
        state.taxa_applied = false;
    } else {
        state.taxa = taxa;
        state.taxa_applied = true;
    }
}
