//! Collada `library_geometries`: sources and polygon primitives, kept as index lists until instanced.

use crate::features::import_export::logic::geometry::V3;
use crate::features::import_export::logic::triangulate::triangulate_3d;
use crate::features::import_export::models::{ImportReport, Severity};
use crate::features::import_export::services::common::MAX_INDICES;
use roxmltree::Node;
use std::collections::HashMap;

/// One drawable primitive set of a geometry: triangle corners as `(position, uv)` index pairs.
pub struct Primitive {
    /// The `material` symbol the instance binds to a real material.
    pub symbol: Option<String>,
    /// Three entries per triangle; the UV index is `u32::MAX` when absent.
    pub corners: Vec<(u32, u32)>,
}

/// A geometry with its positions, texture coordinates and primitives.
pub struct Geometry {
    /// Vertex positions in file units.
    pub positions: Vec<V3>,
    /// Texture coordinates.
    pub uvs: Vec<[f32; 2]>,
    /// Primitive sets.
    pub primitives: Vec<Primitive>,
}

/// Element children with the given local name.
pub fn children<'a, 'input>(
    node: Node<'a, 'input>,
    name: &'static str,
) -> impl Iterator<Item = Node<'a, 'input>> {
    node.children()
        .filter(move |c| c.is_element() && c.tag_name().name() == name)
}

/// The first element child with the given local name.
pub fn first<'a, 'input>(node: Node<'a, 'input>, name: &'static str) -> Option<Node<'a, 'input>> {
    children(node, name).next()
}

/// Numbers of the node's text.
pub fn numbers(node: Node) -> Vec<f64> {
    node.text()
        .unwrap_or("")
        .split_ascii_whitespace()
        .filter_map(|word| word.parse().ok())
        .collect()
}

fn reference(value: Option<&str>) -> &str {
    value.unwrap_or("").trim_start_matches('#')
}

/// `(values, stride)` of every `<source>` of a mesh by id.
fn sources(mesh: Node) -> HashMap<String, (Vec<f64>, usize)> {
    let mut out = HashMap::new();
    for source in children(mesh, "source") {
        let (Some(id), Some(array)) = (source.attribute("id"), first(source, "float_array")) else {
            continue;
        };
        let stride = first(source, "technique_common")
            .and_then(|t| first(t, "accessor"))
            .and_then(|a| a.attribute("stride"))
            .and_then(|s| s.parse().ok())
            .unwrap_or(3usize)
            .max(1);
        out.insert(id.to_owned(), (numbers(array), stride));
    }
    out
}

struct Input<'a> {
    semantic: &'a str,
    source: &'a str,
    offset: usize,
}

fn read_primitive(
    node: Node,
    kind: &str,
    position_offset: usize,
    uv_offset: Option<usize>,
    stride: usize,
    positions: &[V3],
    report: &mut ImportReport,
) -> Option<Primitive> {
    let counts: Vec<usize> = first(node, "vcount")
        .map(|v| numbers(v).iter().map(|n| *n as usize).collect())
        .unwrap_or_default();
    let mut corners: Vec<(u32, u32)> = Vec::new();
    let mut invalid = 0usize;
    let mut polygon = |indices: &[usize], corners: &mut Vec<(u32, u32)>| {
        let corner = |k: usize| {
            let at = k * stride;
            (
                indices.get(at + position_offset).copied(),
                uv_offset.map_or(Some(u32::MAX as usize), |o| indices.get(at + o).copied()),
            )
        };
        let count = indices.len() / stride;
        let list: Vec<(usize, usize)> = (0..count)
            .filter_map(|k| Some((corner(k).0?, corner(k).1?)))
            .collect();
        if list.len() < 3 || list.len() != count || list.iter().any(|c| c.0 >= positions.len()) {
            invalid += 1;
            return;
        }
        let triangles: Vec<[usize; 3]> = if list.len() <= 4 {
            (1..list.len() - 1).map(|i| [0, i, i + 1]).collect()
        } else {
            let points: Vec<V3> = list.iter().map(|c| positions[c.0]).collect();
            triangulate_3d(&points, &[]).1
        };
        for triangle in triangles {
            for k in triangle {
                corners.push((list[k].0 as u32, list[k].1 as u32));
            }
        }
    };
    match kind {
        "triangles" => {
            for p in children(node, "p") {
                let indices: Vec<usize> = numbers(p).iter().map(|n| *n as usize).collect();
                for triangle in indices.chunks(stride * 3) {
                    polygon(triangle, &mut corners);
                }
            }
        }
        "polylist" => {
            let indices: Vec<usize> = children(node, "p")
                .flat_map(numbers)
                .map(|n| n as usize)
                .collect();
            let mut at = 0usize;
            for count in counts {
                let end = at
                    .saturating_add(count.saturating_mul(stride))
                    .min(indices.len());
                polygon(&indices[at..end], &mut corners);
                at = end;
            }
        }
        _ => {
            for p in children(node, "p") {
                let indices: Vec<usize> = numbers(p).iter().map(|n| *n as usize).collect();
                polygon(&indices, &mut corners);
            }
        }
    }
    if invalid > 0 {
        report.note(
            Severity::Warning,
            "dae.bad_polygons",
            &[&invalid.to_string()],
        );
    }
    Some(Primitive {
        symbol: node.attribute("material").map(str::to_owned),
        corners,
    })
}

