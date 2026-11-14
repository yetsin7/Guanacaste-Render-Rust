//! Triangles and a bounding volume hierarchy (binned surface-area heuristic, near-first traversal) for ray queries.

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
    /// Index of the first of the three corner UVs in the scene's UV list, or [`NO_UV`].
    pub uv: u32,
}

/// Marks a triangle that has no texture coordinates.
pub const NO_UV: u32 = u32::MAX;

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

    /// Moller-Trumbore: `(t, u, v)` when the ray hits within `(t_min, t_max)`.
    fn intersect(
        &self,
        origin: Vec3,
        dir: Vec3,
        t_min: f32,
        t_max: f32,
    ) -> Option<(f32, f32, f32)> {
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
        (t > t_min && t < t_max).then_some((t, u, v))
    }
}

/// The nearest intersection of a ray.
#[derive(Debug, Clone, Copy)]
pub struct Hit {
    /// Distance along the ray.
    pub t: f32,
    /// Index of the triangle in [`Bvh::triangles`].
    pub triangle: usize,
    /// Barycentric weight of the second corner.
    pub u: f32,
    /// Barycentric weight of the third corner.
    pub v: f32,
}

/// A node of 32 bytes: the box and two packed words. A leaf stores its first triangle (flagged with [`LEAF`]) and
/// its triangle count; an inner node stores the left child with the split axis in the top bits, and the right child.
#[derive(Debug, Clone, Copy)]
struct Node {
    min: Vec3,
    max: Vec3,
    a: u32,
    b: u32,
}

/// Marks a leaf in [`Node::a`].
const LEAF: u32 = 1 << 31;
/// Bits of [`Node::a`] that hold the split axis of an inner node.
const AXIS_SHIFT: u32 = 29;

impl Node {
    fn leaf(min: Vec3, max: Vec3, start: usize, count: usize) -> Self {
        Self {
            min,
            max,
            a: LEAF | start as u32,
            b: count as u32,
        }
    }

    fn is_leaf(&self) -> bool {
        self.a & LEAF != 0
    }

    /// First triangle of a leaf.
    fn start(&self) -> usize {
        (self.a & !LEAF) as usize
    }

    /// Triangle count of a leaf.
    fn count(&self) -> usize {
        self.b as usize
    }

    /// Left child of an inner node.
    fn left(&self) -> u32 {
        self.a & ((1 << AXIS_SHIFT) - 1)
    }

    /// Right child of an inner node.
    fn right(&self) -> u32 {
        self.b
    }

    /// Split axis of an inner node.
    fn axis(&self) -> usize {
        ((self.a >> AXIS_SHIFT) & 3) as usize
    }
}

const LEAF_SIZE: usize = 4;
const BINS: usize = 12;

/// Triangles reordered for fast ray traversal.
#[derive(Debug, Default)]
pub struct Bvh {
    /// The triangles in tree order.
    pub triangles: Vec<Triangle>,
    nodes: Vec<Node>,
}

/// Per-primitive data used while building.
struct Prim {
    min: Vec3,
    max: Vec3,
    center: Vec3,
}

fn area(min: Vec3, max: Vec3) -> f32 {
    let d = max - min;
    if d.x < 0.0 {
        return 0.0;
    }
    2.0 * (d.x * d.y + d.y * d.z + d.z * d.x)
}

impl Bvh {
    /// Builds the hierarchy with a binned surface-area heuristic.
    pub fn build(triangles: Vec<Triangle>) -> Self {
        if triangles.is_empty() {
            return Self::default();
        }
        let prims: Vec<Prim> = triangles
            .iter()
            .map(|t| {
                let (min, max) = t.bounds();
                Prim {
                    min,
                    max,
                    center: (min + max) * 0.5,
                }
            })
            .collect();
        let mut order: Vec<u32> = (0..triangles.len() as u32).collect();
        let mut nodes = Vec::with_capacity(triangles.len() / 2 + 1);
        let count = order.len();
        build_node(&prims, &mut order, 0, count, &mut nodes);
        let triangles = order.iter().map(|&i| triangles[i as usize]).collect();
        Self { triangles, nodes }
    }

