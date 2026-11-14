//! Boolean subtraction of convex cutters (openings, half-spaces, extruded boxes) from triangle soups.

use super::piece::Piece;
use crate::features::import_export::logic::geometry::{add, cross, dot, normalize, scale, sub, V3};

const EPS: f64 = 1e-6;

/// The inside of a plane is `dot(n, x) - d <= 0`.
#[derive(Debug, Clone, Copy)]
pub struct Plane {
    /// Unit normal pointing outside.
    pub n: V3,
    /// Offset along the normal.
    pub d: f64,
}

impl Plane {
    /// Plane through `point` with outward `normal`.
    pub fn new(point: V3, normal: V3) -> Self {
        let n = normalize(normal);
        Self {
            n,
            d: dot(n, point),
        }
    }

    fn side(&self, p: V3) -> f64 {
        dot(self.n, p) - self.d
    }

    fn same_as(&self, other: &Plane) -> bool {
        dot(self.n, other.n) > 1.0 - 1e-9 && (self.d - other.d).abs() < 1e-6
    }
}

/// A convex region as the intersection of half-spaces, with optional bounds.
#[derive(Debug, Clone)]
pub struct Convex {
    /// Bounding planes.
    pub planes: Vec<Plane>,
    bounds: Option<(V3, V3)>,
}

fn bounds_of(points: &[V3]) -> (V3, V3) {
    let mut min = [f64::INFINITY; 3];
    let mut max = [f64::NEG_INFINITY; 3];
    for p in points {
        for axis in 0..3 {
            min[axis] = min[axis].min(p[axis]);
            max[axis] = max[axis].max(p[axis]);
        }
    }
    (min, max)
}

impl Convex {
    /// A region from planes (duplicates are merged) with optional bounds.
    pub fn from_planes(planes: Vec<Plane>, bounds: Option<(V3, V3)>) -> Self {
        let mut unique: Vec<Plane> = Vec::new();
        for plane in planes {
            if !unique.iter().any(|other| other.same_as(&plane)) {
                unique.push(plane);
            }
        }
        Self {
            planes: unique,
            bounds,
        }
    }

    /// The closed solid made by `piece` when it is convex; `None` otherwise.
    pub fn from_piece(piece: &Piece) -> Option<Self> {
        if piece.indices.len() < 12 {
            return None;
        }
        let mut triangles: Vec<(V3, V3)> = Vec::new();
        let mut volume = 0.0;
        for tri in piece.indices.chunks(3) {
            let [a, b, c] = [tri[0], tri[1], tri[2]].map(|i| piece.positions[i as usize]);
            let normal = cross(sub(b, a), sub(c, a));
            volume += dot(a, cross(b, c)) / 6.0;
            if dot(normal, normal) > 1e-18 {
                triangles.push((a, normal));
            }
        }
        let flip = if volume < 0.0 { -1.0 } else { 1.0 };
        let planes: Vec<Plane> = triangles
            .iter()
            .map(|(a, n)| Plane::new(*a, scale(*n, flip)))
            .collect();
        let convex = Self::from_planes(planes, Some(bounds_of(&piece.positions)));
        let tolerance = 1e-5;
        let fits = piece.positions.iter().all(|p| {
            convex
                .planes
                .iter()
                .all(|plane| plane.side(*p) <= tolerance)
        });
        (fits && convex.planes.len() >= 4).then_some(convex)
    }

    /// Whether the triangle is entirely outside some plane (so it cannot touch the region).
    fn separates(&self, tri: &[V3; 3]) -> bool {
        self.planes
            .iter()
            .any(|plane| tri.iter().all(|p| plane.side(*p) > EPS))
    }

    fn overlaps(&self, min: V3, max: V3) -> bool {
        self.bounds.is_none_or(|(bmin, bmax)| {
            (0..3).all(|axis| min[axis] <= bmax[axis] + EPS && max[axis] >= bmin[axis] - EPS)
        })
    }
}

