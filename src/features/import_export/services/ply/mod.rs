//! Stanford PLY reader: ASCII and binary (little and big endian). Reads positions, polygon faces, texture
//! coordinates (`s,t` / `u,v`), vertex colors (averaged into one material color: the scene model has no
//! per-vertex color) and the `TextureFile` comment.

mod header;

use super::common::{
    flat_scene, orient, read_limited, source_info, texture_path, too_big, MAX_FILE_BYTES,
    MAX_INDICES,
};
use crate::features::import_export::logic::geometry::V3;
use crate::features::import_export::logic::triangulate::triangulate_3d;
use crate::features::import_export::models::{
    ImportError, ImportOptions, ImportOutcome, ImportReport, Material, Mesh, Severity, SourceFormat,
};
use crate::features::import_export::services::progress::Progress;
use header::{parse_header, Element, Encoding, Header, Reader, Scalar};
use std::path::Path;

const MAX_LIST: usize = 1 << 16;
const CHECK_EVERY: usize = 16384;

fn truncated() -> ImportError {
    ImportError::Invalid(
        "El archivo PLY está truncado: terminan los datos antes de lo que declara la cabecera."
            .into(),
    )
}

/// Vertex data gathered from the `vertex` element.
#[derive(Default)]
struct Vertices {
    positions: Vec<V3>,
    uvs: Vec<[f32; 2]>,
    color_sum: [f64; 3],
    colored: usize,
}

/// Roles of a vertex property.
#[derive(Clone, Copy, PartialEq)]
enum Role {
    X,
    Y,
    Z,
    U,
    V,
    Red,
    Green,
    Blue,
    Other,
}

fn role(name: &str) -> Role {
    match name {
        "x" => Role::X,
        "y" => Role::Y,
        "z" => Role::Z,
        "s" | "u" | "texture_u" | "texture_s" => Role::U,
        "t" | "v" | "texture_v" | "texture_t" => Role::V,
        "red" | "r" | "diffuse_red" => Role::Red,
        "green" | "g" | "diffuse_green" => Role::Green,
        "blue" | "b" | "diffuse_blue" => Role::Blue,
        _ => Role::Other,
    }
}

/// Minimum bytes (or characters) an instance of `element` takes, to reject absurd counts early.
fn min_size(element: &Element, binary: bool) -> usize {
    element
        .props
        .iter()
        .map(|p| {
            if !binary {
                2
            } else {
                p.count.map_or(p.kind.size(), Scalar::size)
            }
        })
        .sum::<usize>()
        .max(1)
}

fn read_vertices(
    element: &Element,
    reader: &mut Reader,
    progress: &Progress,
) -> Result<Vertices, ImportError> {
    let roles: Vec<Role> = element.props.iter().map(|p| role(&p.name)).collect();
    let has_uv = roles.contains(&Role::U) && roles.contains(&Role::V);
    let mut out = Vertices::default();
    for index in 0..element.count {
        if index % CHECK_EVERY == 0 {
            progress.check()?;
        }
        let (mut p, mut uv, mut color, mut has_color) = ([0.0; 3], [0.0f32; 2], [0.0; 3], false);
        for (property, role) in element.props.iter().zip(&roles) {
            if let Some(count) = property.count {
                let n = reader.read(count).ok_or_else(truncated)? as usize;
                for _ in 0..n.min(MAX_LIST) {
                    reader.read(property.kind).ok_or_else(truncated)?;
                }
                continue;
            }
            let value = reader.read(property.kind).ok_or_else(truncated)?;
            let channel = |v: f64| {
                if property.kind == Scalar::U8 {
                    v / 255.0
                } else {
                    v
                }
            };
            match role {
                Role::X => p[0] = value,
                Role::Y => p[1] = value,
                Role::Z => p[2] = value,
                Role::U => uv[0] = value as f32,
                Role::V => uv[1] = value as f32,
                Role::Red => (color[0], has_color) = (channel(value), true),
                Role::Green => color[1] = channel(value),
                Role::Blue => color[2] = channel(value),
                Role::Other => {}
            }
        }
        out.positions.push(p);
        if has_uv {
            out.uvs.push(uv);
        }
        if has_color {
            out.colored += 1;
            for axis in 0..3 {
                out.color_sum[axis] += color[axis];
            }
        }
    }
    Ok(out)
}

