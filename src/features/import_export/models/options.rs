//! Import options, source formats and errors.

/// File formats the importer understands.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SourceFormat {
    /// IFC in STEP encoding (IFC2x3 and IFC4).
    Ifc,
    /// Wavefront OBJ with optional MTL materials.
    Obj,
    /// glTF 2.0 JSON or binary GLB.
    Gltf,
    /// Autodesk FBX, binary encoding.
    Fbx,
}

impl SourceFormat {
    /// Short display name.
    pub fn name(self) -> &'static str {
        match self {
            SourceFormat::Ifc => "IFC",
            SourceFormat::Obj => "OBJ",
            SourceFormat::Gltf => "glTF",
            SourceFormat::Fbx => "FBX",
        }
    }
}

/// Options that matter for formats without explicit units or axes.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ImportOptions {
    /// The file's vertical axis is Y (converted to Z-up). Ignored by IFC, glTF and FBX, which declare their own.
    pub y_up: bool,
    /// Meters per file unit for OBJ. Ignored by formats that declare units.
    pub unit_scale: f64,
}

impl Default for ImportOptions {
    fn default() -> Self {
        Self {
            y_up: true,
            unit_scale: 1.0,
        }
    }
}

/// Why an import did not produce a scene.
#[derive(Debug, Clone, PartialEq)]
pub enum ImportError {
    /// The user cancelled.
    Cancelled,
    /// The file could not be read.
    Io(String),
    /// The file content is not valid for its format.
    Invalid(String),
    /// The importer crashed internally; the application keeps running.
    Crashed(String),
}

impl ImportError {
    /// Localization key of the error headline.
    pub fn key(&self) -> &'static str {
        match self {
            ImportError::Cancelled => "import.err_cancelled",
            ImportError::Io(_) => "import.err_io",
            ImportError::Invalid(_) => "import.err_invalid",
            ImportError::Crashed(_) => "import.err_crashed",
        }
    }

    /// Technical detail, empty when there is none.
    pub fn detail(&self) -> &str {
        match self {
            ImportError::Cancelled => "",
            ImportError::Io(text) | ImportError::Invalid(text) | ImportError::Crashed(text) => text,
        }
    }
}

impl From<std::io::Error> for ImportError {
    fn from(error: std::io::Error) -> Self {
        ImportError::Io(error.to_string())
    }
}
