//! Original Guanacaste 3D built-in image maps, embedded so installed projects stay portable.

/// Image bytes for a built-in texture key; unknown and generated tree keys return `None`.
pub fn image(key: &str) -> Option<&'static [u8]> {
    match key {
        "ambientcg-bricks105" => Some(include_bytes!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/assets/materials/guanacaste-3d/ambientcg-bricks105.jpg"
        ))),
        "ambientcg-bricks113a" => Some(include_bytes!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/assets/materials/guanacaste-3d/ambientcg-bricks113a.jpg"
        ))),
        "ambientcg-concrete034" => Some(include_bytes!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/assets/materials/guanacaste-3d/ambientcg-concrete034.jpg"
        ))),
        "ambientcg-concrete046" => Some(include_bytes!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/assets/materials/guanacaste-3d/ambientcg-concrete046.jpg"
        ))),
        "ambientcg-fabric085" => Some(include_bytes!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/assets/materials/guanacaste-3d/ambientcg-fabric085.jpg"
        ))),
        "ambientcg-ground108" => Some(include_bytes!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/assets/materials/guanacaste-3d/ambientcg-ground108.jpg"
        ))),
        "ambientcg-marble012" => Some(include_bytes!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/assets/materials/guanacaste-3d/ambientcg-marble012.jpg"
        ))),
        "ambientcg-plaster001" => Some(include_bytes!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/assets/materials/guanacaste-3d/ambientcg-plaster001.jpg"
        ))),
        "ambientcg-tiles107" => Some(include_bytes!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/assets/materials/guanacaste-3d/ambientcg-tiles107.jpg"
        ))),
        "ambientcg-tiles141" => Some(include_bytes!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/assets/materials/guanacaste-3d/ambientcg-tiles141.jpg"
        ))),
        "ambientcg-wood051" => Some(include_bytes!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/assets/materials/guanacaste-3d/ambientcg-wood051.jpg"
        ))),
        "ambientcg-wood092" => Some(include_bytes!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/assets/materials/guanacaste-3d/ambientcg-wood092.jpg"
        ))),
        "ambientcg-wood096" => Some(include_bytes!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/assets/materials/guanacaste-3d/ambientcg-wood096.jpg"
        ))),
        "brick-red" => Some(include_bytes!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/assets/materials/guanacaste-3d/brick-red.png"
        ))),
        "concrete" => Some(include_bytes!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/assets/materials/guanacaste-3d/concrete.png"
        ))),
        "gravel" => Some(include_bytes!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/assets/materials/guanacaste-3d/gravel.png"
        ))),
        "stone-paving" => Some(include_bytes!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/assets/materials/guanacaste-3d/stone-paving.png"
        ))),
        "stucco-white" => Some(include_bytes!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/assets/materials/guanacaste-3d/stucco-white.png"
        ))),
        "wood-floor" => Some(include_bytes!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/assets/materials/guanacaste-3d/wood-floor.png"
        ))),
        _ => None,
    }
}
