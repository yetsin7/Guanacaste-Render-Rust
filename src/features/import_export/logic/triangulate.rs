//! Polygon triangulation: ear clipping with hole bridging, in 2D and for planar 3D polygons.

use super::geometry::{cross, V3};

/// Signed area of a 2D polygon (positive when counter-clockwise).
pub fn signed_area(points: &[[f64; 2]]) -> f64 {
    let mut sum = 0.0;
    for index in 0..points.len() {
        let a = points[index];
        let b = points[(index + 1) % points.len()];
        sum += a[0] * b[1] - b[0] * a[1];
    }
    sum / 2.0
}

fn cross2(o: [f64; 2], a: [f64; 2], b: [f64; 2]) -> f64 {
    (a[0] - o[0]) * (b[1] - o[1]) - (a[1] - o[1]) * (b[0] - o[0])
}

fn inside(p: [f64; 2], a: [f64; 2], b: [f64; 2], c: [f64; 2]) -> bool {
    let d1 = cross2(a, b, p);
    let d2 = cross2(b, c, p);
    let d3 = cross2(c, a, p);
    d1 >= -1e-12 && d2 >= -1e-12 && d3 >= -1e-12
}

fn same(a: [f64; 2], b: [f64; 2]) -> bool {
    (a[0] - b[0]).abs() < 1e-12 && (a[1] - b[1]).abs() < 1e-12
}

/// Whether segments `a-b` and `c-d` cross at an interior point of both.
fn properly_intersect(a: [f64; 2], b: [f64; 2], c: [f64; 2], d: [f64; 2]) -> bool {
    if same(a, c) || same(a, d) || same(b, c) || same(b, d) {
        return false;
    }
    let d1 = cross2(a, b, c);
    let d2 = cross2(a, b, d);
    let d3 = cross2(c, d, a);
    let d4 = cross2(c, d, b);
    d1 * d2 < 0.0 && d3 * d4 < 0.0
}

/// Ear-clips the polygon given as indices into `points`; returns triangles (counter-clockwise).
fn ear_clip(points: &[[f64; 2]], mut poly: Vec<usize>) -> Vec<[usize; 3]> {
    let area: f64 = (0..poly.len())
        .map(|i| {
            let a = points[poly[i]];
            let b = points[poly[(i + 1) % poly.len()]];
            a[0] * b[1] - b[0] * a[1]
        })
        .sum();
    if area < 0.0 {
        poly.reverse();
    }
    let mut triangles = Vec::new();
    while poly.len() > 3 {
        let n = poly.len();
        let mut clipped = false;
        for i in 0..n {
            let (ia, ib, ic) = (poly[(i + n - 1) % n], poly[i], poly[(i + 1) % n]);
            let (a, b, c) = (points[ia], points[ib], points[ic]);
            if cross2(a, b, c) <= 1e-14 {
                continue;
            }
            let blocked = poly.iter().any(|&other| {
                let p = points[other];
                other != ia
                    && other != ib
                    && other != ic
                    && !same(p, a)
                    && !same(p, b)
                    && !same(p, c)
                    && inside(p, a, b, c)
            });
            if !blocked {
                triangles.push([ia, ib, ic]);
                poly.remove(i);
                clipped = true;
                break;
            }
        }
        if !clipped {
            // Degenerate input: drop a vertex so the loop always terminates.
            poly.remove(0);
        }
    }
    if poly.len() == 3 && cross2(points[poly[0]], points[poly[1]], points[poly[2]]).abs() > 1e-14 {
        triangles.push([poly[0], poly[1], poly[2]]);
    }
    triangles
}

/// Merges `hole` into the outer polygon `poly` through a bridge edge.
fn bridge(points: &[[f64; 2]], poly: &mut Vec<usize>, hole: &[usize], others: &[Vec<usize>]) {
    let start = (0..hole.len())
        .max_by(|&a, &b| points[hole[a]][0].total_cmp(&points[hole[b]][0]))
        .unwrap_or(0);
    let m = points[hole[start]];
    let mut candidates: Vec<usize> = (0..poly.len()).collect();
    candidates.sort_by(|&a, &b| {
        let da = (points[poly[a]][0] - m[0]).hypot(points[poly[a]][1] - m[1]);
        let db = (points[poly[b]][0] - m[0]).hypot(points[poly[b]][1] - m[1]);
        da.total_cmp(&db)
    });
    let edges_clear = |target: [f64; 2]| {
        let mut loops: Vec<&Vec<usize>> = others.iter().collect();
        loops.push(&*poly);
        for ring in loops {
            for i in 0..ring.len() {
                if properly_intersect(
                    m,
                    target,
                    points[ring[i]],
                    points[ring[(i + 1) % ring.len()]],
                ) {
                    return false;
                }
            }
        }
        for i in 0..hole.len() {
            if properly_intersect(
                m,
                target,
                points[hole[i]],
                points[hole[(i + 1) % hole.len()]],
            ) {
                return false;
            }
        }
        true
    };
    let chosen = candidates
        .iter()
        .copied()
        .find(|&index| edges_clear(points[poly[index]]))
        .unwrap_or(candidates[0]);
    let mut merged = Vec::with_capacity(poly.len() + hole.len() + 2);
    merged.extend_from_slice(&poly[..=chosen]);
    for step in 0..=hole.len() {
        merged.push(hole[(start + step) % hole.len()]);
    }
    merged.push(poly[chosen]);
    merged.extend_from_slice(&poly[chosen + 1..]);
    *poly = merged;
}

