//! The Guanacaste Render project container (`.gr` files with the `GRRPROJ1` signature). See `docs/project-format.md` for the exact layout.
//!
//! ```text
//! HEADER (96 bytes, little endian)
//!   0  8  "GRRPROJ1"
//!   8  2  container version (1)
//!  10  2  reserved
//!  12  4  schema version
//!  16 16  document id
//!  32  8  JSON length uncompressed
//!  40  8  JSON length compressed (gzip)
//!  48  4  block count
//!  52  4  reserved
//!  56 32  SHA-256 of the uncompressed JSON
//!  88  8  offset of the block index
//! JSON (gzip) | blocks (each gzip) | index (64 bytes per block)
//! ```

use sha2::{Digest, Sha256};
use std::borrow::Cow;
use std::fs::File;
use std::io::{Read, Seek, SeekFrom, Write};
use std::path::Path;

/// File signature.
pub const SIGNATURE: &[u8; 8] = crate::features::import_export::logic::formats::RENDER_SIGNATURE;
/// Version of the container layout.
pub const CONTAINER_VERSION: u16 = 1;
/// Size of the header in bytes.
pub const HEADER_BYTES: u64 = 96;
const INDEX_ENTRY_BYTES: u64 = 64;

/// Block kind: a mesh (vertices and indices).
pub const KIND_MESH: u32 = 1;
/// Block kind: a PNG thumbnail.
pub const KIND_THUMBNAIL: u32 = 2;