/// Reads the `face` element into triangles. Returns how many faces were invalid.
fn read_faces(
    element: &Element,
    reader: &mut Reader,
    vertices: &[V3],
    progress: &Progress,
    indices: &mut Vec<u32>,
) -> Result<usize, ImportError> {
    let mut invalid = 0;
    for index in 0..element.count {
        if index % CHECK_EVERY == 0 {
            progress.check()?;
        }
        if indices.len() >= MAX_INDICES {
            return Err(too_big());
        }
        for property in &element.props {
            let Some(count) = property.count else {
                reader.read(property.kind).ok_or_else(truncated)?;
                continue;
            };
            let n = reader.read(count).ok_or_else(truncated)? as usize;
            let mut face: Vec<usize> = Vec::with_capacity(n.min(MAX_LIST));
            for _ in 0..n.min(MAX_LIST) {
                face.push(reader.read(property.kind).ok_or_else(truncated)? as usize);
            }
            if !matches!(property.name.as_str(), "vertex_indices" | "vertex_index") {
                continue;
            }
            if face.len() < 3 || face.iter().any(|i| *i >= vertices.len()) {
                invalid += 1;
            } else if face.len() <= 4 {
                for i in 1..face.len() - 1 {
                    indices.extend([face[0] as u32, face[i] as u32, face[i + 1] as u32]);
                }
            } else {
                let points: Vec<V3> = face.iter().map(|i| vertices[*i]).collect();
                for t in triangulate_3d(&points, &[]).1 {
                    indices.extend(t.map(|k| face[k] as u32));
                }
            }
        }
    }
    Ok(invalid)
}

fn build(
    header: &Header,
    bytes: &[u8],
    options: &ImportOptions,
    progress: &Progress,
    report: &mut ImportReport,
) -> Result<(Mesh, Vertices), ImportError> {
    let binary = header.encoding != Encoding::Ascii;
    let body = bytes.get(header.body..).unwrap_or(&[]);
    let text;
    let mut reader = if binary {
        Reader::Binary {
            bytes: body,
            at: 0,
            big: header.encoding == Encoding::Big,
        }
    } else {
        text = String::from_utf8_lossy(body);
        Reader::Ascii(text.split_ascii_whitespace())
    };
    let mut vertices = Vertices::default();
    let mut indices: Vec<u32> = Vec::new();
    for element in &header.elements {
        if element.count.saturating_mul(min_size(element, binary)) > reader.remaining() {
            return Err(truncated());
        }
        match element.name.as_str() {
            "vertex" => vertices = read_vertices(element, &mut reader, progress)?,
            "face" => {
                let invalid = read_faces(
                    element,
                    &mut reader,
                    &vertices.positions,
                    progress,
                    &mut indices,
                )?;
                if invalid > 0 {
                    report.note(Severity::Warning, "ply.bad_faces", &[&invalid.to_string()]);
                }
            }
            _ => {
                for _ in 0..element.count {
                    for property in &element.props {
                        let n = property.count.map_or(Some(1.0), |c| reader.read(c));
                        for _ in 0..(n.ok_or_else(truncated)? as usize).min(MAX_LIST) {
                            reader.read(property.kind).ok_or_else(truncated)?;
                        }
                    }
                }
            }
        }
    }
    let mesh = Mesh {
        positions: vertices
            .positions
            .iter()
            .map(|p| orient(*p, options))
            .collect(),
        indices,
        material: None,
        uvs: vertices.uvs.clone(),
    };
    Ok((mesh, vertices))
}

/// Imports PLY `bytes`; `folder` is where a referenced texture is looked for.
pub fn import_bytes(
    bytes: &[u8],
    name: &str,
    folder: Option<&Path>,
    options: &ImportOptions,
    progress: &Progress,
) -> Result<ImportOutcome, ImportError> {
    let started = std::time::Instant::now();
    let mut report = ImportReport::default();
    let header = parse_header(bytes)?;
    let (mut mesh, vertices) = build(&header, bytes, options, progress, &mut report)?;
    let mut materials = Vec::new();
    if vertices.colored > 0 || (header.texture.is_some() && !mesh.uvs.is_empty()) {
        let n = vertices.colored.max(1) as f64;
        let color = if vertices.colored > 0 {
            report.note(Severity::Info, "ply.colors_averaged", &[]);
            let c = vertices.color_sum.map(|v| (v / n).clamp(0.0, 1.0) as f32);
            [c[0], c[1], c[2], 1.0]
        } else {
            [0.8, 0.8, 0.8, 1.0]
        };
        materials.push(Material {
            name: name.to_owned(),
            color,
            texture: header
                .texture
                .as_deref()
                .filter(|_| !mesh.uvs.is_empty())
                .map(|file| texture_path(folder, file)),
            pbr: None,
        });
        mesh.material = Some(0);
    }
    report.note(
        Severity::Info,
        "mesh.units_assumed",
        &[&crate::features::import_export::services::ifc::label_for(
            options.unit_scale,
        )],
    );
    progress.set(1.0);
    let source = source_info(
        SourceFormat::Ply,
        "PLY",
        "",
        options.unit_scale,
        bytes.len() as u64,
    );
    flat_scene(
        source,
        "PLY",
        name,
        vec![(name.to_owned(), mesh)],
        materials,
        report,
        started,
    )
}

/// Imports the PLY file at `path`.
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
        .unwrap_or("PLY")
        .to_owned();
    import_bytes(&bytes, &name, path.parent(), options, progress)
}
