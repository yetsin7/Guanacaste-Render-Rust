//! 3D Studio `.3ds` reader: named objects, triangle meshes, texture coordinates and materials (diffuse color,
//! transparency, texture file name). Animation, lights and cameras are ignored. The chunk walker checks every
//! length against the data, so a damaged file produces an error or a partial model, never a panic.

use super::common::{
    finish, node, orient, read_limited, source_info, texture_path, MeshBuilder, MAX_FILE_BYTES,
};
use crate::features::import_export::logic::geometry::V3;
use crate::features::import_export::models::{
    ImportError, ImportOptions, ImportOutcome, ImportReport, Material, Mesh, NodeKind, Severity,
    SourceFormat,
};
use crate::features::import_export::services::progress::Progress;
use std::collections::HashMap;
use std::path::Path;

const MAIN: u16 = 0x4D4D;
const EDITOR: u16 = 0x3D3D;
const OBJECT: u16 = 0x4000;
const TRIMESH: u16 = 0x4100;
const VERTICES: u16 = 0x4110;
const FACES: u16 = 0x4120;
const FACE_MATERIAL: u16 = 0x4130;
const UVS: u16 = 0x4140;
const MATERIAL: u16 = 0xAFFF;
const MAT_NAME: u16 = 0xA000;
const MAT_DIFFUSE: u16 = 0xA020;
const MAT_TRANSPARENCY: u16 = 0xA050;
const MAT_TEXTURE: u16 = 0xA200;
const MAT_FILE: u16 = 0xA300;
const COLOR_FLOAT: u16 = 0x0010;
const COLOR_BYTE: u16 = 0x0011;
const COLOR_BYTE_LINEAR: u16 = 0x0012;
const COLOR_FLOAT_LINEAR: u16 = 0x0013;
const PERCENT_SHORT: u16 = 0x0030;
const PERCENT_FLOAT: u16 = 0x0031;

/// The sub-chunks of `data`, and whether the chunk list ended on a damaged header or length.
fn chunks(data: &[u8]) -> (Vec<(u16, &[u8])>, bool) {
    let mut found = Vec::new();
    let mut at = 0usize;
    while at < data.len() {
        let Some(header) = data.get(at..at + 6) else {
            return (found, true);
        };
        let id = u16::from_le_bytes([header[0], header[1]]);
        let length = u32::from_le_bytes([header[2], header[3], header[4], header[5]]) as usize;
        if length < 6 {
            return (found, true);
        }
        let end = at.saturating_add(length);
        found.push((id, &data[at + 6..end.min(data.len())]));
        if end > data.len() {
            return (found, true);
        }
        at = end;
    }
    (found, false)
}

fn child(data: &[u8], id: u16) -> Option<&[u8]> {
    chunks(data).0.into_iter().find(|c| c.0 == id).map(|c| c.1)
}

/// A NUL-terminated string and the bytes after it.
fn cstring(data: &[u8]) -> (String, &[u8]) {
    let end = data.iter().position(|b| *b == 0).unwrap_or(data.len());
    (
        String::from_utf8_lossy(&data[..end]).into_owned(),
        data.get(end + 1..).unwrap_or(&[]),
    )
}

fn u16_at(data: &[u8], at: usize) -> Option<usize> {
    let raw = data.get(at..at + 2)?;
    Some(u16::from_le_bytes([raw[0], raw[1]]) as usize)
}

fn f32_at(data: &[u8], at: usize) -> Option<f32> {
    let raw = data.get(at..at + 4)?;
    Some(f32::from_le_bytes([raw[0], raw[1], raw[2], raw[3]]))
}

/// RGB of a color chunk (float or byte flavor) found among `data`'s children.
fn color_in(data: &[u8]) -> Option<[f32; 3]> {
    for (id, body) in chunks(data).0 {
        match id {
            COLOR_FLOAT | COLOR_FLOAT_LINEAR => {
                return Some([f32_at(body, 0)?, f32_at(body, 4)?, f32_at(body, 8)?]);
            }
            COLOR_BYTE | COLOR_BYTE_LINEAR => {
                let rgb = body.get(..3)?;
                return Some([rgb[0], rgb[1], rgb[2]].map(|v| v as f32 / 255.0));
            }
            _ => {}
        }
    }
    None
}