    /// Bounds of the whole content, or `None` when empty.
    pub fn bounds(&self) -> Option<(Vec3, Vec3)> {
        self.nodes.first().map(|n| (n.min, n.max))
    }

    /// The nearest hit with `t_min < t < t_max`.
    pub fn closest(&self, origin: Vec3, dir: Vec3, t_min: f32, t_max: f32) -> Option<Hit> {
        self.traverse(origin, dir, t_min, t_max, false)
    }

    /// Any hit with `t_min < t < t_max` (the first one found, not necessarily the nearest); cheaper for shadows.
    pub fn any_hit(&self, origin: Vec3, dir: Vec3, t_min: f32, t_max: f32) -> Option<Hit> {
        self.traverse(origin, dir, t_min, t_max, true)
    }

    fn traverse(
        &self,
        origin: Vec3,
        dir: Vec3,
        t_min: f32,
        t_max: f32,
        first_only: bool,
    ) -> Option<Hit> {
        if self.nodes.is_empty() {
            return None;
        }
        let inv = Vec3::new(1.0 / dir.x, 1.0 / dir.y, 1.0 / dir.z);
        let negative = [dir.x < 0.0, dir.y < 0.0, dir.z < 0.0];
        let mut best: Option<Hit> = None;
        let mut limit = t_max;
        // Nodes still to visit with the distance at which the ray enters their box: once a closer hit is known,
        // everything that starts beyond it is skipped without touching it.
        let mut stack = [(0u32, 0.0f32); 64];
        let root = entry(&self.nodes[0], origin, inv, t_min, limit)?;
        stack[0] = (0, root);
        let mut top = 1;
        while top > 0 {
            top -= 1;
            let (index, enters) = stack[top];
            if enters > limit {
                continue;
            }
            let node = &self.nodes[index as usize];
            if node.is_leaf() {
                let start = node.start();
                for (offset, tri) in self.triangles[start..start + node.count()]
                    .iter()
                    .enumerate()
                {
                    if let Some((t, u, v)) = tri.intersect(origin, dir, t_min, limit) {
                        limit = t;
                        best = Some(Hit {
                            t,
                            triangle: start + offset,
                            u,
                            v,
                        });
                        if first_only {
                            return best;
                        }
                    }
                }
            } else if top + 2 <= stack.len() {
                let (near, far) = if negative[node.axis()] {
                    (node.right(), node.left())
                } else {
                    (node.left(), node.right())
                };
                let near_hit = entry(&self.nodes[near as usize], origin, inv, t_min, limit);
                let far_hit = entry(&self.nodes[far as usize], origin, inv, t_min, limit);
                match (near_hit, far_hit) {
                    (Some(a), Some(b)) => {
                        // The closer box is visited first (it is pushed last).
                        let (first, second) = if a <= b {
                            ((near, a), (far, b))
                        } else {
                            ((far, b), (near, a))
                        };
                        stack[top] = second;
                        stack[top + 1] = first;
                        top += 2;
                    }
                    (Some(a), None) => {
                        stack[top] = (near, a);
                        top += 1;
                    }
                    (None, Some(b)) => {
                        stack[top] = (far, b);
                        top += 1;
                    }
                    (None, None) => {}
                }
            }
        }
        best
    }
}

/// The distance at which the ray enters the node's box within `t_min..t_max`, or `None` when it misses.
fn entry(node: &Node, origin: Vec3, inv: Vec3, t_min: f32, t_max: f32) -> Option<f32> {
    let t0 = (node.min - origin).mul_v(inv);
    let t1 = (node.max - origin).mul_v(inv);
    // `max`/`min` ignore a NaN produced by 0 * inf, which keeps rays parallel to a slab inside it.
    let near =
        t0.x.min(t1.x)
            .max(t0.y.min(t1.y))
            .max(t0.z.min(t1.z))
            .max(t_min);
    let far =
        t0.x.max(t1.x)
            .min(t0.y.max(t1.y))
            .min(t0.z.max(t1.z))
            .min(t_max);
    (near <= far).then_some(near)
}