/// Splits a convex polygon by `plane` into (inside, outside) parts.
fn split(poly: &[V3], plane: &Plane) -> (Vec<V3>, Vec<V3>) {
    let (mut inside, mut outside) = (Vec::new(), Vec::new());
    let sides: Vec<f64> = poly.iter().map(|p| plane.side(*p) - EPS).collect();
    for i in 0..poly.len() {
        let j = (i + 1) % poly.len();
        let (a, b, sa, sb) = (poly[i], poly[j], sides[i], sides[j]);
        if sa <= 0.0 {
            inside.push(a);
        } else {
            outside.push(a);
        }
        if (sa <= 0.0) != (sb <= 0.0) {
            let t = sa / (sa - sb);
            let cut = add(a, scale(sub(b, a), t));
            inside.push(cut);
            outside.push(cut);
        }
    }
    (inside, outside)
}

fn clip_inside(poly: Vec<V3>, plane: &Plane) -> Vec<V3> {
    if poly.len() < 3 {
        return poly;
    }
    split(&poly, plane).0
}

fn emit(out: &mut Piece, poly: &[V3]) {
    if poly.len() < 3 {
        return;
    }
    let base: Vec<u32> = poly.iter().map(|p| out.vertex(*p)).collect();
    for i in 1..poly.len() - 1 {
        out.triangle(base[0], base[i], base[i + 1]);
    }
}

fn area_is_tiny(poly: &[V3]) -> bool {
    let mut sum = [0.0; 3];
    for i in 1..poly.len().saturating_sub(1) {
        let c = cross(sub(poly[i], poly[0]), sub(poly[i + 1], poly[0]));
        sum = add(sum, c);
    }
    dot(sum, sum) < 1e-12
}

/// The surface of the cutter that lies inside the host, facing into the hole.
fn reveals(host: &Piece, cutter: &Convex, hull: Option<&Convex>, out: &mut Piece) {
    let Some(hull) = hull else { return };
    let (min, max) = bounds_of(&host.positions);
    let center = scale(add(min, max), 0.5);
    let size = dot(sub(max, min), sub(max, min)).sqrt() * 2.0 + 1.0;
    for (index, plane) in cutter.planes.iter().enumerate() {
        let origin = sub(center, scale(plane.n, plane.side(center)));
        let hint = if plane.n[2].abs() < 0.9 {
            [0.0, 0.0, 1.0]
        } else {
            [1.0, 0.0, 0.0]
        };
        let u = normalize(cross(plane.n, hint));
        let v = cross(plane.n, u);
        let corner = |a: f64, b: f64| add(origin, add(scale(u, a * size), scale(v, b * size)));
        // Reversed so the face looks into the cutter.
        let mut poly = vec![
            corner(-1.0, -1.0),
            corner(-1.0, 1.0),
            corner(1.0, 1.0),
            corner(1.0, -1.0),
        ];
        for (other_index, other) in cutter.planes.iter().enumerate() {
            if other_index != index {
                poly = clip_inside(poly, other);
            }
        }
        for plane in &hull.planes {
            poly = clip_inside(poly, plane);
        }
        let on_host_face = hull
            .planes
            .iter()
            .any(|h| poly.iter().all(|p| h.side(*p).abs() < 1e-5));
        if poly.len() >= 3 && !on_host_face && !area_is_tiny(&poly) {
            emit(out, &poly);
        }
    }
}

