//! glTF 2.0 (JSON and GLB) reader.

use crate::features::import_export::logic::geometry::{Mat4, V3};
use crate::features::import_export::models::{
    ImportError, ImportOutcome, ImportReport, ImportedScene, Material, Mesh, NodeKind, SceneNode,
    Severity, SourceFormat, SourceInfo,
};
use crate::features::import_export::services::progress::Progress;
use serde_json::Value;
use std::path::Path;

fn invalid(text: &str) -> ImportError {
    ImportError::Invalid(text.to_owned())
}

fn base64(data: &str) -> Vec<u8> {
    let mut out = Vec::with_capacity(data.len() * 3 / 4);
    let (mut buffer, mut bits) = (0u32, 0);
    for byte in data.bytes() {
        let value = match byte {
            b'A'..=b'Z' => byte - b'A',
            b'a'..=b'z' => byte - b'a' + 26,
            b'0'..=b'9' => byte - b'0' + 52,
            b'+' | b'-' => 62,
            b'/' | b'_' => 63,
            _ => continue,
        };
        buffer = (buffer << 6) | value as u32;
        bits += 6;
        if bits >= 8 {
            bits -= 8;
            out.push((buffer >> bits) as u8);
            buffer &= (1 << bits) - 1;
        }
    }
    out
}

fn split_glb(bytes: &[u8]) -> Result<(String, Option<Vec<u8>>), ImportError> {
    let word = |at: usize| {
        bytes
            .get(at..at + 4)
            .map(|b| u32::from_le_bytes([b[0], b[1], b[2], b[3]]))
    };
    let (mut at, mut json, mut bin) = (12usize, None, None);
    while let (Some(length), Some(kind)) = (word(at), word(at + 4)) {
        let body = bytes
            .get(at + 8..at + 8 + length as usize)
            .ok_or_else(|| invalid("GLB truncado"))?;
        match kind {
            0x4E4F_534A => json = Some(String::from_utf8_lossy(body).into_owned()),
            0x004E_4942 if bin.is_none() => bin = Some(body.to_vec()),
            _ => {}
        }
        at += 8 + length as usize;
    }
    Ok((json.ok_or_else(|| invalid("GLB sin bloque JSON"))?, bin))
}

struct Doc {
    json: Value,
    buffers: Vec<Vec<u8>>,
}

impl Doc {
    fn accessor(&self, index: usize) -> Option<(Vec<f64>, usize)> {
        let accessor = self.json["accessors"].get(index)?;
        let view = self.json["bufferViews"].get(accessor["bufferView"].as_u64()? as usize)?;
        let buffer = self.buffers.get(view["buffer"].as_u64()? as usize)?;
        let components = match accessor["type"].as_str()? {
            "SCALAR" => 1,
            "VEC2" => 2,
            "VEC3" => 3,
            "VEC4" => 4,
            _ => return None,
        };
        let size = match accessor["componentType"].as_u64()? {
            5120 | 5121 => 1,
            5122 | 5123 => 2,
            _ => 4,
        };
        let stride = view["byteStride"]
            .as_u64()
            .map_or(size * components, |s| s as usize);
        let start = view["byteOffset"].as_u64().unwrap_or(0) as usize
            + accessor["byteOffset"].as_u64().unwrap_or(0) as usize;
        let count = accessor["count"].as_u64()? as usize;
        let kind = accessor["componentType"].as_u64()?;
        let mut out = Vec::with_capacity(count * components);
        for item in 0..count {
            for component in 0..components {
                let at = start + item * stride + component * size;
                let raw = buffer.get(at..at + size)?;
                out.push(match kind {
                    5126 => f32::from_le_bytes([raw[0], raw[1], raw[2], raw[3]]) as f64,
                    5125 => u32::from_le_bytes([raw[0], raw[1], raw[2], raw[3]]) as f64,
                    5123 => u16::from_le_bytes([raw[0], raw[1]]) as f64,
                    5121 => raw[0] as f64,
                    _ => return None,
                });
            }
        }
        Some((out, components))
    }
}