/// The best binned split: `(axis, bin boundary)` or `None` when no split beats a leaf.
fn best_split(prims: &[Prim], items: &[u32], cmin: Vec3, cmax: Vec3) -> Option<(usize, usize)> {
    let mut best: Option<(f32, usize, usize)> = None;
    for axis in 0..3 {
        let extent = cmax.axis(axis) - cmin.axis(axis);
        if extent < 1e-9 {
            continue;
        }
        let mut counts = [0usize; BINS];
        let mut lo = [Vec3::splat(f32::MAX); BINS];
        let mut hi = [Vec3::splat(f32::MIN); BINS];
        for &i in items {
            let p = &prims[i as usize];
            let b = bin_of(p.center.axis(axis), cmin.axis(axis), extent);
            counts[b] += 1;
            lo[b] = lo[b].min(p.min);
            hi[b] = hi[b].max(p.max);
        }
        // Sweep from the right to get the cost of every suffix, then from the left.
        let mut right_cost = [0.0f32; BINS];
        let (mut rl, mut rh, mut rc) = (Vec3::splat(f32::MAX), Vec3::splat(f32::MIN), 0usize);
        for b in (1..BINS).rev() {
            rl = rl.min(lo[b]);
            rh = rh.max(hi[b]);
            rc += counts[b];
            right_cost[b] = area(rl, rh) * rc as f32;
        }
        let (mut ll, mut lh, mut lc) = (Vec3::splat(f32::MAX), Vec3::splat(f32::MIN), 0usize);
        for b in 0..BINS - 1 {
            ll = ll.min(lo[b]);
            lh = lh.max(hi[b]);
            lc += counts[b];
            let cost = area(ll, lh) * lc as f32 + right_cost[b + 1];
            if lc > 0 && lc < items.len() && best.is_none_or(|(c, _, _)| cost < c) {
                best = Some((cost, axis, b + 1));
            }
        }
    }
    best.map(|(_, axis, boundary)| (axis, boundary))
}

fn bin_of(value: f32, min: f32, extent: f32) -> usize {
    (((value - min) / extent * BINS as f32) as usize).min(BINS - 1)
}

fn build_node(
    prims: &[Prim],
    order: &mut [u32],
    start: usize,
    end: usize,
    nodes: &mut Vec<Node>,
) -> u32 {
    let items = &order[start..end];
    let mut min = Vec3::splat(f32::MAX);
    let mut max = Vec3::splat(f32::MIN);
    let mut cmin = min;
    let mut cmax = max;
    for &i in items {
        let p = &prims[i as usize];
        min = min.min(p.min);
        max = max.max(p.max);
        cmin = cmin.min(p.center);
        cmax = cmax.max(p.center);
    }
    let index = nodes.len() as u32;
    nodes.push(Node::leaf(min, max, start, end - start));
    if end - start <= LEAF_SIZE {
        return index;
    }
    let split = best_split(prims, items, cmin, cmax);
    let (axis, mid) = match split {
        Some((axis, boundary)) => {
            let (lo, extent) = (cmin.axis(axis), cmax.axis(axis) - cmin.axis(axis));
            let slice = &mut order[start..end];
            let mut left = 0;
            for k in 0..slice.len() {
                let c = prims[slice[k] as usize].center.axis(axis);
                if bin_of(c, lo, extent) < boundary {
                    slice.swap(k, left);
                    left += 1;
                }
            }
            (axis, start + left)
        }
        None => {
            // Degenerate centroids: split in the middle so the tree stays finite.
            let axis = 0;
            (axis, start + (end - start) / 2)
        }
    };
    let mid = if mid == start || mid == end {
        start + (end - start) / 2
    } else {
        mid
    };
    let left = build_node(prims, order, start, mid, nodes);
    let right = build_node(prims, order, mid, end, nodes);
    let node = &mut nodes[index as usize];
    node.a = left | ((axis as u32) << AXIS_SHIFT);
    node.b = right;
    index
}
