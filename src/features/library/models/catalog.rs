//! Catalog entities and UI state.

/// Botanical groups of the taxa filter.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Taxa {
    /// Broadleaf trees.
    Broadleaf,
    /// Conifers.
    Conifers,
    /// Palms.
    Palms,
    /// Grasses.
    Grasses,
    /// Flowers.
    Flowers,
    /// Species native to North America (a region, not a group).
    NorthAmerica,
}

impl Taxa {
    /// Filters in display order.
    pub const ALL: [Taxa; 6] = [
        Taxa::Broadleaf,
        Taxa::Conifers,
        Taxa::Palms,
        Taxa::Grasses,
        Taxa::Flowers,
        Taxa::NorthAmerica,
    ];
}

/// Crown shape of a placeholder tree thumbnail.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CanopyKind {
    /// Round deciduous crown.
    Round,
    /// Stacked conical crown.
    Conifer,
    /// Wide crown of blossoms.
    Blossom,
}

/// Look of a procedurally painted tree thumbnail (a placeholder for the real asset preview).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TreeStyle {
    /// Crown shape.
    pub kind: CanopyKind,
    /// Two foliage colors as `0xRRGGBB`.
    pub leaves: [u32; 2],
    /// Trunk color as `0xRRGGBB`.
    pub trunk: u32,
}

/// What a catalog card shows as its picture.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AssetArt {
    /// The bundled elm render (placeholder).
    ElmRender,
    /// A painted tree.
    Tree(TreeStyle),
}

/// Quality or novelty badge of a card.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AssetBadge {
    /// High-definition asset.
    Hd,
    /// Recently added asset.
    New,
}

/// A vegetation asset of the catalog.
#[derive(Debug, Clone, PartialEq)]
pub struct LibraryAsset {
    /// Display name.
    pub name: &'static str,
    /// Botanical name and short description.
    pub species: &'static str,
    /// Average user rating, 0..=5.
    pub rating: f32,
    /// Height in meters.
    pub height_m: f32,
    /// Relative canopy volume, used for sorting.
    pub volume: u32,
    /// Caption of the second technical readout.
    pub stat_label: &'static str,
    /// Value of the second technical readout.
    pub stat_value: &'static str,
    /// Taxa group.
    pub taxa: Taxa,
    /// Badge over the picture.
    pub badge: AssetBadge,
    /// Scale label over the picture.
    pub scale: &'static str,
    /// Picture.
    pub art: AssetArt,
}

/// Sort orders of the catalog.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SortKey {
    /// Largest canopy first.
    #[default]
    CanopyVolume,
    /// Alphabetical.
    Name,
    /// Best rated first.
    Rating,
    /// Tallest first.
    Height,
}

impl SortKey {
    /// The next order in the cycle.
    pub fn next(self) -> Self {
        match self {
            SortKey::CanopyVolume => SortKey::Name,
            SortKey::Name => SortKey::Rating,
            SortKey::Rating => SortKey::Height,
            SortKey::Height => SortKey::CanopyVolume,
        }
    }
}

/// UI state of the library screen.
#[derive(Debug, Clone, PartialEq)]
pub struct LibraryState {
    /// Search text.
    pub search: String,
    /// Highlighted taxa filter.
    pub taxa: Taxa,
    /// False until the user picks a filter: the default view shows the whole page with the preset highlighted.
    pub taxa_applied: bool,
    /// Sort order.
    pub sort: SortKey,
    /// Layout toggle (0 grid, 1 list).
    pub layout: usize,
    /// Index of the selected asset in the catalog.
    pub selected: usize,
    /// Current page, 0-based.
    pub page: usize,
}

impl Default for LibraryState {
    fn default() -> Self {
        Self {
            search: String::new(),
            taxa: Taxa::Broadleaf,
            taxa_applied: false,
            sort: SortKey::default(),
            layout: 0,
            selected: 0,
            page: 0,
        }
    }
}
