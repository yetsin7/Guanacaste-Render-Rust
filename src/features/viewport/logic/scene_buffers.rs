//! Turns imported meshes into GPU-ready vertex and index bytes, once per scene (never per frame).
//!
//! Vertices are 16 bytes: position (3 x `f32`) and an RGBA8 color baked from the mesh material. Normals are not
//! stored; the shader derives flat normals from screen-space derivatives. Meshes are grouped into chunks so a huge
//! model never needs one giant buffer; transparent materials go to their own chunks.

use super::grid::{self, Overlay};
use crate::features::import_export::models::{Bounds, Mesh};

/// Bytes per vertex: 3 floats and 4 color bytes.
pub const VERTEX_STRIDE: usize = 16;
/// A chunk is closed once it holds this many vertices (a single larger mesh gets a chunk of its own).
pub const CHUNK_TARGET_VERTICES: usize = 100_000;
/// Largest buffer allocated for one chunk; bigger meshes are skipped and counted.
pub const MAX_BUFFER_BYTES: usize = 200 * 1024 * 1024;
/// Color of meshes without a material.
pub const DEFAULT_COLOR: [f32; 4] = [0.78, 0.78, 0.76, 1.0];
/// Materials with less alpha than this are drawn blended.
pub const OPAQUE_ALPHA: f32 = 0.98;
/// Materials with at most this alpha are invisible and are left out of the buffers.
pub const HIDDEN_ALPHA: f32 = 0.004;

/// A group of meshes drawn with one draw call.
#[derive(Debug, Clone, PartialEq)]
pub struct SceneChunk {
    /// Interleaved vertices (see [`VERTEX_STRIDE`]).
    pub vertex_bytes: Vec<u8>,
    /// `u32` indices, native endian.
    pub index_bytes: Vec<u8>,
    /// Number of vertices.
    pub vertex_count: u32,
    /// Number of indices.
    pub index_count: u32,
    /// Bounds of the vertices the chunk draws.
    pub bounds: Bounds,
    /// Drawn with alpha blending after the opaque chunks.
    pub transparent: bool,
}

/// Everything the GPU needs for a scene.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct SceneBuffers {
    /// Mesh chunks, opaque first.
    pub chunks: Vec<SceneChunk>,
    /// Bounds of the drawn geometry, `None` when nothing is drawable.
    pub bounds: Option<Bounds>,
    /// Triangles drawn.
    pub triangles: usize,
    /// Vertices uploaded.
    pub vertices: usize,
    /// Meshes dropped because they were too large or had no valid triangle.
    pub skipped_meshes: usize,
    /// Ground grid and axes.
    pub overlay: Overlay,
}

fn pack_color(color: [f32; 4]) -> [u8; 4] {
    color.map(|c| (c.clamp(0.0, 1.0) * 255.0 + 0.5) as u8)
}

fn grow(bounds: &mut Option<Bounds>, point: [f32; 3]) {
    match bounds {
        Some(b) => {
            for axis in 0..3 {
                b.min[axis] = b.min[axis].min(point[axis]);
                b.max[axis] = b.max[axis].max(point[axis]);
            }
        }
        None => {
            *bounds = Some(Bounds {
                min: point,
                max: point,
            })
        }
    }
}

fn merge(into: &mut Option<Bounds>, other: &Bounds) {
    grow(into, other.min);
    grow(into, other.max);
}

#[derive(Default)]
struct Builder {
    vertices: Vec<u8>,
    indices: Vec<u8>,
    vertex_count: usize,
    index_count: usize,
    bounds: Option<Bounds>,
}

impl Builder {
    fn finish(&mut self, transparent: bool, out: &mut Vec<SceneChunk>) {
        let taken = std::mem::take(self);
        if let Some(bounds) = taken.bounds {
            out.push(SceneChunk {
                vertex_bytes: taken.vertices,
                index_bytes: taken.indices,
                vertex_count: taken.vertex_count as u32,
                index_count: taken.index_count as u32,
                bounds,
                transparent,
            });
        }
    }
}

/// Valid triangles of `mesh` as indices, with the bounds of the vertices they use.
fn valid_triangles(mesh: &Mesh) -> (Vec<u32>, Option<Bounds>) {
    let count = mesh.positions.len();
    let usable = |index: u32| {
        mesh.positions
            .get(index as usize)
            .is_some_and(|p| p.iter().all(|v| v.is_finite()))
    };
    let mut indices = Vec::with_capacity(mesh.indices.len());
    let mut bounds = None;
    for triangle in mesh.indices.chunks_exact(3) {
        if count > 0 && triangle.iter().all(|i| usable(*i)) {
            for index in triangle {
                grow(&mut bounds, mesh.positions[*index as usize]);
            }
            indices.extend_from_slice(triangle);
        }
    }
    (indices, bounds)
}

