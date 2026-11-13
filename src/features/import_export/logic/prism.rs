//! Simple solids built from 2D footprints: extruded prisms, straight bands and rectangles.

use super::triangulate::{signed_area, triangulate};
use crate::features::import_export::models::Mesh;

/// A closed prism: `footprint` extruded from `z0` to `z1`. Returns `None` for degenerate input.
pub fn prism(footprint: &[[f64; 2]], z0: f64, z1: f64) -> Option<Mesh> {
    let mut outline = footprint.to_vec();
    if outline.len() > 1 && outline.first() == outline.last() {
        outline.pop();
    }
    let area = signed_area(&outline);
    if outline.len() < 3 || area.abs() < 1e-9 || (z1 - z0).abs() < 1e-9 {
        return None;
    }
    if area < 0.0 {
        outline.reverse();
    }
    let (low, high) = (z0.min(z1), z0.max(z1));
    let count = outline.len() as u32;
    let mut mesh = Mesh::default();
    for z in [low, high] {
        mesh.positions
            .extend(outline.iter().map(|p| [p[0] as f32, p[1] as f32, z as f32]));
    }
    for [a, b, c] in triangulate(&outline, &[]) {
        let (a, b, c) = (a as u32, b as u32, c as u32);
        mesh.indices
            .extend([a, c, b, a + count, b + count, c + count]);
    }
    for i in 0..count {
        let j = (i + 1) % count;
        mesh.indices
            .extend([i, j, j + count, i, j + count, i + count]);
    }
    Some(mesh)
}

/// Footprint of a straight element of `width` from `start` to `end`.
pub fn band(start: [f64; 2], end: [f64; 2], width: f64) -> Vec<[f64; 2]> {
    let (dx, dy) = (end[0] - start[0], end[1] - start[1]);
    let length = dx.hypot(dy);
    if length < 1e-9 {
        return Vec::new();
    }
    let (nx, ny) = (-dy / length * width / 2.0, dx / length * width / 2.0);
    vec![
        [start[0] + nx, start[1] + ny],
        [start[0] - nx, start[1] - ny],
        [end[0] - nx, end[1] - ny],
        [end[0] + nx, end[1] + ny],
    ]
}

/// Rectangular footprint of `width` by `depth` centered on `center`.
pub fn rectangle(center: [f64; 2], width: f64, depth: f64) -> Vec<[f64; 2]> {
    let (w, d) = (width / 2.0, depth / 2.0);
    vec![
        [center[0] - w, center[1] - d],
        [center[0] + w, center[1] - d],
        [center[0] + w, center[1] + d],
        [center[0] - w, center[1] + d],
    ]
}
