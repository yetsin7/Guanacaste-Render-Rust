//! Recognizing files by extension: supported formats and the proprietary ones that cannot be read.

use crate::features::import_export::models::SourceFormat;
use std::path::Path;

/// Proprietary formats that need the authoring software to be read.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Proprietary {
    /// Revit project, family or template (.rvt, .rfa, .rte, .rft).
    Revit,
    /// AutoCAD drawing (.dwg).
    Dwg,
    /// Navisworks model (.nwd, .nwc).
    Navisworks,
    /// Compressed IFC (.ifczip) or IFC XML (.ifcxml): readable after extracting or re-exporting.
    IfcVariant,
    /// SketchUp model (.skp): closed format, read only through Trimble's licensed SDK.
    SketchUp,
    /// 3ds Max scene (.max).
    Max,
    /// Rhino model (.3dm).
    Rhino,
    /// Blender scene (.blend).
    Blender,
    /// ArchiCAD project (.pln, .pla, .tpl).
    ArchiCad,
    /// CAD exchange solids (STEP / IGES): need a geometry kernel to tessellate.
    CadExchange,
}

impl Proprietary {
    /// Localization key of the explanation shown to the user.
    pub fn explanation_key(self) -> &'static str {
        match self {
            Proprietary::Revit => "import.how_revit",
            Proprietary::Dwg => "import.how_dwg",
            Proprietary::Navisworks => "import.how_navis",
            Proprietary::IfcVariant => "import.how_ifc_variant",
            Proprietary::SketchUp => "import.how_skp",
            Proprietary::Max => "import.how_max",
            Proprietary::Rhino => "import.how_rhino",
            Proprietary::Blender => "import.how_blender",
            Proprietary::ArchiCad => "import.how_archicad",
            Proprietary::CadExchange => "import.how_step",
        }
    }
}

/// What a file name means for the importer.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Detected {
    /// A format this application can read.
    Supported(SourceFormat),
    /// A format that cannot be read directly: the user must export it from its program.
    Proprietary(Proprietary),
    /// Anything else.
    Unknown,
}

/// Classifies `path` by its extension (case-insensitive).
pub fn detect(path: &Path) -> Detected {
    let extension = path
        .extension()
        .and_then(|text| text.to_str())
        .unwrap_or("")
        .to_ascii_lowercase();
    match extension.as_str() {
        "ifc" => Detected::Supported(SourceFormat::Ifc),
        "obj" => Detected::Supported(SourceFormat::Obj),
        "gltf" | "glb" => Detected::Supported(SourceFormat::Gltf),
        "fbx" => Detected::Supported(SourceFormat::Fbx),
        "gr" | "g3d" => Detected::Supported(SourceFormat::Legacy),
        "stl" => Detected::Supported(SourceFormat::Stl),
        "ply" => Detected::Supported(SourceFormat::Ply),
        "dae" => Detected::Supported(SourceFormat::Collada),
        "3ds" => Detected::Supported(SourceFormat::ThreeDs),
        "rvt" | "rfa" | "rte" | "rft" => Detected::Proprietary(Proprietary::Revit),
        "dwg" | "dxf" => Detected::Proprietary(Proprietary::Dwg),
        "skp" | "skb" => Detected::Proprietary(Proprietary::SketchUp),
        "max" => Detected::Proprietary(Proprietary::Max),
        "3dm" => Detected::Proprietary(Proprietary::Rhino),
        "blend" => Detected::Proprietary(Proprietary::Blender),
        "pln" | "pla" | "tpl" => Detected::Proprietary(Proprietary::ArchiCad),
        "step" | "stp" | "iges" | "igs" => Detected::Proprietary(Proprietary::CadExchange),
        "nwd" | "nwc" => Detected::Proprietary(Proprietary::Navisworks),
        "ifczip" | "ifcxml" => Detected::Proprietary(Proprietary::IfcVariant),
        _ => Detected::Unknown,
    }
}

/// File extensions offered by the file picker.
pub const PICKER_EXTENSIONS: [&str; 11] = [
    "ifc", "obj", "gltf", "glb", "fbx", "dae", "stl", "ply", "3ds", "gr", "g3d",
];

/// Extensions of closed formats the picker also lists so the user gets the conversion guide.
pub const GUIDE_EXTENSIONS: [&str; 15] = [
    "skp", "rvt", "rfa", "rte", "rft", "dwg", "dxf", "nwd", "nwc", "max", "3dm", "blend", "pln",
    "step", "stp",
];

/// Signature of a Guanacaste Render project (`.gr`).
pub const RENDER_SIGNATURE: &[u8; 8] = b"GRRPROJ1";
/// Signature of a Guanacaste 3D project (also `.gr`, read-only here).
pub const LEGACY_SIGNATURE: &[u8; 8] = b"G3DNATV1";

/// What the first eight bytes of a `.gr` file say it is. The extension is never trusted.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Signature {
    /// A Guanacaste Render project (`GRRPROJ1`).
    Render,
    /// A Guanacaste 3D project (`G3DNATV1`).
    Legacy,
    /// Anything else, including files shorter than eight bytes.
    Unknown,
}

/// Classifies the first bytes of a file.
pub fn signature_of(head: &[u8]) -> Signature {
    match head.get(..8) {
        Some(bytes) if bytes == RENDER_SIGNATURE => Signature::Render,
        Some(bytes) if bytes == LEGACY_SIGNATURE => Signature::Legacy,
        _ => Signature::Unknown,
    }
}

/// Reads the signature of the file at `path`.
pub fn sniff(path: &Path) -> std::io::Result<Signature> {
    use std::io::Read;
    let mut head = Vec::with_capacity(8);
    std::fs::File::open(path)?.take(8).read_to_end(&mut head)?;
    Ok(signature_of(&head))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn supported_and_proprietary_are_told_apart() {
        assert_eq!(
            detect(Path::new("A.IFC")),
            Detected::Supported(SourceFormat::Ifc)
        );
        assert_eq!(
            detect(Path::new("modelo.glb")),
            Detected::Supported(SourceFormat::Gltf)
        );
        assert_eq!(
            detect(Path::new("casa.rvt")),
            Detected::Proprietary(Proprietary::Revit)
        );
        assert_eq!(
            detect(Path::new("familia.RFA")),
            Detected::Proprietary(Proprietary::Revit)
        );
        assert_eq!(
            detect(Path::new("plano.dwg")),
            Detected::Proprietary(Proprietary::Dwg)
        );
        assert_eq!(
            detect(Path::new("Casa.SKP")),
            Detected::Proprietary(Proprietary::SketchUp)
        );
        assert_eq!(
            detect(Path::new("malla.stl")),
            Detected::Supported(SourceFormat::Stl)
        );
        assert_eq!(
            detect(Path::new("escena.DAE")),
            Detected::Supported(SourceFormat::Collada)
        );
        assert_eq!(
            detect(Path::new("viejo.3ds")),
            Detected::Supported(SourceFormat::ThreeDs)
        );
        assert_eq!(detect(Path::new("notas.txt")), Detected::Unknown);
        assert_eq!(detect(Path::new("sin_extension")), Detected::Unknown);
    }
}
