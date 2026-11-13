//! Triangles and a bounding volume hierarchy for ray queries.

use super::vec3::Vec3;

/// A triangle stored as a corner and two edges.
#[derive(Debug, Clone, Copy)]
pub struct Triangle {
    /// First corner.
    pub v0: Vec3,
    /// Edge from the first to the second corner.
    pub e1: Vec3,
    /// Edge from the first to the third corner.
    pub e2: Vec3,
    /// Index into the render materials.
    pub material: u32,
}

impl Triangle {
    /// Unit geometric normal (zero for a degenerate triangle).
    pub fn normal(&self) -> Vec3 {
        self.e1.cross(self.e2).normalized()
    }

    fn bounds(&self) -> (Vec3, Vec3) {
        let v1 = self.v0 + self.e1;
        let v2 = self.v0 + self.e2;
        (self.v0.min(v1).min(v2), self.v0.max(v1).max(v2))
    }

    /// Moller-Trumbore: distance along the ray, when it hits within `(t_min, t_max)`.
    fn intersect(&self, origin: Vec3, dir: Vec3, t_min: f32, t_max: f32) -> Option<f32> {
        let p = dir.cross(self.e2);
        let det = self.e1.dot(p);
        if det.abs() < 1e-14 {
            return None;
        }
        let inv = 1.0 / det;
        let s = origin - self.v0;
        let u = s.dot(p) * inv;
        if !(0.0..=1.0).contains(&u) {
            return None;
        }
        let q = s.cross(self.e1);
        let v = dir.dot(q) * inv;
        if v < 0.0 || u + v > 1.0 {
            return None;
        }
        let t = self.e2.dot(q) * inv;
        (t > t_min && t < t_max).then_some(t)
    }
}

/// The nearest intersection of a ray.
#[derive(Debug, Clone, Copy)]
pub struct Hit {
    /// Distance along the ray.
    pub t: f32,
    /// Index of the triangle in [`Bvh::triangles`].
    pub triangle: usize,
}

#[derive(Debug, Clone, Copy)]
struct Node {
    min: Vec3,
    max: Vec3,
    /// First triangle of a leaf, or index of the left child of an inner node.
    first: u32,
    /// Triangle count of a leaf; zero for an inner node.
    count: u32,
    /// Index of the right child of an inner node.
    right: u32,
}

const LEAF_SIZE: usize = 4;

/// Triangles reordered for fast ray traversal.
#[derive(Debug, Default)]
pub struct Bvh {
    /// The triangles in tree order.
    pub triangles: Vec<Triangle>,
    nodes: Vec<Node>,
}

impl Bvh {
    /// Builds the hierarchy by median splits along the longest axis of the centroids.
    pub fn build(mut triangles: Vec<Triangle>) -> Self {
        if triangles.is_empty() {
            return Self::default();
        }
        let mut nodes = Vec::with_capacity(triangles.len() / 2 + 1);
        let count = triangles.len();
        build_node(&mut triangles, 0, count, &mut nodes);
        Self { triangles, nodes }
    }

    /// Bounds of the whole content, or `None` when empty.
    pub fn bounds(&self) -> Option<(Vec3, Vec3)> {
        self.nodes.first().map(|n| (n.min, n.max))
    }

    /// The nearest hit with `t_min < t < t_max`.
    pub fn closest(&self, origin: Vec3, dir: Vec3, t_min: f32, t_max: f32) -> Option<Hit> {
        if self.nodes.is_empty() {
            return None;
        }
        let inv = Vec3::new(1.0 / dir.x, 1.0 / dir.y, 1.0 / dir.z);
        let mut best: Option<Hit> = None;
        let mut limit = t_max;
        let mut stack = [0u32; 64];
        let mut top = 1;
        while top > 0 {
            top -= 1;
            let node = &self.nodes[stack[top] as usize];
            if !hits_box(node, origin, inv, t_min, limit) {
                continue;
            }
            if node.count > 0 {
                let start = node.first as usize;
                for (offset, tri) in self.triangles[start..start + node.count as usize]
                    .iter()
                    .enumerate()
                {
                    if let Some(t) = tri.intersect(origin, dir, t_min, limit) {
                        limit = t;
                        best = Some(Hit {
                            t,
                            triangle: start + offset,
                        });
                    }
                }
            } else if top + 2 <= stack.len() {
                stack[top] = node.right;
                stack[top + 1] = node.first;
                top += 2;
            }
        }
        best
    }
}

fn hits_box(node: &Node, origin: Vec3, inv: Vec3, t_min: f32, t_max: f32) -> bool {
    let mut near = t_min;
    let mut far = t_max;
    for axis in 0..3 {
        let o = origin.axis(axis);
        let i = inv.axis(axis);
        let mut t0 = (node.min.axis(axis) - o) * i;
        let mut t1 = (node.max.axis(axis) - o) * i;
        if t0 > t1 {
            std::mem::swap(&mut t0, &mut t1);
        }
        // `max`/`min` ignore a NaN produced by 0 * inf, which keeps rays parallel to a slab inside it.
        near = near.max(t0);
        far = far.min(t1);
        if near > far {
            return false;
        }
    }
    true
}

fn build_node(tris: &mut [Triangle], start: usize, end: usize, nodes: &mut Vec<Node>) -> u32 {
    let slice = &tris[start..end];
    let mut min = Vec3::new(f32::MAX, f32::MAX, f32::MAX);
    let mut max = Vec3::new(f32::MIN, f32::MIN, f32::MIN);
    let mut cmin = min;
    let mut cmax = max;
    for tri in slice {
        let (lo, hi) = tri.bounds();
        min = min.min(lo);
        max = max.max(hi);
        let c = (lo + hi) * 0.5;
        cmin = cmin.min(c);
        cmax = cmax.max(c);
    }
    let index = nodes.len() as u32;
    nodes.push(Node {
        min,
        max,
        first: start as u32,
        count: (end - start) as u32,
        right: 0,
    });
    if end - start <= LEAF_SIZE {
        return index;
    }
    let extent = cmax - cmin;
    let axis = if extent.x >= extent.y && extent.x >= extent.z {
        0
    } else if extent.y >= extent.z {
        1
    } else {
        2
    };
    let mid = (end - start) / 2;
    tris[start..end].select_nth_unstable_by(mid, |a, b| {
        let ca = a.bounds().0.axis(axis) + a.bounds().1.axis(axis);
        let cb = b.bounds().0.axis(axis) + b.bounds().1.axis(axis);
        ca.total_cmp(&cb)
    });
    let left = build_node(tris, start, start + mid, nodes);
    let right = build_node(tris, start + mid, end, nodes);
    let node = &mut nodes[index as usize];
    node.count = 0;
    node.first = left;
    node.right = right;
    index
}