/// Why a container could not be read or written.
#[derive(Debug, Clone, PartialEq)]
pub enum ContainerError {
    /// Operating-system error.
    Io(String),
    /// The file does not start with the project signature.
    NotAProject,
    /// A container layout this version does not understand.
    Unsupported(&'static str),
    /// The file contradicts its own header or checksums.
    Damaged(&'static str),
    /// A declared size exceeds the configured limits.
    TooLarge(&'static str),
}

impl From<std::io::Error> for ContainerError {
    fn from(error: std::io::Error) -> Self {
        ContainerError::Io(error.to_string())
    }
}

/// Size limits applied while reading, so a hostile or corrupt file cannot exhaust memory.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Limits {
    /// Maximum uncompressed JSON size.
    pub max_json: u64,
    /// Maximum uncompressed size of one block.
    pub max_block: u64,
    /// Maximum total uncompressed size of all blocks.
    pub max_total: u64,
}

impl Default for Limits {
    fn default() -> Self {
        Self {
            max_json: 256 << 20,
            max_block: 1 << 30,
            max_total: 8 << 30,
        }
    }
}

/// Parsed header.
#[derive(Debug, Clone, PartialEq)]
pub struct Header {
    /// Schema version of the JSON document.
    pub schema_version: u32,
    /// Document identifier.
    pub document_id: [u8; 16],
    /// Uncompressed JSON length.
    pub json_raw: u64,
    /// Compressed JSON length.
    pub json_gz: u64,
    /// Number of blocks.
    pub block_count: u32,
    /// SHA-256 of the uncompressed JSON.
    pub json_sha: [u8; 32],
    /// Offset of the block index.
    pub index_offset: u64,
}

/// One entry of the block index.
#[derive(Debug, Clone, PartialEq)]
pub struct BlockEntry {
    /// Block kind.
    pub kind: u32,
    /// Identifier inside its kind (the mesh index for meshes).
    pub id: u32,
    /// Absolute offset of the compressed data.
    pub offset: u64,
    /// Compressed length.
    pub gz_len: u64,
    /// Uncompressed length.
    pub raw_len: u64,
    /// SHA-256 of the uncompressed data.
    pub sha: [u8; 32],
}

/// A block to write.
pub struct BlockSource<'a> {
    /// Block kind.
    pub kind: u32,
    /// Identifier inside its kind.
    pub id: u32,
    /// Uncompressed bytes.
    pub raw: Cow<'a, [u8]>,
}

fn gzip(data: &[u8], level: flate2::Compression) -> std::io::Result<Vec<u8>> {
    let mut encoder = flate2::write::GzEncoder::new(Vec::new(), level);
    encoder.write_all(data)?;
    encoder.finish()
}

fn gunzip(data: &[u8], expected: u64) -> Result<Vec<u8>, ContainerError> {
    let mut out = Vec::with_capacity(expected as usize);
    flate2::read::GzDecoder::new(data)
        .take(expected + 1)
        .read_to_end(&mut out)
        .map_err(|_| ContainerError::Damaged("datos comprimidos no válidos"))?;
    if out.len() as u64 != expected {
        return Err(ContainerError::Damaged(
            "la longitud descomprimida no coincide con la cabecera",
        ));
    }
    Ok(out)
}

fn sha(data: &[u8]) -> [u8; 32] {
    Sha256::digest(data).into()
}

/// Writes a complete container to `out`.
pub fn write<W: Write + Seek>(
    out: &mut W,
    schema_version: u32,
    document_id: [u8; 16],
    json: &[u8],
    blocks: &mut dyn Iterator<Item = BlockSource<'_>>,
) -> std::io::Result<()> {
    out.write_all(&[0u8; HEADER_BYTES as usize])?;
    let json_gz = gzip(json, flate2::Compression::default())?;
    out.write_all(&json_gz)?;
    let mut entries = Vec::new();
    for block in blocks {
        let gz = gzip(&block.raw, flate2::Compression::fast())?;
        entries.push(BlockEntry {
            kind: block.kind,
            id: block.id,
            offset: out.stream_position()?,
            gz_len: gz.len() as u64,
            raw_len: block.raw.len() as u64,
            sha: sha(&block.raw),
        });
        out.write_all(&gz)?;
    }
    let index_offset = out.stream_position()?;
    for entry in &entries {
        out.write_all(&entry.kind.to_le_bytes())?;
        out.write_all(&entry.id.to_le_bytes())?;
        out.write_all(&entry.offset.to_le_bytes())?;
        out.write_all(&entry.gz_len.to_le_bytes())?;
        out.write_all(&entry.raw_len.to_le_bytes())?;
        out.write_all(&entry.sha)?;
    }
    let mut header = Vec::with_capacity(HEADER_BYTES as usize);
    header.extend_from_slice(SIGNATURE);
    header.extend_from_slice(&CONTAINER_VERSION.to_le_bytes());
    header.extend_from_slice(&0u16.to_le_bytes());
    header.extend_from_slice(&schema_version.to_le_bytes());
    header.extend_from_slice(&document_id);
    header.extend_from_slice(&(json.len() as u64).to_le_bytes());
    header.extend_from_slice(&(json_gz.len() as u64).to_le_bytes());
    header.extend_from_slice(&(entries.len() as u32).to_le_bytes());
    header.extend_from_slice(&0u32.to_le_bytes());
    header.extend_from_slice(&sha(json));
    header.extend_from_slice(&index_offset.to_le_bytes());
    out.seek(SeekFrom::Start(0))?;
    out.write_all(&header)?;
    out.seek(SeekFrom::End(0))?;
    Ok(())
}

/// A container opened for reading: header and JSON are verified; blocks are read one at a time on demand.
pub struct Reader {
    file: File,
    limits: Limits,
    /// The header.
    pub header: Header,
    /// Verified uncompressed JSON.
    pub json: Vec<u8>,
    /// Block index.
    pub blocks: Vec<BlockEntry>,
}

fn u32_at(bytes: &[u8], at: usize) -> u32 {
    u32::from_le_bytes([bytes[at], bytes[at + 1], bytes[at + 2], bytes[at + 3]])
}

fn u64_at(bytes: &[u8], at: usize) -> u64 {
    let mut value = [0u8; 8];
    value.copy_from_slice(&bytes[at..at + 8]);
    u64::from_le_bytes(value)
}

impl Reader {
    /// Opens `path`, verifying the header, the JSON checksum and the block index.
    pub fn open(path: &Path, limits: Limits) -> Result<Reader, ContainerError> {
        let mut file = File::open(path)?;
        let length = file.metadata()?.len();
        let mut head = [0u8; HEADER_BYTES as usize];
        if length < HEADER_BYTES || file.read_exact(&mut head).is_err() || &head[..8] != SIGNATURE {
            return Err(ContainerError::NotAProject);
        }
        if u16::from_le_bytes([head[8], head[9]]) != CONTAINER_VERSION {
            return Err(ContainerError::Unsupported(
                "versión de contenedor desconocida",
            ));
        }
        let mut document_id = [0u8; 16];
        document_id.copy_from_slice(&head[16..32]);
        let mut json_sha = [0u8; 32];
        json_sha.copy_from_slice(&head[56..88]);
        let header = Header {
            schema_version: u32_at(&head, 12),
            document_id,
            json_raw: u64_at(&head, 32),
            json_gz: u64_at(&head, 40),
            block_count: u32_at(&head, 48),
            json_sha,
            index_offset: u64_at(&head, 88),
        };
        if header.schema_version < 1 || header.json_raw == 0 || header.json_gz == 0 {
            return Err(ContainerError::Damaged(
                "las longitudes de la cabecera no son válidas",
            ));
        }
        if header.json_raw > limits.max_json {
            return Err(ContainerError::TooLarge(
                "el documento JSON supera el límite de carga",
            ));
        }
        let index_bytes = header.block_count as u64 * INDEX_ENTRY_BYTES;
        let json_end = HEADER_BYTES.checked_add(header.json_gz);
        if json_end.is_none_or(|end| end > header.index_offset)
            || header.index_offset.checked_add(index_bytes) != Some(length)
        {
            return Err(ContainerError::Damaged(
                "el archivo está truncado o su cabecera es incoherente",
            ));
        }
        let mut gz = vec![0u8; header.json_gz as usize];
        file.read_exact(&mut gz)?;
        let json = gunzip(&gz, header.json_raw)?;
        if sha(&json) != header.json_sha {
            return Err(ContainerError::Damaged(
                "la suma de comprobación del documento no coincide",
            ));
        }
        file.seek(SeekFrom::Start(header.index_offset))?;
        let mut raw = vec![0u8; index_bytes as usize];
        file.read_exact(&mut raw)?;
        let mut blocks = Vec::with_capacity(header.block_count as usize);
        let mut total = 0u64;
        for chunk in raw.as_chunks::<{ INDEX_ENTRY_BYTES as usize }>().0 {
            let mut block_sha = [0u8; 32];
            block_sha.copy_from_slice(&chunk[32..64]);
            let entry = BlockEntry {
                kind: u32_at(chunk, 0),
                id: u32_at(chunk, 4),
                offset: u64_at(chunk, 8),
                gz_len: u64_at(chunk, 16),
                raw_len: u64_at(chunk, 24),
                sha: block_sha,
            };
            if entry.offset < json_end.unwrap_or(u64::MAX)
                || entry
                    .offset
                    .checked_add(entry.gz_len)
                    .is_none_or(|end| end > header.index_offset)
            {
                return Err(ContainerError::Damaged(
                    "un bloque apunta fuera del archivo",
                ));
            }
            if entry.raw_len > limits.max_block {
                return Err(ContainerError::TooLarge(
                    "un bloque supera el límite de carga",
                ));
            }
            total = total.saturating_add(entry.raw_len);
            blocks.push(entry);
        }
        if total > limits.max_total {
            return Err(ContainerError::TooLarge(
                "el contenido supera el límite de carga",
            ));
        }
        Ok(Reader {
            file,
            limits,
            header,
            json,
            blocks,
        })
    }

    /// Reads and verifies block number `index` of the index.
    pub fn read_block(&mut self, index: usize) -> Result<Vec<u8>, ContainerError> {
        let entry = self
            .blocks
            .get(index)
            .cloned()
            .ok_or(ContainerError::Damaged("bloque inexistente"))?;
        if entry.raw_len > self.limits.max_block {
            return Err(ContainerError::TooLarge(
                "un bloque supera el límite de carga",
            ));
        }
        self.file.seek(SeekFrom::Start(entry.offset))?;
        let mut gz = vec![0u8; entry.gz_len as usize];
        self.file.read_exact(&mut gz)?;
        let raw = gunzip(&gz, entry.raw_len)?;
        if sha(&raw) != entry.sha {
            return Err(ContainerError::Damaged(
                "la suma de comprobación de un bloque no coincide",
            ));
        }
        Ok(raw)
    }

    /// Index position of the block of `kind` with identifier `id`.
    pub fn find(&self, kind: u32, id: u32) -> Option<usize> {
        self.blocks
            .iter()
            .position(|b| b.kind == kind && b.id == id)
    }
}
