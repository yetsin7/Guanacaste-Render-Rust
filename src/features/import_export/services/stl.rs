//! STL reader, binary and ASCII. STL stores loose triangles, so equal vertices are welded to get an indexed mesh.
//! A truncated binary file loads the triangles that are there and says so in the report.

use super::common::{
    flat_scene, orient, read_limited, source_info, too_big, MeshBuilder, MAX_FILE_BYTES,
};
use crate::features::import_export::models::{
    ImportError, ImportOptions, ImportOutcome, ImportReport, Mesh, Severity, SourceFormat,
};
use crate::features::import_export::services::progress::Progress;
use std::path::Path;

const HEADER: usize = 84;
const RECORD: usize = 50;
const CHECK_EVERY: usize = 8192;

fn float(bytes: &[u8], at: usize) -> f64 {
    let mut raw = [0u8; 4];
    raw.copy_from_slice(&bytes[at..at + 4]);
    f32::from_le_bytes(raw) as f64
}

/// Whether the bytes are an ASCII STL rather than a binary one whose header happens to start with `solid`.
fn is_ascii(bytes: &[u8]) -> bool {
    let start = bytes
        .iter()
        .position(|b| !b.is_ascii_whitespace())
        .unwrap_or(0);
    if !bytes[start..].starts_with(b"solid") {
        return false;
    }
    if bytes.len() >= HEADER {
        let declared = u32::from_le_bytes([bytes[80], bytes[81], bytes[82], bytes[83]]) as usize;
        if declared
            .checked_mul(RECORD)
            .and_then(|n| n.checked_add(HEADER))
            == Some(bytes.len())
        {
            return false;
        }
    }
    bytes.iter().take(2048).all(|b| *b != 0)
}

fn finite(p: [f64; 3]) -> bool {
    p.iter().all(|v| v.is_finite() && v.abs() < 1e18)
}

fn binary(
    bytes: &[u8],
    options: &ImportOptions,
    progress: &Progress,
    report: &mut ImportReport,
) -> Result<Vec<(String, Mesh)>, ImportError> {
    if bytes.len() < HEADER {
        return Err(ImportError::Invalid(
            "El archivo STL es demasiado corto: le falta la cabecera.".into(),
        ));
    }
    let declared = u32::from_le_bytes([bytes[80], bytes[81], bytes[82], bytes[83]]) as usize;
    let available = (bytes.len() - HEADER) / RECORD;
    if declared > available {
        report.note(
            Severity::Warning,
            "stl.truncated",
            &[&declared.to_string(), &available.to_string()],
        );
    }
    let mut builder = MeshBuilder::default();
    let mut invalid = 0usize;
    for index in 0..declared.min(available) {
        if index % CHECK_EVERY == 0 {
            progress.check()?;
            progress.set(index as f32 / declared.max(1) as f32 * 0.95);
        }
        if builder.is_full() {
            return Err(too_big());
        }
        let base = HEADER + index * RECORD + 12;
        let corners: Vec<[f64; 3]> = (0..3)
            .map(|k| {
                let at = base + k * 12;
                [float(bytes, at), float(bytes, at + 4), float(bytes, at + 8)]
            })
            .collect();
        if !corners.iter().copied().all(finite) {
            invalid += 1;
            continue;
        }
        let ids: Vec<u32> = corners
            .iter()
            .map(|p| builder.vertex(orient(*p, options), None))
            .collect();
        builder.triangle(ids[0], ids[1], ids[2]);
    }
    if invalid > 0 {
        report.note(Severity::Warning, "mesh.nonfinite", &[&invalid.to_string()]);
    }
    Ok(vec![("STL".into(), builder.finish(None))])
}