fn node_matrix(node: &Value) -> Mat4 {
    if let Some(m) = node["matrix"].as_array().filter(|m| m.len() == 16) {
        let v: Vec<f64> = m.iter().map(|x| x.as_f64().unwrap_or(0.0)).collect();
        return Mat4([
            [v[0], v[4], v[8], v[12]],
            [v[1], v[5], v[9], v[13]],
            [v[2], v[6], v[10], v[14]],
            [0.0, 0.0, 0.0, 1.0],
        ]);
    }
    let number = |key: &str, n: usize, default: f64| -> Vec<f64> {
        node[key].as_array().map_or_else(
            || vec![default; n],
            |a| a.iter().map(|x| x.as_f64().unwrap_or(default)).collect(),
        )
    };
    let t = number("translation", 3, 0.0);
    let s = number("scale", 3, 1.0);
    let q = node["rotation"]
        .as_array()
        .map_or(vec![0.0, 0.0, 0.0, 1.0], |a| {
            a.iter().map(|x| x.as_f64().unwrap_or(0.0)).collect()
        });
    let (x, y, z, w) = (q[0], q[1], q[2], q[3]);
    Mat4([
        [
            (1.0 - 2.0 * (y * y + z * z)) * s[0],
            2.0 * (x * y - z * w) * s[1],
            2.0 * (x * z + y * w) * s[2],
            t[0],
        ],
        [
            2.0 * (x * y + z * w) * s[0],
            (1.0 - 2.0 * (x * x + z * z)) * s[1],
            2.0 * (y * z - x * w) * s[2],
            t[1],
        ],
        [
            2.0 * (x * z - y * w) * s[0],
            2.0 * (y * z + x * w) * s[1],
            (1.0 - 2.0 * (x * x + y * y)) * s[2],
            t[2],
        ],
        [0.0, 0.0, 0.0, 1.0],
    ])
}

struct Reader<'a> {
    doc: &'a Doc,
    nodes: Vec<SceneNode>,
    meshes: Vec<Mesh>,
    report: ImportReport,
}

impl Reader<'_> {
    fn visit(&mut self, index: usize, parent: usize, matrix: Mat4, depth: usize) {
        let Some(node) = self.doc.json["nodes"].get(index) else {
            return;
        };
        if depth > 64 {
            return;
        }
        let world = matrix.mul(&node_matrix(node));
        let id = self.nodes.len();
        let name = node["name"]
            .as_str()
            .map_or_else(|| format!("nodo {index}"), str::to_owned);
        self.nodes.push(SceneNode {
            name,
            class: "glTF".into(),
            kind: NodeKind::Group,
            type_name: None,
            parent: Some(parent),
            children: Vec::new(),
            meshes: Vec::new(),
            level: None,
        });
        self.nodes[parent].children.push(id);
        if let Some(mesh) = node["mesh"].as_u64() {
            self.add_mesh(mesh as usize, &world, id);
        }
        for child in node["children"]
            .as_array()
            .into_iter()
            .flatten()
            .filter_map(Value::as_u64)
        {
            self.visit(child as usize, id, world, depth + 1);
        }
    }

    fn add_mesh(&mut self, index: usize, world: &Mat4, node: usize) {
        let Some(mesh) = self.doc.json["meshes"].get(index) else {
            return;
        };
        for primitive in mesh["primitives"].as_array().into_iter().flatten() {
            if primitive["mode"].as_u64().unwrap_or(4) != 4 {
                self.report.note(
                    Severity::Warning,
                    "gltf.mode_skipped",
                    &[&primitive["mode"].to_string()],
                );
                continue;
            }
            if primitive["extensions"]["KHR_draco_mesh_compression"].is_object() {
                self.report.note(Severity::Warning, "gltf.draco", &[]);
                continue;
            }
            let Some((positions, _)) = primitive["attributes"]["POSITION"]
                .as_u64()
                .and_then(|a| self.doc.accessor(a as usize))
            else {
                self.report
                    .note(Severity::Warning, "gltf.primitive_failed", &[]);
                continue;
            };
            let vertices = positions.len() / 3;
            let indices: Vec<u32> = match primitive["indices"]
                .as_u64()
                .and_then(|a| self.doc.accessor(a as usize))
            {
                Some((values, _)) => values.into_iter().map(|v| v as u32).collect(),
                None => (0..vertices as u32).collect(),
            };
            if indices.iter().any(|i| *i as usize >= vertices) {
                self.report
                    .note(Severity::Warning, "gltf.primitive_failed", &[]);
                continue;
            }
            let mapped: Vec<[f32; 3]> = positions
                .chunks(3)
                .map(|p| {
                    let w: V3 = world.point([p[0], p[1], p[2]]);
                    [w[0] as f32, -w[2] as f32, w[1] as f32]
                })
                .collect();
            self.meshes.push(Mesh {
                positions: mapped,
                indices,
                material: primitive["material"].as_u64().map(|m| m as usize),
            });
            self.nodes[node].meshes.push(self.meshes.len() - 1);
            self.nodes[node].kind = NodeKind::Element;
        }
    }
}