fn parse_material(data: &[u8], folder: Option<&Path>) -> Material {
    let mut material = Material {
        name: "3DS".into(),
        color: [0.8, 0.8, 0.8, 1.0],
        texture: None,
    };
    for (id, body) in chunks(data).0 {
        match id {
            MAT_NAME => material.name = cstring(body).0,
            MAT_DIFFUSE => {
                if let Some(c) = color_in(body) {
                    material.color[..3].copy_from_slice(&c);
                }
            }
            MAT_TRANSPARENCY => {
                let percent = chunks(body).0.into_iter().find_map(|(id, b)| match id {
                    PERCENT_SHORT => u16_at(b, 0).map(|v| v as f32),
                    PERCENT_FLOAT => f32_at(b, 0),
                    _ => None,
                });
                if let Some(p) = percent {
                    material.color[3] = (1.0 - p / 100.0).clamp(0.0, 1.0);
                }
            }
            MAT_TEXTURE => {
                if let Some(file) = child(body, MAT_FILE) {
                    material.texture = Some(texture_path(folder, &cstring(file).0));
                }
            }
            _ => {}
        }
    }
    material
}

/// Meshes of one `TRIMESH` chunk, one per material, plus the count of faces that were dropped.
fn parse_trimesh(
    data: &[u8],
    material_ids: &HashMap<String, usize>,
    options: &ImportOptions,
) -> (Vec<Mesh>, usize) {
    let mut positions: Vec<V3> = Vec::new();
    let mut uvs: Vec<[f32; 2]> = Vec::new();
    let mut faces: Vec<[usize; 3]> = Vec::new();
    let mut groups: Vec<(Option<usize>, Vec<usize>)> = Vec::new();
    for (id, body) in chunks(data).0 {
        match id {
            VERTICES => {
                let n = u16_at(body, 0).unwrap_or(0);
                positions = (0..n)
                    .filter_map(|i| {
                        let at = 2 + i * 12;
                        Some([
                            f32_at(body, at)? as f64,
                            f32_at(body, at + 4)? as f64,
                            f32_at(body, at + 8)? as f64,
                        ])
                    })
                    .collect();
            }
            UVS => {
                let n = u16_at(body, 0).unwrap_or(0);
                uvs = (0..n)
                    .filter_map(|i| Some([f32_at(body, 2 + i * 8)?, f32_at(body, 6 + i * 8)?]))
                    .collect();
            }
            FACES => {
                let n = u16_at(body, 0).unwrap_or(0);
                faces = (0..n)
                    .filter_map(|i| {
                        let at = 2 + i * 8;
                        Some([
                            u16_at(body, at)?,
                            u16_at(body, at + 2)?,
                            u16_at(body, at + 4)?,
                        ])
                    })
                    .collect();
                for (sub, group) in chunks(body.get(2 + n * 8..).unwrap_or(&[])).0 {
                    if sub != FACE_MATERIAL {
                        continue;
                    }
                    let (name, rest) = cstring(group);
                    let count = u16_at(rest, 0).unwrap_or(0);
                    let members = (0..count).filter_map(|i| u16_at(rest, 2 + i * 2)).collect();
                    groups.push((material_ids.get(&name).copied(), members));
                }
            }
            _ => {}
        }
    }
    let mut face_material: Vec<Option<usize>> = vec![None; faces.len()];
    for (material, members) in &groups {
        for face in members.iter().filter(|f| **f < faces.len()) {
            face_material[*face] = *material;
        }
    }
    let mut builders: Vec<(Option<usize>, MeshBuilder)> = Vec::new();
    let mut dropped = 0;
    for (face, material) in faces.iter().zip(&face_material) {
        let valid = face.iter().all(|i| *i < positions.len())
            && face
                .iter()
                .all(|i| positions[*i].iter().all(|v| v.is_finite()));
        if !valid {
            dropped += 1;
            continue;
        }
        let slot = match builders.iter().position(|(m, _)| m == material) {
            Some(slot) => slot,
            None => {
                builders.push((*material, MeshBuilder::default()));
                builders.len() - 1
            }
        };
        let builder = &mut builders[slot].1;
        let ids = face.map(|i| builder.vertex(orient(positions[i], options), uvs.get(i).copied()));
        builder.triangle(ids[0], ids[1], ids[2]);
    }
    let meshes = builders
        .into_iter()
        .map(|(material, builder)| builder.finish(material))
        .collect();
    (meshes, dropped)
}

