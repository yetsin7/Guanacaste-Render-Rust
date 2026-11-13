//! File access of the lighting domain.

use crate::features::lighting::logic::parse_ies;
use crate::features::lighting::models::IesProfile;
use std::path::Path;

/// Reads and parses an IES file; the error is a localization key. Non-UTF-8 bytes (Latin-1 files) are replaced.
pub fn load_ies(path: &Path) -> Result<IesProfile, &'static str> {
    let bytes = std::fs::read(path).map_err(|_| "lighting.ies_error_read")?;
    parse_ies(&String::from_utf8_lossy(&bytes)).map_err(|error| error.message_key())
}
