//! The `G3DNATV1` container of Guanacaste 3D projects: header checks, bounded decompression and checksum.
//! Every length comes from an untrusted file, so all of them are validated before anything is allocated.

use crate::features::import_export::models::ImportError;
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::io::Read;

/// Signature of a Guanacaste 3D native container.
pub const SIGNATURE: &[u8; 8] = b"G3DNATV1";
/// Size of the fixed header.
pub const HEADER_BYTES: usize = 81;
/// Largest compressed file and largest uncompressed JSON accepted.
pub const MAX_PAYLOAD: u64 = 256 << 20;

/// An [`ImportError::Invalid`] with `text`.
pub fn invalid(text: &str) -> ImportError {
    ImportError::Invalid(text.to_owned())
}

/// Verifies the container and returns `(schema version, JSON document)`.
pub fn read_container(bytes: &[u8]) -> Result<(i32, Value), ImportError> {
    if bytes.len() < 8 || &bytes[..8] != SIGNATURE {
        return Err(invalid(
            "No es un proyecto de Guanacaste 3D (firma G3DNATV1 ausente).",
        ));
    }
    if bytes.len() < HEADER_BYTES {
        return Err(invalid(
            "El archivo está truncado: no contiene la cabecera completa. Vuelve a copiarlo desde el original.",
        ));
    }
    let i32_at =
        |at: usize| i32::from_le_bytes([bytes[at], bytes[at + 1], bytes[at + 2], bytes[at + 3]]);
    let i64_at = |at: usize| {
        let mut value = [0u8; 8];
        value.copy_from_slice(&bytes[at..at + 8]);
        i64::from_le_bytes(value)
    };
    if i32_at(8) != 1 {
        return Err(invalid(
            "Versión de contenedor de Guanacaste 3D desconocida.",
        ));
    }
    if bytes[12] != 1 {
        return Err(invalid(
            "El archivo es una familia o plantilla de Guanacaste 3D, no un proyecto.",
        ));
    }
    let schema = i32_at(13);
    let (raw_len, gz_len) = (i64_at(33), i64_at(41));
    if raw_len <= 0 || gz_len <= 0 || raw_len as u64 > MAX_PAYLOAD || gz_len as u64 > MAX_PAYLOAD {
        return Err(invalid(
            "Las longitudes del contenedor no son válidas o superan el límite de 256 MB.",
        ));
    }
    let expected = HEADER_BYTES as u64 + gz_len as u64;
    if (bytes.len() as u64) < expected {
        return Err(invalid(
            "El archivo está truncado: faltan datos al final. Vuelve a copiarlo desde el original.",
        ));
    }
    if bytes.len() as u64 != expected {
        return Err(invalid(
            "Las longitudes del contenedor no coinciden con el archivo.",
        ));
    }
    // The header is not trusted for the allocation: grow as data actually decompresses.
    let mut json = Vec::with_capacity((gz_len as usize).saturating_mul(4).min(raw_len as usize));
    flate2::read::GzDecoder::new(&bytes[HEADER_BYTES..])
        .take(raw_len as u64 + 1)
        .read_to_end(&mut json)
        .map_err(|_| invalid("Los datos comprimidos están dañados."))?;
    if json.len() as i64 != raw_len || Sha256::digest(&json)[..] != bytes[49..81] {
        return Err(invalid("La suma de comprobación del proyecto no coincide."));
    }
    let value = serde_json::from_slice(&json).map_err(|e| invalid(&e.to_string()))?;
    Ok((schema, value))
}