/// Parses a `<geometry>` that has a `<mesh>`; other kinds (splines, convex meshes) return `None`.
pub fn parse(geometry: Node, report: &mut ImportReport) -> Option<Geometry> {
    let mesh = first(geometry, "mesh")?;
    let sources = sources(mesh);
    let vertices = first(mesh, "vertices");
    let vertices_id = vertices.and_then(|v| v.attribute("id")).unwrap_or("");
    let vertex_source = vertices
        .into_iter()
        .flat_map(|v| children(v, "input"))
        .find(|i| i.attribute("semantic") == Some("POSITION"))
        .map(|i| reference(i.attribute("source")).to_owned());
    let (flat, stride) = sources.get(vertex_source.as_deref()?)?;
    let positions: Vec<V3> = flat
        .chunks_exact(*stride)
        .filter(|c| c.len() >= 3)
        .map(|c| [c[0], c[1], c[2]])
        .collect();
    let mut uvs: Vec<[f32; 2]> = Vec::new();
    let mut uv_source: Option<String> = None;
    let mut primitives = Vec::new();
    for node in mesh.children().filter(Node::is_element) {
        let kind = node.tag_name().name();
        if !matches!(kind, "triangles" | "polylist" | "polygons") {
            if matches!(kind, "tristrips" | "trifans" | "lines" | "linestrips") {
                report.note(Severity::Warning, "dae.unsupported_primitive", &[kind]);
            }
            continue;
        }
        let inputs: Vec<Input> = children(node, "input")
            .map(|i| Input {
                semantic: i.attribute("semantic").unwrap_or(""),
                source: reference(i.attribute("source")),
                offset: i
                    .attribute("offset")
                    .and_then(|o| o.parse().ok())
                    .unwrap_or(0),
            })
            .collect();
        let stride = inputs.iter().map(|i| i.offset + 1).max().unwrap_or(1);
        let Some(position_offset) = inputs
            .iter()
            .find(|i| i.semantic == "VERTEX" && i.source == vertices_id)
            .map(|i| i.offset)
        else {
            continue;
        };
        let texcoord = inputs.iter().find(|i| i.semantic == "TEXCOORD");
        let uv_offset = texcoord.and_then(|t| {
            let (values, uv_stride) = sources.get(t.source)?;
            if uv_source.is_none() {
                uv_source = Some(t.source.to_owned());
                uvs = values
                    .chunks_exact(*uv_stride)
                    .filter(|c| c.len() >= 2)
                    .map(|c| [c[0] as f32, c[1] as f32])
                    .collect();
            }
            (uv_source.as_deref() == Some(t.source)).then_some(t.offset)
        });
        if let Some(mut primitive) = read_primitive(
            node,
            kind,
            position_offset,
            uv_offset,
            stride,
            &positions,
            report,
        ) {
            if uv_offset.is_some() && primitive.corners.iter().any(|c| c.1 as usize >= uvs.len()) {
                primitive.corners.iter_mut().for_each(|c| {
                    if c.1 as usize >= uvs.len() {
                        c.1 = u32::MAX;
                    }
                });
            }
            if primitive.corners.len() > MAX_INDICES {
                return None;
            }
            primitives.push(primitive);
        }
    }
    Some(Geometry {
        positions,
        uvs,
        primitives,
    })
}
