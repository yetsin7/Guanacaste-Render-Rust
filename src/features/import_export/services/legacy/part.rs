//! A `Part` collects the triangles of one element grouped by material key. Geometry is a triangle soup (three own
//! vertices per triangle) so flat faces keep exact normals and every face can be textured by projection.

use super::surface::Surface;
use crate::features::import_export::logic::triangulate::{signed_area, triangulate};
use crate::features::import_export::models::Mesh;
use std::collections::HashMap;

/// A point in scene meters (Z up).
pub type P3 = [f32; 3];

/// What a group of triangles is made of: a surface category or an already registered material.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Key {
    /// A surface category of the original application.
    Surface(Surface),
    /// A material registered in the scene by index (embedded textures, per-element overrides).
    Custom(usize),
}

impl From<Surface> for Key {
    fn from(surface: Surface) -> Self {
        Key::Surface(surface)
    }
}

impl Key {
    fn wants_uv(self) -> bool {
        match self {
            Key::Surface(surface) => surface.textured(),
            Key::Custom(_) => true,
        }
    }
}

fn sub(a: P3, b: P3) -> P3 {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

fn cross(a: P3, b: P3) -> P3 {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}

/// Texture coordinates in meters by projection along the dominant axis of the triangle's normal.
pub fn project_uv(corners: [P3; 3]) -> [[f32; 2]; 3] {
    let n = cross(sub(corners[1], corners[0]), sub(corners[2], corners[0]));
    let (ax, ay, az) = (n[0].abs(), n[1].abs(), n[2].abs());
    let pick = |p: P3| {
        if az >= ax && az >= ay {
            [p[0], p[1]]
        } else if ax >= ay {
            [p[1], p[2]]
        } else {
            [p[0], p[2]]
        }
    };
    corners.map(pick)
}

/// The triangles of one element, grouped by material key in order of first use.
#[derive(Debug, Default, Clone)]
pub struct Part {
    groups: Vec<(Key, Mesh)>,
    lookup: HashMap<Key, usize>,
}

impl Part {
    /// An empty part.
    pub fn new() -> Self {
        Self::default()
    }

    /// Number of triangles.
    #[cfg(test)]
    pub fn triangle_count(&self) -> usize {
        self.groups.iter().map(|(_, m)| m.indices.len() / 3).sum()
    }

    fn group(&mut self, key: Key) -> &mut Mesh {
        let next = self.groups.len();
        let slot = *self.lookup.entry(key).or_insert(next);
        if slot == next {
            self.groups.push((key, Mesh::default()));
        }
        &mut self.groups[slot].1
    }

    /// One triangle with projected texture coordinates when its material is textured.
    pub fn tri(&mut self, key: impl Into<Key>, corners: [P3; 3]) {
        self.push(key.into(), corners, None);
    }

    fn push(&mut self, key: Key, corners: [P3; 3], uv: Option<[[f32; 2]; 3]>) {
        let wants = key.wants_uv();
        let mesh = self.group(key);
        let start = mesh.positions.len() as u32;
        mesh.positions.extend_from_slice(&corners);
        mesh.indices.extend([start, start + 1, start + 2]);
        if wants {
            mesh.uvs
                .extend_from_slice(&uv.unwrap_or_else(|| project_uv(corners)));
        }
    }

    /// A planar quad given counter-clockwise from its front.
    pub fn quad(&mut self, key: impl Into<Key>, a: P3, b: P3, c: P3, d: P3) {
        let key = key.into();
        self.push(key, [a, b, c], None);
        self.push(key, [a, c, d], None);
    }

    /// A convex polygon fanned from its first corner.
    pub fn fan(&mut self, key: impl Into<Key>, points: &[P3]) {
        let key = key.into();
        for k in 1..points.len().saturating_sub(1) {
            self.push(key, [points[0], points[k], points[k + 1]], None);
        }
    }

    /// Appends an indexed mesh (shared vertices, explicit UVs) under `key`; indices are checked by the caller.
    pub fn indexed(
        &mut self,
        key: impl Into<Key>,
        positions: &[P3],
        uvs: &[[f32; 2]],
        indices: &[u32],
    ) {
        let mesh = self.group(key.into());
        let base = mesh.positions.len() as u32;
        mesh.positions.extend_from_slice(positions);
        mesh.uvs.extend_from_slice(uvs);
        mesh.indices.extend(indices.iter().map(|i| i + base));
    }

    /// A closed prism over a convex outline (any winding) between per-corner bottoms and tops.
    pub fn convex_prism_h(
        &mut self,
        outline: &[[f64; 2]],
        bottoms: &[f32],
        tops: &[f32],
        side: impl Into<Key>,
        cap: impl Into<Key>,
    ) {
        let (side, cap) = (side.into(), cap.into());
        let n = outline.len();
        if n < 3 || bottoms.len() != n || tops.len() != n || signed_area(outline).abs() < 1e-10 {
            return;
        }
        let order: Vec<usize> = if signed_area(outline) < 0.0 {
            (0..n).rev().collect()
        } else {
            (0..n).collect()
        };
        let at = |i: usize, z: &[f32]| -> P3 { [outline[i][0] as f32, outline[i][1] as f32, z[i]] };
        let top: Vec<P3> = order.iter().map(|&i| at(i, tops)).collect();
        let bottom: Vec<P3> = order.iter().rev().map(|&i| at(i, bottoms)).collect();
        self.fan(cap, &top);
        self.fan(cap, &bottom);
        for k in 0..n {
            let (i, j) = (order[k], order[(k + 1) % n]);
            self.quad(
                side,
                at(i, bottoms),
                at(j, bottoms),
                at(j, tops),
                at(i, tops),
            );
        }
    }

    /// A closed prism over a convex outline from `z0` to `z1`.
    pub fn convex_prism(
        &mut self,
        outline: &[[f64; 2]],
        z0: f64,
        z1: f64,
        side: impl Into<Key>,
        cap: impl Into<Key>,
    ) {
        let (low, high) = (z0.min(z1) as f32, z0.max(z1) as f32);
        let n = outline.len();
        self.convex_prism_h(outline, &vec![low; n], &vec![high; n], side, cap);
    }

    /// An axis-aligned box.
    #[cfg(test)]
    pub fn cuboid(&mut self, key: impl Into<Key>, min: P3, max: P3) {
        let key = key.into();
        let outline = [
            [f64::from(min[0]), f64::from(min[1])],
            [f64::from(max[0]), f64::from(min[1])],
            [f64::from(max[0]), f64::from(max[1])],
            [f64::from(min[0]), f64::from(max[1])],
        ];
        self.convex_prism(&outline, f64::from(min[2]), f64::from(max[2]), key, key);
    }

    /// A closed prism over any simple polygon, optionally with holes cut through it.
    pub fn prism(
        &mut self,
        outline: &[[f64; 2]],
        holes: &[Vec<[f64; 2]>],
        z0: f64,
        z1: f64,
        side: impl Into<Key>,
        cap: impl Into<Key>,
    ) {
        let (side, cap) = (side.into(), cap.into());
        let (low, high) = (z0.min(z1) as f32, z0.max(z1) as f32);
        let mut outer = outline.to_vec();
        if outer.len() > 1 && outer.first() == outer.last() {
            outer.pop();
        }
        if outer.len() < 3 || signed_area(&outer).abs() < 1e-10 {
            return;
        }
        if signed_area(&outer) < 0.0 {
            outer.reverse();
        }
        let holes: Vec<Vec<[f64; 2]>> = holes
            .iter()
            .filter(|h| h.len() >= 3 && signed_area(h).abs() > 1e-10)
            .cloned()
            .collect();
        let mut points = outer.clone();
        for hole in &holes {
            points.extend_from_slice(hole);
        }
        let lift = |i: usize, z: f32| -> P3 { [points[i][0] as f32, points[i][1] as f32, z] };
        for [a, b, c] in triangulate(&outer, &holes) {
            self.push(cap, [lift(a, high), lift(b, high), lift(c, high)], None);
            self.push(cap, [lift(a, low), lift(c, low), lift(b, low)], None);
        }
        let mut ring = |ring: &[[f64; 2]], offset: usize, outward_left: bool| {
            let n = ring.len();
            for k in 0..n {
                let (i, j) = (offset + k, offset + (k + 1) % n);
                let (i, j) = if outward_left { (i, j) } else { (j, i) };
                self.quad(
                    side,
                    lift(i, low),
                    lift(j, low),
                    lift(j, high),
                    lift(i, high),
                );
            }
        };
        ring(&outer, 0, true);
        let mut offset = outer.len();
        for hole in &holes {
            // Holes are walked clockwise so their faces look into the hole, away from the solid.
            let clockwise = signed_area(hole) < 0.0;
            ring(hole, offset, clockwise);
            offset += hole.len();
        }
    }

    /// Moves every triangle of `from` to `to`, projecting texture coordinates when `to` needs them.
    pub fn retarget(&mut self, from: Surface, to: Key) {
        let Some(slot) = self.lookup.remove(&Key::Surface(from)) else {
            return;
        };
        let mesh = std::mem::take(&mut self.groups[slot].1);
        let wants = to.wants_uv();
        for k in 0..mesh.indices.len() / 3 {
            let corners = [0, 1, 2].map(|c| mesh.positions[mesh.indices[k * 3 + c] as usize]);
            let uv = (!mesh.uvs.is_empty() && wants)
                .then(|| [0, 1, 2].map(|c| mesh.uvs[mesh.indices[k * 3 + c] as usize]));
            self.push(to, corners, uv);
        }
    }

    /// Adds every triangle of `other`.
    pub fn absorb(&mut self, other: Part) {
        for (key, mesh) in other.groups {
            if mesh.indices.is_empty() {
                continue;
            }
            if key.wants_uv() && mesh.uvs.len() != mesh.positions.len() {
                // Projected UVs are per triangle corner: re-emit the soup so they line up.
                for t in mesh.indices.chunks_exact(3) {
                    self.tri(key, [0, 1, 2].map(|c| mesh.positions[t[c] as usize]));
                }
            } else {
                self.indexed(key, &mesh.positions, &mesh.uvs, &mesh.indices);
            }
        }
    }

    /// The groups of triangles in order of first use.
    pub fn into_groups(self) -> Vec<(Key, Mesh)> {
        self.groups
            .into_iter()
            .filter(|(_, mesh)| !mesh.indices.is_empty())
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_box_has_twelve_triangles_and_textured_surfaces_get_uvs() {
        let mut part = Part::new();
        part.cuboid(Surface::Wall, [0.0; 3], [1.0, 2.0, 3.0]);
        part.cuboid(Surface::WallBrick, [0.0; 3], [1.0, 2.0, 3.0]);
        assert_eq!(part.triangle_count(), 24);
        let groups = part.into_groups();
        assert!(groups[0].1.uvs.is_empty(), "plaster is plain");
        assert_eq!(groups[1].1.uvs.len(), groups[1].1.positions.len());
    }

    #[test]
    fn a_prism_with_a_hole_keeps_the_hole_open() {
        let mut part = Part::new();
        let outer = [[0.0, 0.0], [4.0, 0.0], [4.0, 4.0], [0.0, 4.0]];
        let hole = vec![[1.0, 1.0], [3.0, 1.0], [3.0, 3.0], [1.0, 3.0]];
        part.prism(&outer, &[hole], 0.0, 0.2, Surface::Floor, Surface::Floor);
        let mesh = &part.into_groups()[0].1;
        let area: f32 = mesh
            .indices
            .chunks_exact(3)
            .filter(|t| {
                mesh.positions[t[0] as usize][2] > 0.1 && mesh.positions[t[1] as usize][2] > 0.1
            })
            .filter(|t| mesh.positions[t[2] as usize][2] > 0.1)
            .map(|t| {
                let (a, b, c) = (
                    mesh.positions[t[0] as usize],
                    mesh.positions[t[1] as usize],
                    mesh.positions[t[2] as usize],
                );
                let n = cross(sub(b, a), sub(c, a));
                if n[2].abs() > 1e-6 {
                    n[2].abs() / 2.0
                } else {
                    0.0
                }
            })
            .sum();
        assert!((area - 12.0).abs() < 1e-3, "top face area {area}");
    }

    #[test]
    fn retargeting_projects_coordinates_for_a_textured_material() {
        let mut part = Part::new();
        part.cuboid(Surface::Wall, [0.0; 3], [1.0; 3]);
        part.retarget(Surface::Wall, Key::Custom(3));
        let groups = part.into_groups();
        assert_eq!(groups.len(), 1);
        assert_eq!(groups[0].0, Key::Custom(3));
        assert_eq!(groups[0].1.uvs.len(), 36);
    }
}
