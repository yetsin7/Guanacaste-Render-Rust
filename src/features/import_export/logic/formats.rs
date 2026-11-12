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
}

impl Proprietary {
    /// Localization key of the explanation shown to the user.
    pub fn explanation_key(self) -> &'static str {
        match self {
            Proprietary::Revit => "import.how_revit",
            Proprietary::Dwg => "import.how_dwg",
            Proprietary::Navisworks => "import.how_navis",
            Proprietary::IfcVariant => "import.how_ifc_variant",
        }
    }
}

/// What a file name means for the importer.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Detected {
    /// A format this application can read.
    Supported(SourceFormat),
    /// A format that cannot be read without Autodesk software.
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
        "rvt" | "rfa" | "rte" | "rft" => Detected::Proprietary(Proprietary::Revit),
        "dwg" => Detected::Proprietary(Proprietary::Dwg),
        "nwd" | "nwc" => Detected::Proprietary(Proprietary::Navisworks),
        "ifczip" | "ifcxml" => Detected::Proprietary(Proprietary::IfcVariant),
        _ => Detected::Unknown,
    }
}

/// File extensions offered by the file picker.
pub const PICKER_EXTENSIONS: [&str; 5] = ["ifc", "obj", "gltf", "glb", "fbx"];

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
        assert_eq!(detect(Path::new("notas.txt")), Detected::Unknown);
        assert_eq!(detect(Path::new("sin_extension")), Detected::Unknown);
    }
}