fn ascii(
    bytes: &[u8],
    options: &ImportOptions,
    progress: &Progress,
    report: &mut ImportReport,
) -> Result<Vec<(String, Mesh)>, ImportError> {
    let text = String::from_utf8_lossy(bytes);
    let mut parts: Vec<(String, MeshBuilder)> = vec![("STL".into(), MeshBuilder::default())];
    let mut corners: Vec<[f64; 3]> = Vec::new();
    let (mut invalid, mut loose) = (0usize, 0usize);
    for (number, line) in text.lines().enumerate() {
        if number % CHECK_EVERY == 0 {
            progress.check()?;
            progress.set(number as f32 / (text.len() as f32 / 40.0).max(1.0) * 0.95);
        }
        let mut words = line.split_whitespace();
        match words.next() {
            Some("solid") => {
                let name = words.collect::<Vec<_>>().join(" ");
                if parts
                    .last()
                    .is_some_and(|(_, b)| !b.mesh.indices.is_empty())
                {
                    parts.push((name, MeshBuilder::default()));
                } else if !name.is_empty() {
                    if let Some(last) = parts.last_mut() {
                        last.0 = name;
                    }
                }
            }
            Some("vertex") => {
                let v: Vec<f64> = words.filter_map(|w| w.parse().ok()).collect();
                if v.len() < 3 {
                    loose += 1;
                    continue;
                }
                corners.push([v[0], v[1], v[2]]);
                if corners.len() == 3 {
                    let Some((_, builder)) = parts.last_mut() else {
                        break;
                    };
                    if builder.is_full() {
                        return Err(too_big());
                    }
                    if corners.iter().copied().all(finite) {
                        let ids: Vec<u32> = corners
                            .iter()
                            .map(|p| builder.vertex(orient(*p, options), None))
                            .collect();
                        builder.triangle(ids[0], ids[1], ids[2]);
                    } else {
                        invalid += 1;
                    }
                    corners.clear();
                }
            }
            Some("endfacet") if !corners.is_empty() => {
                loose += 1;
                corners.clear();
            }
            _ => {}
        }
    }
    if invalid > 0 {
        report.note(Severity::Warning, "mesh.nonfinite", &[&invalid.to_string()]);
    }
    if loose > 0 {
        report.note(
            Severity::Warning,
            "stl.loose_vertices",
            &[&loose.to_string()],
        );
    }
    Ok(parts
        .into_iter()
        .map(|(name, builder)| (name, builder.finish(None)))
        .collect())
}

/// Imports STL `bytes` (`size` is the file size).
pub fn import_bytes(
    bytes: &[u8],
    name: &str,
    options: &ImportOptions,
    progress: &Progress,
) -> Result<ImportOutcome, ImportError> {
    let started = std::time::Instant::now();
    let mut report = ImportReport::default();
    let mut parts = if is_ascii(bytes) {
        ascii(bytes, options, progress, &mut report)?
    } else {
        binary(bytes, options, progress, &mut report)?
    };
    if let [(part_name, _)] = parts.as_mut_slice() {
        if part_name == "STL" {
            *part_name = name.to_owned();
        }
    }
    report.note(
        Severity::Info,
        "mesh.units_assumed",
        &[&source_label(options)],
    );
    progress.set(1.0);
    let source = source_info(
        SourceFormat::Stl,
        if is_ascii(bytes) {
            "STL ASCII"
        } else {
            "STL binario"
        },
        "",
        options.unit_scale,
        bytes.len() as u64,
    );
    flat_scene(source, "STL", name, parts, Vec::new(), report, started)
}

fn source_label(options: &ImportOptions) -> String {
    crate::features::import_export::services::ifc::label_for(options.unit_scale)
}

/// Imports the STL file at `path`.
pub fn import(
    path: &Path,
    options: &ImportOptions,
    progress: &Progress,
) -> Result<ImportOutcome, ImportError> {
    let bytes = read_limited(path, MAX_FILE_BYTES)?;
    progress.check()?;
    let name = path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("STL")
        .to_owned();
    import_bytes(&bytes, &name, options, progress)
}