/// Triangulates `outer` with `holes`. Indices refer to the concatenation `outer ++ holes[0] ++ holes[1] ...`.
pub fn triangulate(outer: &[[f64; 2]], holes: &[Vec<[f64; 2]>]) -> Vec<[usize; 3]> {
    if outer.len() < 3 {
        return Vec::new();
    }
    let mut points: Vec<[f64; 2]> = outer.to_vec();
    let mut poly: Vec<usize> = (0..outer.len()).collect();
    if signed_area(outer) < 0.0 {
        poly.reverse();
    }
    let mut hole_rings: Vec<Vec<usize>> = Vec::new();
    for hole in holes.iter().filter(|hole| hole.len() >= 3) {
        let first = points.len();
        points.extend_from_slice(hole);
        let mut ring: Vec<usize> = (first..points.len()).collect();
        if signed_area(hole) > 0.0 {
            ring.reverse();
        }
        hole_rings.push(ring);
    }
    hole_rings.sort_by(|a, b| {
        let max_x = |ring: &Vec<usize>| {
            ring.iter()
                .map(|&i| points[i][0])
                .fold(f64::NEG_INFINITY, f64::max)
        };
        max_x(b).total_cmp(&max_x(a))
    });
    for index in 0..hole_rings.len() {
        let (before, rest) = hole_rings.split_at(index);
        bridge(&points, &mut poly, &rest[0], before);
    }
    ear_clip(&points, poly)
}

/// Triangulates a planar 3D polygon with holes. Returns the concatenated points `outer ++ holes...` and triangles
/// whose winding follows the outer loop's orientation.
pub fn triangulate_3d(outer: &[V3], holes: &[Vec<V3>]) -> (Vec<V3>, Vec<[usize; 3]>) {
    let mut normal = [0.0; 3];
    for index in 0..outer.len() {
        let a = outer[index];
        let b = outer[(index + 1) % outer.len()];
        let c = cross(a, b);
        for axis in 0..3 {
            normal[axis] += c[axis];
        }
    }
    let axis = (0..3)
        .max_by(|&a, &b| normal[a].abs().total_cmp(&normal[b].abs()))
        .unwrap_or(2);
    let (u, v) = ((axis + 1) % 3, (axis + 2) % 3);
    let project = |p: &V3| [p[u], p[v]];
    let outer_2d: Vec<[f64; 2]> = outer.iter().map(project).collect();
    let holes_2d: Vec<Vec<[f64; 2]>> = holes
        .iter()
        .map(|hole| hole.iter().map(project).collect())
        .collect();
    let mut triangles = triangulate(&outer_2d, &holes_2d);
    if normal[axis] < 0.0 {
        for triangle in &mut triangles {
            triangle.swap(1, 2);
        }
    }
    let mut points = outer.to_vec();
    for hole in holes.iter().filter(|hole| hole.len() >= 3) {
        points.extend_from_slice(hole);
    }
    (points, triangles)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn area_of(points: &[[f64; 2]], triangles: &[[usize; 3]]) -> f64 {
        triangles
            .iter()
            .map(|t| cross2(points[t[0]], points[t[1]], points[t[2]]).abs() / 2.0)
            .sum()
    }

    #[test]
    fn square_makes_two_triangles() {
        let square = [[0.0, 0.0], [2.0, 0.0], [2.0, 2.0], [0.0, 2.0]];
        let triangles = triangulate(&square, &[]);
        assert_eq!(triangles.len(), 2);
        assert!((area_of(&square, &triangles) - 4.0).abs() < 1e-9);
    }

    #[test]
    fn concave_l_shape_keeps_its_area() {
        let l = [
            [0.0, 0.0],
            [4.0, 0.0],
            [4.0, 1.0],
            [1.0, 1.0],
            [1.0, 3.0],
            [0.0, 3.0],
        ];
        let triangles = triangulate(&l, &[]);
        assert_eq!(triangles.len(), 4);
        assert!((area_of(&l, &triangles) - 6.0).abs() < 1e-9);
    }

    #[test]
    fn clockwise_input_is_accepted() {
        let square = [[0.0, 0.0], [0.0, 2.0], [2.0, 2.0], [2.0, 0.0]];
        assert_eq!(triangulate(&square, &[]).len(), 2);
    }

    #[test]
    fn a_hole_is_subtracted() {
        let outer = vec![[0.0, 0.0], [10.0, 0.0], [10.0, 10.0], [0.0, 10.0]];
        let hole = vec![[4.0, 4.0], [6.0, 4.0], [6.0, 6.0], [4.0, 6.0]];
        let triangles = triangulate(&outer, std::slice::from_ref(&hole));
        let mut all = outer.clone();
        all.extend(hole);
        assert!((area_of(&all, &triangles) - 96.0).abs() < 1e-6);
    }

    #[test]
    fn planar_3d_polygon_keeps_its_orientation() {
        let quad = [
            [0.0, 0.0, 1.0],
            [1.0, 0.0, 1.0],
            [1.0, 1.0, 1.0],
            [0.0, 1.0, 1.0],
        ];
        let (points, triangles) = triangulate_3d(&quad, &[]);
        assert_eq!(triangles.len(), 2);
        for t in triangles {
            let n = cross(
                super::super::geometry::sub(points[t[1]], points[t[0]]),
                super::super::geometry::sub(points[t[2]], points[t[0]]),
            );
            assert!(n[2] > 0.0);
        }
    }
}
