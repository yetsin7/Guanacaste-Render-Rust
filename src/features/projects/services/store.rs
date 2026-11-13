//! Saving and loading projects: atomic writes with verification and backup, limits, migrations and read-only
//! handling of newer schemas.

use super::container::{
    self, BlockSource, ContainerError, Limits, Reader, KIND_MESH, KIND_THUMBNAIL,
};
use super::util;
use crate::features::import_export::logic::formats::{sniff, Signature};
use crate::features::import_export::models::Mesh;
use crate::features::projects::logic::migrations::{migrate, MIGRATIONS};
use crate::features::projects::models::{Project, ProjectDocument, SCHEMA_VERSION};
use std::borrow::Cow;
use std::fs::File;
use std::path::{Path, PathBuf};

/// Why saving or loading failed.
#[derive(Debug, Clone, PartialEq)]
pub enum StoreError {
    /// Container-level problem.
    Container(ContainerError),
    /// The JSON document could not be read or written.
    Document(String),
    /// A mesh block does not match the document.
    Mismatch(&'static str),
    /// The destination is a Guanacaste 3D project, which is never overwritten.
    LegacyTarget,
    /// The project came from a newer schema and is read-only.
    ReadOnly,
}

impl From<ContainerError> for StoreError {
    fn from(error: ContainerError) -> Self {
        StoreError::Container(error)
    }
}

impl From<std::io::Error> for StoreError {
    fn from(error: std::io::Error) -> Self {
        StoreError::Container(ContainerError::from(error))
    }
}

impl StoreError {
    /// Localization key of the headline.
    pub fn key(&self) -> &'static str {
        match self {
            StoreError::Container(ContainerError::Io(_)) => "store.err_io",
            StoreError::Container(ContainerError::NotAProject) => "store.err_not_project",
            StoreError::Container(ContainerError::Unsupported(_)) => "store.err_unsupported",
            StoreError::Container(ContainerError::Damaged(_)) | StoreError::Mismatch(_) => {
                "store.err_damaged"
            }
            StoreError::Container(ContainerError::TooLarge(_)) => "store.err_too_large",
            StoreError::Document(_) => "store.err_document",
            StoreError::LegacyTarget => "store.err_legacy_target",
            StoreError::ReadOnly => "store.err_read_only",
        }
    }

    /// Technical detail.
    pub fn detail(&self) -> String {
        match self {
            StoreError::Container(ContainerError::Io(text)) | StoreError::Document(text) => {
                text.clone()
            }
            StoreError::Container(
                ContainerError::Unsupported(text)
                | ContainerError::Damaged(text)
                | ContainerError::TooLarge(text),
            )
            | StoreError::Mismatch(text) => (*text).to_owned(),
            StoreError::Container(ContainerError::NotAProject)
            | StoreError::LegacyTarget
            | StoreError::ReadOnly => String::new(),
        }
    }
}

/// Encodes a mesh as `u32 vertices, u32 indices, f32 x3 per vertex, u32 per index` (little endian).
pub fn encode_mesh(mesh: &Mesh) -> Vec<u8> {
    let mut out = Vec::with_capacity(8 + mesh.positions.len() * 12 + mesh.indices.len() * 4);
    out.extend_from_slice(&(mesh.positions.len() as u32).to_le_bytes());
    out.extend_from_slice(&(mesh.indices.len() as u32).to_le_bytes());
    for position in &mesh.positions {
        for value in position {
            out.extend_from_slice(&value.to_le_bytes());
        }
    }
    for index in &mesh.indices {
        out.extend_from_slice(&index.to_le_bytes());
    }
    out
}

