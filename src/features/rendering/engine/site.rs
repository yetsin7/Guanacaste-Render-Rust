//! What the model looks like as a site: the bounds of the building itself (without hidden soil, pipes or far-away
//! stray objects) and the height of the ground around it. Cameras frame the first; the landscape sits on the second.

use crate::features::import_export::models::{Bounds, Mesh};

/// Height of the ground bins in meters.
const BIN: f32 = 0.05;

/// The measured site of a scene.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Site {
    /// Bounds of the visible building without outliers.
    pub focus: Option<Bounds>,
    /// Height of the ground (lawn or floor level) the building stands on.
    pub ground_z: f32,
    /// Direction (radians from +X, seen from above) from the building toward the facade with the most doors and
    /// windows, where the "front" cameras stand.
    pub facade: f32,
}

struct Piece {
    min: [f32; 3],
    max: [f32; 3],
    triangles: usize,
}

fn piece(mesh: &Mesh) -> Option<Piece> {
    let mut points = mesh
        .positions
        .iter()
        .filter(|p| p.iter().all(|v| v.is_finite()));
    let first = *points.next()?;
    let (mut min, mut max) = (first, first);
    for p in points {
        for axis in 0..3 {
            min[axis] = min[axis].min(p[axis]);
            max[axis] = max[axis].max(p[axis]);
        }
    }
    Some(Piece {
        min,
        max,
        triangles: mesh.indices.len() / 3,
    })
}

/// The `fraction` quantile of `values` weighted by `weight` (values and weights are consumed sorted).
fn quantile(mut values: Vec<(f32, f32)>, fraction: f32) -> f32 {
    values.sort_by(|a, b| a.0.total_cmp(&b.0));
    let total: f32 = values.iter().map(|v| v.1).sum();
    let mut seen = 0.0;
    for (value, weight) in &values {
        seen += weight;
        if seen >= total * fraction {
            return *value;
        }
    }
    values.last().map_or(0.0, |v| v.0)
}

/// Bounds of the pieces that sit with the bulk of the model: pieces whose center is far outside the central 90 %
/// of centers (a stray object, a distant wall) are ignored.
fn core_bounds(pieces: &[Piece]) -> Option<Bounds> {
    if pieces.is_empty() {
        return None;
    }
    let mut limits = [(0.0f32, 0.0f32); 2];
    for (axis, limit) in limits.iter_mut().enumerate() {
        let centers = |pieces: &[Piece]| -> Vec<(f32, f32)> {
            pieces
                .iter()
                .map(|p| {
                    (
                        (p.min[axis] + p.max[axis]) * 0.5,
                        (p.triangles.max(1) as f32).sqrt(),
                    )
                })
                .collect()
        };
        let (low, high) = (
            quantile(centers(pieces), 0.05),
            quantile(centers(pieces), 0.95),
        );
        let margin = (high - low) * 0.3 + 1.0;
        *limit = (low - margin, high + margin);
    }
    let mut bounds: Option<Bounds> = None;
    for p in pieces {
        let inside = (0..2).all(|axis| {
            let center = (p.min[axis] + p.max[axis]) * 0.5;
            center >= limits[axis].0 && center <= limits[axis].1
        });
        if !inside {
            continue;
        }
        let b = bounds.get_or_insert(Bounds {
            min: p.min,
            max: p.max,
        });
        for axis in 0..3 {
            b.min[axis] = b.min[axis].min(p.min[axis]);
            b.max[axis] = b.max[axis].max(p.max[axis]);
        }
    }
    bounds
}

/// Height where upward-facing surfaces gather: the lowest level with at least a fifth of the area of the busiest
/// one (floors and lawn), so foundations buried below do not count.
fn ground_level(meshes: &[Mesh], visible: &[bool]) -> Option<f32> {
    let mut bins: std::collections::BTreeMap<i32, f32> = std::collections::BTreeMap::new();
    for (mesh, _) in meshes
        .iter()
        .zip(visible)
        .filter(|(m, v)| **v && m.indices.len() >= 3)
    {
        for tri in mesh.indices.chunks_exact(3) {
            let Some(corners) = tri
                .iter()
                .map(|i| mesh.positions.get(*i as usize).copied())
                .collect::<Option<Vec<_>>>()
            else {
                continue;
            };
            let e1 = [0, 1, 2].map(|k| corners[1][k] - corners[0][k]);
            let e2 = [0, 1, 2].map(|k| corners[2][k] - corners[0][k]);
            let n = [
                e1[1] * e2[2] - e1[2] * e2[1],
                e1[2] * e2[0] - e1[0] * e2[2],
                e1[0] * e2[1] - e1[1] * e2[0],
            ];
            let area = (n[0] * n[0] + n[1] * n[1] + n[2] * n[2]).sqrt() * 0.5;
            // Level and facing up (or down: slabs are often modeled with flipped faces).
            if area > 1e-6 && (n[2].abs() / (2.0 * area)) > 0.98 {
                let z = (corners[0][2] + corners[1][2] + corners[2][2]) / 3.0;
                *bins.entry((z / BIN).round() as i32).or_insert(0.0) += area;
            }
        }
    }
    let window = |bin: i32| -> f32 { (-2..=2).filter_map(|d| bins.get(&(bin + d))).sum() };
    let best = bins.keys().map(|b| window(*b)).fold(0.0f32, f32::max);
    bins.keys()
        .find(|b| window(**b) >= best * 0.2)
        .map(|b| *b as f32 * BIN)
}

/// The side of `focus` that carries most of the doors and windows (`openings` are mesh indices), as the direction
/// toward the camera that faces it. Doors count double. Ties go to the south.
fn facade_direction(meshes: &[Mesh], visible: &[bool], openings: &[usize], focus: &Bounds) -> f32 {
    let center = [
        (focus.min[0] + focus.max[0]) * 0.5,
        (focus.min[1] + focus.max[1]) * 0.5,
    ];
    // +X, +Y, -X, -Y
    let mut sides = [0.0f32; 4];
    sides[3] = 0.01;
    for &index in openings {
        if !visible.get(index).copied().unwrap_or(false) {
            continue;
        }
        let Some(p) = meshes.get(index).and_then(piece) else {
            continue;
        };
        let offset = [
            (p.min[0] + p.max[0]) * 0.5 - center[0],
            (p.min[1] + p.max[1]) * 0.5 - center[1],
        ];
        let slot = if offset[0].abs() > offset[1].abs() {
            if offset[0] > 0.0 {
                0
            } else {
                2
            }
        } else if offset[1] > 0.0 {
            1
        } else {
            3
        };
        sides[slot] += 1.0;
    }
    let best = (0..4).fold(3, |best, i| if sides[i] > sides[best] { i } else { best });
    best as f32 * std::f32::consts::FRAC_PI_2
}

/// Measures `meshes` considering only the ones that `visible` marks; `openings` are the indices of door and
/// window meshes.
pub fn measure(meshes: &[Mesh], visible: &[bool], openings: &[usize]) -> Site {
    let pieces: Vec<Piece> = meshes
        .iter()
        .zip(visible)
        .filter(|(_, v)| **v)
        .filter_map(|(m, _)| piece(m))
        .collect();
    let focus = core_bounds(&pieces);
    Site {
        facade: focus.as_ref().map_or(super::framing::SOUTH, |f| {
            facade_direction(meshes, visible, openings, f)
        }),
        focus,
        ground_z: ground_level(meshes, visible).unwrap_or(0.0),
    }
}