fn append(builder: &mut Builder, mesh: &Mesh, indices: &[u32], bounds: &Bounds, color: [u8; 4]) {
    let base = builder.vertex_count as u32;
    for position in &mesh.positions {
        let p = if position.iter().all(|v| v.is_finite()) {
            *position
        } else {
            [0.0; 3]
        };
        for value in p {
            builder.vertices.extend_from_slice(&value.to_ne_bytes());
        }
        builder.vertices.extend_from_slice(&color);
    }
    for index in indices {
        builder
            .indices
            .extend_from_slice(&(base + index).to_ne_bytes());
    }
    builder.vertex_count += mesh.positions.len();
    builder.index_count += indices.len();
    merge(&mut builder.bounds, bounds);
}

/// Meshes in an order that keeps neighbors together (Morton order of their centers, ground plane and other huge
/// meshes last) so every chunk covers a small region and frustum culling can skip most of them. The first value is
/// the mesh's original position, which decides whether it counts for the framing.
fn spatial_order(meshes: &[Mesh]) -> Vec<(usize, &Mesh)> {
    let mut bounds: Option<Bounds> = None;
    let centers: Vec<Option<[f32; 3]>> = meshes
        .iter()
        .map(|mesh| {
            let (_, b) = valid_triangles(mesh);
            b.map(|b| {
                merge(&mut bounds, &b);
                [0, 1, 2].map(|k| (b.min[k] + b.max[k]) * 0.5)
            })
        })
        .collect();
    let Some(all) = bounds else {
        return meshes.iter().enumerate().collect();
    };
    let cell = |value: f32, axis: usize| -> u32 {
        let extent = (0..3).map(|k| all.max[k] - all.min[k]).fold(1e-3, f32::max);
        (((value - all.min[axis]) / extent).clamp(0.0, 1.0) * 1023.0) as u32
    };
    let spread = |mut v: u32| -> u32 {
        v = (v | (v << 16)) & 0x030000FF;
        v = (v | (v << 8)) & 0x0300F00F;
        v = (v | (v << 4)) & 0x030C30C3;
        (v | (v << 2)) & 0x09249249
    };
    let mut order: Vec<(u32, usize)> = centers
        .iter()
        .enumerate()
        .map(|(i, c)| {
            let key = c.map_or(u32::MAX, |c| {
                spread(cell(c[0], 0)) | (spread(cell(c[1], 1)) << 1) | (spread(cell(c[2], 2)) << 2)
            });
            (key, i)
        })
        .collect();
    order.sort_by_key(|(key, _)| *key);
    order.into_iter().map(|(_, i)| (i, &meshes[i])).collect()
}

/// Builds the buffers of a scene. `material_colors` holds RGBA (0..=1) per material index.
pub fn build(meshes: &[Mesh], material_colors: &[[f32; 4]]) -> SceneBuffers {
    build_framed(meshes, material_colors, usize::MAX)
}

/// Like [`build`], but only the first `framed` meshes count for the bounds (what the camera frames and the grid
/// follows); the rest, such as a wide ground plane, is drawn without stretching the view.
pub fn build_framed(meshes: &[Mesh], material_colors: &[[f32; 4]], framed: usize) -> SceneBuffers {
    let mut out = SceneBuffers::default();
    let mut opaque = Builder::default();
    let mut blended = Builder::default();
    let mut opaque_chunks = Vec::new();
    let mut blended_chunks = Vec::new();
    for (position, mesh) in spatial_order(meshes) {
        let invisible = mesh
            .material
            .and_then(|m| material_colors.get(m))
            .is_some_and(|c| c[3] <= HIDDEN_ALPHA);
        if invisible {
            continue;
        }
        let too_big = mesh.positions.len() * VERTEX_STRIDE > MAX_BUFFER_BYTES
            || mesh.indices.len() * 4 > MAX_BUFFER_BYTES;
        let (indices, bounds) = valid_triangles(mesh);
        let Some(bounds) = bounds.filter(|_| !too_big) else {
            out.skipped_meshes += 1;
            continue;
        };
        let color = mesh
            .material
            .and_then(|m| material_colors.get(m))
            .copied()
            .unwrap_or(DEFAULT_COLOR);
        let transparent = color[3] < OPAQUE_ALPHA;
        let (builder, chunks) = if transparent {
            (&mut blended, &mut blended_chunks)
        } else {
            (&mut opaque, &mut opaque_chunks)
        };
        let full = builder.vertex_count > 0
            && (builder.vertex_count + mesh.positions.len() > CHUNK_TARGET_VERTICES
                || (builder.vertex_count + mesh.positions.len()) * VERTEX_STRIDE
                    > MAX_BUFFER_BYTES
                || (builder.index_count + indices.len()) * 4 > MAX_BUFFER_BYTES);
        if full {
            builder.finish(transparent, chunks);
        }
        append(builder, mesh, &indices, &bounds, pack_color(color));
        if position < framed {
            merge(&mut out.bounds, &bounds);
        }
        out.triangles += indices.len() / 3;
        out.vertices += mesh.positions.len();
    }
    opaque.finish(false, &mut opaque_chunks);
    blended.finish(true, &mut blended_chunks);
    out.chunks = opaque_chunks;
    out.chunks.append(&mut blended_chunks);
    out.overlay = grid::overlay(out.bounds.as_ref());
    out
}