/// Decodes [`encode_mesh`] output, checking that the lengths agree with `vertices` and `indices`.
pub fn decode_mesh(raw: &[u8], vertices: u32, indices: u32) -> Result<Mesh, StoreError> {
    let count = |at: usize| u32::from_le_bytes([raw[at], raw[at + 1], raw[at + 2], raw[at + 3]]);
    if raw.len() < 8
        || count(0) != vertices
        || count(4) != indices
        || raw.len() != 8 + vertices as usize * 12 + indices as usize * 4
    {
        return Err(StoreError::Mismatch(
            "un bloque de malla no coincide con el documento",
        ));
    }
    let float = |at: usize| f32::from_le_bytes([raw[at], raw[at + 1], raw[at + 2], raw[at + 3]]);
    let positions = (0..vertices as usize)
        .map(|v| [float(8 + v * 12), float(12 + v * 12), float(16 + v * 12)])
        .collect();
    let base = 8 + vertices as usize * 12;
    let index_list: Vec<u32> = (0..indices as usize).map(|i| count(base + i * 4)).collect();
    if index_list.iter().any(|i| *i >= vertices) {
        return Err(StoreError::Mismatch(
            "un índice de malla apunta fuera de los vértices",
        ));
    }
    Ok(Mesh {
        positions,
        indices: index_list,
        material: None,
    })
}

/// Result of a save.
#[derive(Debug, Clone, PartialEq)]
pub struct SaveStats {
    /// Size of the written file.
    pub bytes: u64,
    /// Time taken in milliseconds.
    pub millis: u128,
    /// Whether the previous version was kept as `.bak`.
    pub backed_up: bool,
}

fn sibling(path: &Path, suffix: &str) -> PathBuf {
    let mut name = path
        .file_name()
        .map(|n| n.to_os_string())
        .unwrap_or_default();
    name.push(suffix);
    path.with_file_name(name)
}

fn write_file(path: &Path, project: &Project, thumbnail: Option<&[u8]>) -> Result<(), StoreError> {
    let json =
        serde_json::to_vec(&project.document).map_err(|e| StoreError::Document(e.to_string()))?;
    let mut file = File::create(path)?;
    let mut blocks = project
        .meshes
        .iter()
        .enumerate()
        .map(|(index, mesh)| BlockSource {
            kind: KIND_MESH,
            id: index as u32,
            raw: Cow::Owned(encode_mesh(mesh)),
        })
        .chain(thumbnail.map(|png| BlockSource {
            kind: KIND_THUMBNAIL,
            id: 0,
            raw: Cow::Borrowed(png),
        }));
    container::write(&mut file, SCHEMA_VERSION, project.id, &json, &mut blocks)?;
    file.sync_all()?;
    Ok(())
}

fn verify(path: &Path) -> Result<(), StoreError> {
    let mut reader = Reader::open(path, Limits::default())?;
    for index in 0..reader.blocks.len() {
        reader.read_block(index)?;
    }
    Ok(())
}

/// Saves `project` to `path` atomically: write a temporary file, verify it by reading it back, keep the previous
/// version as `<name>.bak`, then replace. A failure leaves the original file untouched.
/// Refuses to save over a Guanacaste 3D file (detected by its signature, whatever its extension).
pub fn save(
    path: &Path,
    project: &Project,
    thumbnail: Option<&[u8]>,
) -> Result<SaveStats, StoreError> {
    if project.read_only {
        return Err(StoreError::ReadOnly);
    }
    if path.exists() && sniff(path)? == Signature::Legacy {
        return Err(StoreError::LegacyTarget);
    }
    let started = std::time::Instant::now();
    let temporary = sibling(path, ".tmp");
    let result = write_file(&temporary, project, thumbnail).and_then(|()| verify(&temporary));
    if let Err(error) = result {
        let _ = std::fs::remove_file(&temporary);
        return Err(error);
    }
    let backed_up = path.exists() && std::fs::copy(path, sibling(path, ".bak")).is_ok();
    if let Err(error) = std::fs::rename(&temporary, path) {
        let _ = std::fs::remove_file(&temporary);
        return Err(error.into());
    }
    Ok(SaveStats {
        bytes: std::fs::metadata(path)?.len(),
        millis: started.elapsed().as_millis(),
        backed_up,
    })
}

/// Something worth telling the user after a successful load.
#[derive(Debug, Clone, PartialEq)]
pub enum LoadWarning {
    /// The file uses a newer schema; it is open read-only.
    NewerSchema {
        /// Schema found in the file.
        found: u32,
        /// Newest schema this version supports.
        supported: u32,
    },
    /// The document was upgraded from an older schema in memory.
    Migrated {
        /// Schema found in the file.
        from: u32,
    },
}

