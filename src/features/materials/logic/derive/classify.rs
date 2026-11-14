//! Which family of surface a material is: keywords of the material and type names (Spanish and English), then the
//! IFC class of the element when the names say nothing.

use super::text::{fold, has};
use super::{SurfaceInfo, SurfaceKind};

/// Pipes and ducts hidden inside walls and ceilings.
const HIDDEN_CLASSES: [&str; 2] = ["IFCFLOWSEGMENT", "IFCFLOWFITTING"];
const SOIL: [&str; 7] = [
    "tierra",
    "relleno",
    "terreno",
    "topograf",
    "talud",
    "soil",
    "embankment",
];
const GRASS: [&str; 6] = ["grama", "cesped", "pasto", "grass", "lawn", "turf"];
const GLASS: [&str; 5] = ["vidrio", "glass", "cristal", "espejo", "mirror"];
const ROOF: [&str; 6] = ["teja", "cubierta", "techo", "roof", "shingle", "tejado"];
const STONE: [&str; 9] = [
    "piedra", "cantera", "granit", "marmol", "marble", "adoquin", "laja", "slate", "stone",
];
const WOOD: [&str; 14] = [
    "madera", "wood", "pino", "pine", "walnut", "nogal", "cedro", "caoba", "roble", "oak",
    "timber", "parquet", "teca", "teak",
];
const STUCCO: [&str; 8] = [
    "estuco", "repello", "revoque", "panete", "stucco", "enluc", "mortero", "plaster",
];
const BRICK: [&str; 6] = ["ladrillo", "brick", "bloque", "block", "cmu", "masonry"];
const PLASTER: [&str; 8] = [
    "gypsum",
    "yeso",
    "drywall",
    "tablero de muro",
    "cielo",
    "ceiling",
    "plafon",
    "cielorraso",
];
const TRIM: [&str; 4] = ["rodapie", "zocalo", "baseboard", "skirting"];
const CERAMIC: [&str; 9] = [
    "ceram",
    "azulej",
    "porcel",
    "baldosa",
    "tile",
    "mosaic",
    "sanitari",
    "elemento de bano",
    "toilet",
];
const METAL: [&str; 15] = [
    "acero",
    "steel",
    "metal",
    "alumin",
    "inox",
    "stainless",
    "iron",
    "hierro",
    "brass",
    "laton",
    "cromo",
    "chrome",
    "zinc",
    "cobre",
    "copper",
];
const CONCRETE: [&str; 5] = ["hormig", "concrete", "cascote", "cemento", "precast"];
const FABRIC: [&str; 6] = ["textil", "fabric", "alfombra", "tela", "carpet", "cuero"];
const PLASTIC: [&str; 6] = [
    "plastic", "pvc", "laccato", "laminado", "acrilic", "laminate",
];
const PAINT: [&str; 9] = [
    "pintura",
    "paint",
    "peinture",
    "revestimiento",
    "jamba",
    "fascia",
    "predeterminado",
    "default",
    "ultrex",
];

fn is_wall(class: &str) -> bool {
    class.starts_with("IFCWALL")
}

/// The family of the surface `info` describes.
pub fn classify(info: &SurfaceInfo) -> SurfaceKind {
    let class = info.class.to_ascii_uppercase();
    let class = class.as_str();
    let names = format!("{} {}", fold(info.name), fold(info.type_name.unwrap_or("")));
    let everything = format!("{names} {}", fold(info.element.unwrap_or("")));
    if HIDDEN_CLASSES.contains(&class)
        || has(&everything, &["tuberia", "pipe", "conduit", "ducto"])
        || info.color[3] <= 0.02
    {
        return SurfaceKind::Hidden;
    }
    if has(&everything, &SOIL) && !has(&names, &["cascote"]) {
        return SurfaceKind::Soil;
    }
    if info.color[3] < 0.98 && !has(&names, &["rejilla"]) {
        return SurfaceKind::Glass;
    }
    by_name(&names, class).unwrap_or_else(|| by_class(class))
}

fn by_name(names: &str, class: &str) -> Option<SurfaceKind> {
    use SurfaceKind::*;
    let names = names.replace("stonewhite", "");
    let names = names.as_str();
    if has(names, &GRASS) {
        return Some(Grass);
    }
    if has(names, &GLASS) {
        return Some(Glass);
    }
    if has(names, &ROOF) || class == "IFCROOF" {
        return Some(RoofTile);
    }
    if has(names, &STONE) {
        return Some(Stone);
    }
    if has(names, &WOOD) {
        return Some(Wood);
    }
    if has(names, &STUCCO) {
        return Some(Stucco);
    }
    if has(names, &BRICK) {
        return Some(if is_wall(class) { Stucco } else { Brick });
    }
    if has(names, &PLASTER) {
        return Some(Plaster);
    }
    if has(names, &TRIM) {
        return Some(Paint);
    }
    if has(names, &FABRIC) {
        return Some(Fabric);
    }
    if has(names, &CERAMIC) {
        return Some(Ceramic);
    }
    if has(names, &METAL) {
        return Some(Metal);
    }
    if has(names, &CONCRETE) {
        return Some(Concrete);
    }
    if has(names, &PLASTIC) {
        return Some(Plastic);
    }
    if has(names, &PAINT) {
        return Some(match class {
            c if is_wall(c) || c == "IFCCOLUMN" => Stucco,
            "IFCCOVERING" => Plaster,
            "IFCSLAB" => Concrete,
            _ => Paint,
        });
    }
    None
}

fn by_class(class: &str) -> SurfaceKind {
    use SurfaceKind::*;
    match class {
        c if is_wall(c) => Stucco,
        "IFCCOLUMN" | "IFCBEAM" | "IFCMEMBER" => Stucco,
        "IFCSLAB" | "IFCSTAIR" | "IFCSTAIRFLIGHT" | "IFCFOOTING" => Concrete,
        "IFCCOVERING" => Plaster,
        "IFCDOOR" => Wood,
        "IFCWINDOW" | "IFCPLATE" | "IFCCURTAINWALL" => Paint,
        "IFCRAILING" => Metal,
        "IFCFLOWTERMINAL" => Ceramic,
        _ => Plastic,
    }
}
