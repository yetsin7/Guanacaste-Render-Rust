//! Low-poly shape builder: boxes, frustums between two points, ellipsoids, quads and planes, grouped into one mesh per
//! material, then packed into an [`ImportedScene`] (meters, Z up, base at the origin).

use crate::features::import_export::models::{
    ImportedScene, Material, Mesh, NodeKind, SceneNode, SourceFormat, SourceInfo,
};
use std::f32::consts::TAU;

type V3 = [f32; 3];

fn add(a: V3, b: V3) -> V3 {
    [a[0] + b[0], a[1] + b[1], a[2] + b[2]]
}

fn sub(a: V3, b: V3) -> V3 {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

fn scale(a: V3, s: f32) -> V3 {
    [a[0] * s, a[1] * s, a[2] * s]
}

fn cross(a: V3, b: V3) -> V3 {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}

fn normalize(a: V3) -> V3 {
    let length = (a[0] * a[0] + a[1] * a[1] + a[2] * a[2]).sqrt();
    if length <= f32::EPSILON {
        [0.0, 0.0, 1.0]
    } else {
        scale(a, 1.0 / length)
    }
}

/// Collects triangles per material and produces an [`ImportedScene`].
#[derive(Debug, Default)]
pub struct ShapeBuilder {
    materials: Vec<Material>,
    meshes: Vec<Mesh>,
}

impl ShapeBuilder {
    /// Registers a material and returns its index (one mesh per material).
    pub fn material(&mut self, name: &str, rgb: u32) -> usize {
        let channel = |shift: u32| ((rgb >> shift) & 0xff) as f32 / 255.0;
        self.materials.push(Material {
            name: name.to_owned(),
            color: [channel(16), channel(8), channel(0), 1.0],
            texture: None,
            pbr: None,
        });
        self.meshes.push(Mesh {
            material: Some(self.materials.len() - 1),
            ..Mesh::default()
        });
        self.materials.len() - 1
    }

    fn vertex(&mut self, material: usize, point: V3) -> u32 {
        let mesh = &mut self.meshes[material];
        mesh.positions.push(point);
        (mesh.positions.len() - 1) as u32
    }

    fn triangle(&mut self, material: usize, a: u32, b: u32, c: u32) {
        self.meshes[material].indices.extend_from_slice(&[a, b, c]);
    }

    /// A convex polygon (counter-clockwise seen from its front), fanned into triangles.
    pub fn polygon(&mut self, material: usize, points: &[V3]) {
        let first = self.vertex(material, points[0]);
        let mut previous = self.vertex(material, points[1]);
        for point in &points[2..] {
            let next = self.vertex(material, *point);
            self.triangle(material, first, previous, next);
            previous = next;
        }
    }

    /// A convex polygon visible from both sides (leaves, fronds, panels).
    pub fn double_sided(&mut self, material: usize, points: &[V3]) {
        self.polygon(material, points);
        let reversed: Vec<V3> = points.iter().rev().copied().collect();
        self.polygon(material, &reversed);
    }

    /// An axis-aligned box given its center and half extents.
    pub fn cuboid(&mut self, material: usize, center: V3, half: V3) {
        let corner = |x: f32, y: f32, z: f32| {
            [
                center[0] + x * half[0],
                center[1] + y * half[1],
                center[2] + z * half[2],
            ]
        };
        let faces = [
            [
                (-1., -1., -1.),
                (-1., 1., -1.),
                (1., 1., -1.),
                (1., -1., -1.),
            ],
            [(-1., -1., 1.), (1., -1., 1.), (1., 1., 1.), (-1., 1., 1.)],
            [
                (-1., -1., -1.),
                (1., -1., -1.),
                (1., -1., 1.),
                (-1., -1., 1.),
            ],
            [(1., 1., -1.), (-1., 1., -1.), (-1., 1., 1.), (1., 1., 1.)],
            [(1., -1., -1.), (1., 1., -1.), (1., 1., 1.), (1., -1., 1.)],
            [
                (-1., 1., -1.),
                (-1., -1., -1.),
                (-1., -1., 1.),
                (-1., 1., 1.),
            ],
        ];
        for face in faces {
            let points = face.map(|(x, y, z)| corner(x, y, z));
            self.polygon(material, &points);
        }
    }

    /// A capped frustum from `from` (radius `r0`) to `to` (radius `r1`); `r1 == 0` makes a cone.
    pub fn frustum(&mut self, material: usize, from: V3, to: V3, r0: f32, r1: f32, segments: u32) {
        let axis = normalize(sub(to, from));
        let helper = if axis[2].abs() < 0.9 {
            [0.0, 0.0, 1.0]
        } else {
            [1.0, 0.0, 0.0]
        };
        let u = normalize(cross(axis, helper));
        let v = cross(axis, u);
        let ring = |center: V3, radius: f32| -> Vec<V3> {
            (0..segments)
                .map(|i| {
                    let angle = TAU * i as f32 / segments as f32;
                    add(
                        center,
                        add(
                            scale(u, angle.cos() * radius),
                            scale(v, angle.sin() * radius),
                        ),
                    )
                })
                .collect()
        };
        let bottom = ring(from, r0);
        let top = ring(to, r1);
        for i in 0..segments as usize {
            let j = (i + 1) % segments as usize;
            if r1 > 0.0 {
                self.polygon(material, &[bottom[i], bottom[j], top[j], top[i]]);
            } else {
                self.polygon(material, &[bottom[i], bottom[j], to]);
            }
        }
        let mut cap: Vec<V3> = bottom.into_iter().rev().collect();
        self.polygon(material, &cap);
        if r1 > 0.0 {
            cap = top;
            self.polygon(material, &cap);
        }
    }

    /// A low-poly ellipsoid with `rings` latitude bands and `segments` meridians.
    pub fn ellipsoid(&mut self, material: usize, center: V3, radii: V3, rings: u32, segments: u32) {
        let point = |ring: u32, segment: u32| {
            let polar = std::f32::consts::PI * ring as f32 / rings as f32;
            let azimuth = TAU * segment as f32 / segments as f32;
            [
                center[0] + radii[0] * polar.sin() * azimuth.cos(),
                center[1] + radii[1] * polar.sin() * azimuth.sin(),
                center[2] + radii[2] * polar.cos(),
            ]
        };
        for ring in 0..rings {
            for segment in 0..segments {
                let next = (segment + 1) % segments;
                let a = point(ring, segment);
                let b = point(ring + 1, segment);
                let c = point(ring + 1, next);
                let d = point(ring, next);
                if ring == 0 {
                    self.polygon(material, &[a, b, c]);
                } else if ring + 1 == rings {
                    self.polygon(material, &[a, b, d]);
                } else {
                    self.polygon(material, &[a, b, c, d]);
                }
            }
        }
    }

    /// Packs the shapes into a scene with one element node named `name`; empty meshes are dropped.
    pub fn finish(self, name: &str) -> ImportedScene {
        let mut meshes = Vec::new();
        let mut materials = Vec::new();
        for (mesh, material) in self.meshes.into_iter().zip(self.materials) {
            if mesh.indices.is_empty() {
                continue;
            }
            materials.push(material);
            meshes.push(Mesh {
                material: Some(materials.len() - 1),
                ..mesh
            });
        }
        let root = SceneNode {
            name: name.to_owned(),
            class: String::new(),
            kind: NodeKind::Project,
            type_name: None,
            parent: None,
            children: vec![1],
            meshes: Vec::new(),
            level: None,
        };
        let element = SceneNode {
            name: name.to_owned(),
            class: "PROCEDURAL".into(),
            kind: NodeKind::Element,
            type_name: None,
            parent: Some(0),
            children: Vec::new(),
            meshes: (0..meshes.len()).collect(),
            level: None,
        };
        ImportedScene {
            // There is no "procedural" source format; OBJ is the closest plain-mesh format and the version and
            // application fields say where the geometry really came from.
            source: SourceInfo {
                format: SourceFormat::Obj,
                version: "procedural".into(),
                application: "Guanacaste Render".into(),
                length_unit: "m".into(),
                unit_scale: 1.0,
                file_size: 0,
            },
            nodes: vec![root, element],
            meshes,
            materials,
            levels: Vec::new(),
            origin_shift: [0.0; 3],
        }
    }
}