/// Imports glTF/GLB bytes; `external` resolves relative buffer URIs.
pub fn import_bytes(
    bytes: &[u8],
    file_size: u64,
    progress: &Progress,
    external: &dyn Fn(&str) -> Option<Vec<u8>>,
) -> Result<ImportOutcome, ImportError> {
    let started = std::time::Instant::now();
    let (text, glb_bin) = if bytes.starts_with(b"glTF") {
        split_glb(bytes)?
    } else {
        (String::from_utf8_lossy(bytes).into_owned(), None)
    };
    let json: Value = serde_json::from_str(&text)
        .map_err(|error| ImportError::Invalid(format!("JSON de glTF no válido: {error}")))?;
    let mut report = ImportReport::default();
    let mut buffers = Vec::new();
    for buffer in json["buffers"].as_array().into_iter().flatten() {
        progress.check()?;
        let data = match buffer["uri"].as_str() {
            None => glb_bin.clone().unwrap_or_default(),
            Some(uri) if uri.starts_with("data:") => {
                base64(uri.split_once(',').map_or("", |(_, data)| data))
            }
            Some(uri) => external(uri).unwrap_or_else(|| {
                report.note(Severity::Error, "gltf.buffer_missing", &[uri]);
                Vec::new()
            }),
        };
        buffers.push(data);
    }
    let doc = Doc { json, buffers };
    let materials: Vec<Material> = doc.json["materials"]
        .as_array()
        .into_iter()
        .flatten()
        .enumerate()
        .map(|(i, m)| {
            let c: Vec<f32> = m["pbrMetallicRoughness"]["baseColorFactor"]
                .as_array()
                .map_or(vec![0.8, 0.8, 0.8, 1.0], |a| {
                    a.iter().map(|x| x.as_f64().unwrap_or(0.8) as f32).collect()
                });
            Material {
                name: m["name"]
                    .as_str()
                    .map_or_else(|| format!("material {i}"), str::to_owned),
                color: [c[0], c[1], c[2], *c.get(3).unwrap_or(&1.0)],
            }
        })
        .collect();
    let root = SceneNode {
        name: "glTF".into(),
        class: String::new(),
        kind: NodeKind::Project,
        type_name: None,
        parent: None,
        children: Vec::new(),
        meshes: Vec::new(),
        level: None,
    };
    let mut reader = Reader {
        doc: &doc,
        nodes: vec![root],
        meshes: Vec::new(),
        report,
    };
    let scene_index = doc.json["scene"].as_u64().unwrap_or(0) as usize;
    let roots: Vec<usize> = doc.json["scenes"]
        .get(scene_index)
        .and_then(|s| s["nodes"].as_array())
        .map_or_else(
            || (0..doc.json["nodes"].as_array().map_or(0, Vec::len)).collect(),
            |a| {
                a.iter()
                    .filter_map(Value::as_u64)
                    .map(|n| n as usize)
                    .collect()
            },
        );
    for node in roots {
        reader.visit(node, 0, Mat4::IDENTITY, 0);
    }
    let mut report = reader.report;
    let elements = reader
        .nodes
        .iter()
        .filter(|n| n.kind == NodeKind::Element)
        .count();
    report.elements_total = elements;
    report.elements_with_geometry = elements;
    report.duration_ms = started.elapsed().as_millis();
    let version = doc.json["asset"]["version"].as_str().unwrap_or("2.0");
    let source = SourceInfo {
        format: SourceFormat::Gltf,
        version: format!("glTF {version}"),
        application: doc.json["asset"]["generator"]
            .as_str()
            .unwrap_or("")
            .to_owned(),
        length_unit: "m".into(),
        unit_scale: 1.0,
        file_size,
    };
    progress.set(1.0);
    Ok(ImportOutcome {
        scene: ImportedScene {
            source,
            nodes: reader.nodes,
            meshes: reader.meshes,
            materials,
            levels: Vec::new(),
            origin_shift: [0.0; 3],
        },
        report,
    })
}

/// Imports the glTF or GLB file at `path`.
pub fn import(path: &Path, progress: &Progress) -> Result<ImportOutcome, ImportError> {
    let bytes = std::fs::read(path)?;
    let folder = path.parent().map(Path::to_path_buf).unwrap_or_default();
    import_bytes(&bytes, bytes.len() as u64, progress, &|uri| {
        std::fs::read(folder.join(uri)).ok()
    })
}
