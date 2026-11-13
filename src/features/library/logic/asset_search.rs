//! Object library listing: procedural kinds plus imported models, filtered by category and accent-insensitive search.

use crate::features::library::models::{AssetCategory, AssetRef, ImportedAssetDoc, ProceduralKind};
use crate::services::localization::tr;

/// One entry of the object library.
#[derive(Debug, Clone, PartialEq)]
pub struct AssetEntry {
    /// The asset (procedural entries use seed 0; the panel applies the chosen seed).
    pub asset: AssetRef,
    /// Display name.
    pub name: String,
    /// Category.
    pub category: AssetCategory,
}

/// Lowercases `text` and removes Spanish accents so "arbol" finds "Árbol".
pub fn fold(text: &str) -> String {
    text.to_lowercase()
        .chars()
        .map(|c| match c {
            'á' | 'à' | 'ä' => 'a',
            'é' | 'è' | 'ë' => 'e',
            'í' | 'ì' | 'ï' => 'i',
            'ó' | 'ò' | 'ö' => 'o',
            'ú' | 'ù' | 'ü' => 'u',
            other => other,
        })
        .collect()
}

/// Every library entry: the procedural kinds in display order, then the imported models.
pub fn entries(imported: &[ImportedAssetDoc]) -> Vec<AssetEntry> {
    let procedural = ProceduralKind::ALL.iter().map(|&kind| AssetEntry {
        asset: AssetRef::Procedural { kind, seed: 0 },
        name: tr(kind.key()).to_owned(),
        category: kind.category(),
    });
    let models = imported.iter().map(|model| AssetEntry {
        asset: AssetRef::Imported { id: model.id },
        name: model.name.clone(),
        category: AssetCategory::Imported,
    });
    procedural.chain(models).collect()
}

/// Entries of `category` (all when `None`) whose name or category name contains every word of `query`.
pub fn filter(
    entries: &[AssetEntry],
    query: &str,
    category: Option<AssetCategory>,
) -> Vec<AssetEntry> {
    let words: Vec<String> = fold(query).split_whitespace().map(str::to_owned).collect();
    entries
        .iter()
        .filter(|entry| category.is_none_or(|wanted| entry.category == wanted))
        .filter(|entry| {
            let haystack = fold(&format!("{} {}", entry.name, tr(entry.category.key())));
            words.iter().all(|word| haystack.contains(word.as_str()))
        })
        .cloned()
        .collect()
}