/// A loaded project.
#[derive(Debug, Clone, PartialEq)]
pub struct Loaded {
    /// The project (read-only when from a newer schema).
    pub project: Project,
    /// Notes for the user.
    pub warnings: Vec<LoadWarning>,
    /// PNG thumbnail, when the file has one.
    pub thumbnail: Option<Vec<u8>>,
}

fn parse_document(
    json: &[u8],
    found: u32,
) -> Result<(ProjectDocument, Vec<LoadWarning>), StoreError> {
    let mut value: serde_json::Value =
        serde_json::from_slice(json).map_err(|e| StoreError::Document(e.to_string()))?;
    let mut warnings = Vec::new();
    if found > SCHEMA_VERSION {
        warnings.push(LoadWarning::NewerSchema {
            found,
            supported: SCHEMA_VERSION,
        });
    } else if found < SCHEMA_VERSION {
        migrate(&mut value, found, SCHEMA_VERSION, MIGRATIONS).map_err(StoreError::Document)?;
        warnings.push(LoadWarning::Migrated { from: found });
    }
    let document =
        serde_json::from_value(value).map_err(|e| StoreError::Document(e.to_string()))?;
    Ok((document, warnings))
}

/// Loads the project at `path` with the given limits.
pub fn load(path: &Path, limits: Limits) -> Result<Loaded, StoreError> {
    let mut reader = Reader::open(path, limits)?;
    let (document, warnings) = parse_document(&reader.json, reader.header.schema_version)?;
    let mut meshes = Vec::with_capacity(document.meshes.len());
    for (index, meta) in document.meshes.iter().enumerate() {
        let block = reader
            .find(KIND_MESH, index as u32)
            .ok_or(StoreError::Mismatch("falta el bloque de una malla"))?;
        let raw = reader.read_block(block)?;
        let mut mesh = decode_mesh(&raw, meta.vertices, meta.indices)?;
        mesh.material = meta.material.map(|m| m as usize);
        meshes.push(mesh);
    }
    let thumbnail = reader
        .find(KIND_THUMBNAIL, 0)
        .map(|i| reader.read_block(i))
        .transpose()?;
    let read_only = warnings
        .iter()
        .any(|w| matches!(w, LoadWarning::NewerSchema { .. }));
    let project = Project {
        id: reader.header.document_id,
        document,
        meshes,
        path: Some(path.to_path_buf()),
        legacy_source: None,
        read_only,
        dirty: false,
    };
    Ok(Loaded {
        project,
        warnings,
        thumbnail,
    })
}

/// Light information about a project file, read without loading any geometry.
#[derive(Debug, Clone, PartialEq)]
pub struct Peek {
    /// Project name.
    pub name: String,
    /// Schema version of the file.
    pub schema_version: u32,
    /// Thumbnail PNG bytes, when present.
    pub thumbnail: Option<Vec<u8>>,
}

/// Reads only the document header and the thumbnail of `path`.
pub fn peek(path: &Path) -> Result<Peek, StoreError> {
    let mut reader = Reader::open(path, Limits::default())?;
    let (document, _) = parse_document(&reader.json, reader.header.schema_version)?;
    let thumbnail = reader
        .find(KIND_THUMBNAIL, 0)
        .map(|i| reader.read_block(i))
        .transpose()?;
    Ok(Peek {
        name: document.meta.name,
        schema_version: reader.header.schema_version,
        thumbnail,
    })
}

/// A new empty project with the given name.
pub fn new_project(name: &str) -> Project {
    let now = util::now_unix();
    let mut document = ProjectDocument::default();
    document.meta.name = name.to_owned();
    document.meta.created = now;
    document.meta.modified = now;
    Project {
        id: util::new_document_id(),
        document,
        meshes: Vec::new(),
        path: None,
        legacy_source: None,
        read_only: false,
        dirty: true,
    }
}