/// Imports 3DS `bytes`; `folder` is where textures are looked for.
pub fn import_bytes(
    bytes: &[u8],
    name: &str,
    folder: Option<&Path>,
    options: &ImportOptions,
    progress: &Progress,
) -> Result<ImportOutcome, ImportError> {
    let started = std::time::Instant::now();
    let (top, mut damaged) = chunks(bytes);
    let Some((_, main)) = top.into_iter().find(|c| c.0 == MAIN) else {
        return Err(ImportError::Invalid(
            "No es un archivo 3DS válido: falta el bloque principal (0x4D4D).".into(),
        ));
    };
    let (main_children, main_damaged) = chunks(main);
    damaged |= main_damaged;
    let editor = main_children
        .iter()
        .find(|c| c.0 == EDITOR)
        .map_or(&[][..], |c| c.1);
    let (entries, editor_damaged) = chunks(editor);
    damaged |= editor_damaged;
    progress.check()?;
    let mut report = ImportReport::default();
    let mut materials: Vec<Material> = Vec::new();
    let mut material_ids: HashMap<String, usize> = HashMap::new();
    for (_, body) in entries.iter().filter(|c| c.0 == MATERIAL) {
        let material = parse_material(body, folder);
        material_ids.insert(material.name.clone(), materials.len());
        materials.push(material);
    }
    let mut nodes = vec![node(name, "", NodeKind::Project, None)];
    let mut meshes: Vec<Mesh> = Vec::new();
    let mut dropped = 0;
    let objects: Vec<_> = entries.iter().filter(|c| c.0 == OBJECT).collect();
    for (index, (_, body)) in objects.iter().enumerate() {
        progress.check()?;
        progress.set(index as f32 / objects.len().max(1) as f32 * 0.95);
        let (object_name, rest) = cstring(body);
        let Some(trimesh) = child(rest, TRIMESH) else {
            continue;
        };
        let (parts, bad) = parse_trimesh(trimesh, &material_ids, options);
        dropped += bad;
        let mut element = node(&object_name, "3DS", NodeKind::Element, Some(0));
        for mesh in parts {
            meshes.push(mesh);
            element.meshes.push(meshes.len() - 1);
        }
        nodes.push(element);
        let at = nodes.len() - 1;
        nodes[0].children.push(at);
    }
    if damaged {
        report.note(Severity::Warning, "tds.damaged", &[]);
    }
    if dropped > 0 {
        report.note(Severity::Warning, "tds.bad_faces", &[&dropped.to_string()]);
    }
    report.note(
        Severity::Info,
        "mesh.units_assumed",
        &[&crate::features::import_export::services::ifc::label_for(
            options.unit_scale,
        )],
    );
    progress.set(1.0);
    let version = child(main, 0x0002)
        .and_then(|b| b.get(..4))
        .map_or(0, |b| u32::from_le_bytes([b[0], b[1], b[2], b[3]]));
    let source = source_info(
        SourceFormat::ThreeDs,
        &format!("3DS v{version}"),
        "",
        options.unit_scale,
        bytes.len() as u64,
    );
    finish(source, nodes, meshes, materials, report, started)
}

/// Imports the 3DS file at `path`.
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
        .unwrap_or("3DS")
        .to_owned();
    import_bytes(&bytes, &name, path.parent(), options, progress)
}