/// `host` minus `cutter`. Reveal faces are added when the host is convex.
pub fn subtract(host: &Piece, cutter: &Convex) -> Piece {
    let mut out = Piece {
        color: host.color,
        ..Piece::default()
    };
    let mut removed = false;
    for tri in host.indices.chunks(3) {
        let t = [tri[0], tri[1], tri[2]].map(|i| host.positions[i as usize]);
        let (min, max) = bounds_of(&t);
        if !cutter.overlaps(min, max) || cutter.separates(&t) {
            emit(&mut out, &t);
            continue;
        }
        removed = true;
        let mut rest = t.to_vec();
        for plane in &cutter.planes {
            let (inside, outside) = split(&rest, plane);
            emit(&mut out, &outside);
            rest = inside;
            if rest.len() < 3 {
                break;
            }
        }
    }
    if removed {
        let hull = Convex::from_piece(host);
        reveals(host, cutter, hull.as_ref(), &mut out);
    }
    out
}

/// Subtracts every cutter from `host`.
pub fn subtract_all(host: Piece, cutters: &[Convex]) -> Piece {
    let mut current = host;
    for cutter in cutters {
        current = subtract(&current, cutter);
    }
    current
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cuboid(min: V3, max: V3) -> Piece {
        let mut piece = Piece::default();
        let c = |x: usize, y: usize, z: usize| {
            [
                if x == 0 { min[0] } else { max[0] },
                if y == 0 { min[1] } else { max[1] },
                if z == 0 { min[2] } else { max[2] },
            ]
        };
        let ids: Vec<u32> = (0..8)
            .map(|i| piece.vertex(c(i & 1, (i >> 1) & 1, (i >> 2) & 1)))
            .collect();
        let quads = [
            [0, 2, 3, 1],
            [4, 5, 7, 6],
            [0, 1, 5, 4],
            [2, 6, 7, 3],
            [0, 4, 6, 2],
            [1, 3, 7, 5],
        ];
        for q in quads {
            piece.quad(ids[q[0]], ids[q[1]], ids[q[2]], ids[q[3]]);
        }
        piece
    }

    fn area(piece: &Piece) -> f64 {
        piece
            .indices
            .chunks(3)
            .map(|t| {
                let [a, b, c] = [t[0], t[1], t[2]].map(|i| piece.positions[i as usize]);
                let n = cross(sub(b, a), sub(c, a));
                dot(n, n).sqrt() / 2.0
            })
            .sum()
    }

    #[test]
    fn box_is_convex_and_subtraction_cuts_a_through_hole() {
        let wall = cuboid([0.0, 0.0, 0.0], [4.0, 0.2, 3.0]);
        let hole = cuboid([1.0, -0.1, 1.0], [2.0, 0.3, 2.0]);
        let cutter = Convex::from_piece(&hole).expect("a box is convex");
        let cut = subtract(&wall, &cutter);
        let before = area(&wall);
        // Two 1x1 faces are removed and four 1x0.2 reveals are added.
        let expected = before - 2.0 + 4.0 * 0.2;
        assert!((area(&cut) - expected).abs() < 1e-4, "{}", area(&cut));
    }

    #[test]
    fn half_space_clips_the_top_of_a_box() {
        let block = cuboid([0.0, 0.0, 0.0], [1.0, 1.0, 1.0]);
        let removed =
            Convex::from_planes(vec![Plane::new([0.0, 0.0, 0.5], [0.0, 0.0, -1.0])], None);
        let cut = subtract(&block, &removed);
        let top = cut
            .positions
            .iter()
            .map(|p| p[2])
            .fold(f64::NEG_INFINITY, f64::max);
        assert!((top - 0.5).abs() < 1e-9);
        assert!((area(&cut) - (1.0 + 4.0 * 0.5 + 1.0)).abs() < 1e-4);
    }

    #[test]
    fn non_convex_pieces_are_rejected_as_cutters() {
        let mut l = cuboid([0.0, 0.0, 0.0], [2.0, 1.0, 1.0]);
        let extra = cuboid([0.0, 0.0, 1.0], [1.0, 1.0, 2.0]);
        let base = l.positions.len() as u32;
        l.positions.extend(extra.positions.iter().copied());
        l.indices.extend(extra.indices.iter().map(|i| i + base));
        assert!(Convex::from_piece(&l).is_none());
    }
}
