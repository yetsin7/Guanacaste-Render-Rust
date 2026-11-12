//! Sample vegetation catalog (placeholder data).

use crate::features::library::models::{
    AssetArt, AssetBadge, CanopyKind, LibraryAsset, Taxa, TreeStyle,
};

/// Number of assets reported by the catalog header.
pub const TOTAL_ASSETS: u32 = 348;
/// Number of pages offered by the pagination control.
pub const PAGES: usize = 5;
/// Counts shown on the taxa chips, in [`Taxa::ALL`] order.
pub const TAXA_COUNTS: [&str; 6] = ["124", "86", "32", "54", "48", "67"];
/// Number of filters shown in the search box badge.
pub const ACTIVE_FILTERS: &str = "3";
/// Instances placed in the scene so far.
pub const PLACED_INSTANCES: &str = "1,248";

fn tree(kind: CanopyKind, leaves: [u32; 2], trunk: u32) -> AssetArt {
    AssetArt::Tree(TreeStyle {
        kind,
        leaves,
        trunk,
    })
}

/// The catalog page shown by the library.
pub fn assets() -> Vec<LibraryAsset> {
    let asset = |name,
                 species,
                 rating,
                 height_m,
                 volume,
                 stat: (&'static str, &'static str),
                 taxa,
                 badge,
                 scale,
                 art| LibraryAsset {
        name,
        species,
        rating,
        height_m,
        volume,
        stat_label: stat.0,
        stat_value: stat.1,
        taxa,
        badge,
        scale,
        art,
    };
    vec![
        asset(
            "Olmo Americano 004",
            "Ulmus americana, árbol de sombra maduro",
            4.9,
            18.5,
            100,
            ("Malla LOD", "142k Poly"),
            Taxa::Broadleaf,
            AssetBadge::Hd,
            "Escala CAD 1:1",
            AssetArt::ElmRender,
        ),
        asset(
            "Arce Azucarero 012",
            "Acer saccharum, follaje otoñal",
            4.8,
            14.2,
            92,
            ("Anim. viento", "Activa 60fps"),
            Taxa::Broadleaf,
            AssetBadge::Hd,
            "Escala 1.0",
            tree(CanopyKind::Round, [0xe0922b, 0xf0b44a], 0x3a2c22),
        ),
        asset(
            "Haya Europea 008",
            "Fagus sylvatica, dosel denso",
            4.7,
            21.0,
            88,
            ("Subsuperficie", "SSS Trans"),
            Taxa::Broadleaf,
            AssetBadge::Hd,
            "Escala 1.0",
            tree(CanopyKind::Round, [0x2f5a35, 0x44784a], 0xb9b8b0),
        ),
        asset(
            "Abedul Plateado 021",
            "Betula pendula, ejemplar esbelto",
            4.9,
            12.0,
            81,
            ("Sombreado", "PBR 8K"),
            Taxa::Broadleaf,
            AssetBadge::New,
            "Escala 1.0",
            tree(CanopyKind::Round, [0x8fae7c, 0xb7cfa0], 0xe8e6df),
        ),
        asset(
            "Roble Blanco 003",
            "Quercus alba, árbol patrimonial",
            5.0,
            24.5,
            76,
            ("Colisión", "Precisa"),
            Taxa::Broadleaf,
            AssetBadge::Hd,
            "Escala 1.0",
            tree(CanopyKind::Round, [0x5b6458, 0x7a8473], 0x4a4036),
        ),
        asset(
            "Arce Rojo 019",
            "Acer rubrum, ejemplar de humedal",
            4.8,
            15.8,
            66,
            ("Variaciones", "4 Estaciones"),
            Taxa::Broadleaf,
            AssetBadge::Hd,
            "Escala 1.0",
            tree(CanopyKind::Round, [0x7a1630, 0xa8263f], 0x2c2326),
        ),
        asset(
            "Pino Silvestre 005",
            "Pinus sylvestris, corteza rugosa",
            4.6,
            16.0,
            52,
            ("Agujas", "Trazadas"),
            Taxa::Conifers,
            AssetBadge::Hd,
            "Escala 1.0",
            tree(CanopyKind::Conifer, [0x3f6a6a, 0x5d8c86], 0xb5703c),
        ),
        asset(
            "Cerezo Japonés 014",
            "Prunus serrulata, flor primaveral",
            4.9,
            9.5,
            41,
            ("Dispersión de pétalos", "GPU PhysX"),
            Taxa::Broadleaf,
            AssetBadge::Hd,
            "Escala 1.0",
            tree(CanopyKind::Blossom, [0xe8b4c4, 0xf6d6df], 0x4a3a38),
        ),
    ]
}
