//! Cutting big triangles into smaller ones before the acceleration structure is built. Architectural models are
//! made of a few huge polygons (floors, ceilings, walls) whose boxes overlap everything around them; the tree can
//! not separate those, so every ray tests dozens of triangles. Small pieces keep the boxes tight.

use super::vec3::Vec3;

/// Longest edge (meters) a triangle may keep.
pub const MAX_EDGE: f32 = 1.5;
/// Triangles with an edge longer than this are the ground that reaches the horizon; they are left whole.
pub const HUGE_EDGE: f32 = 120.0;
/// Most times one triangle is halved.
const MAX_DEPTH: u32 = 8;

/// A triangle's corners with their texture coordinates, when it has any.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Piece {
    /// Corners.
    pub p: [Vec3; 3],
    /// Texture coordinates of the corners.
    pub uv: Option<[[f32; 2]; 3]>,
}

fn longest(p: &[Vec3; 3]) -> (usize, f32) {
    let edges = [
        (p[1] - p[0]).length(),
        (p[2] - p[1]).length(),
        (p[0] - p[2]).length(),
    ];
    (0..3)
        .map(|i| (i, edges[i]))
        .fold((0, 0.0), |best, e| if e.1 > best.1 { e } else { best })
}

fn middle(a: [f32; 2], b: [f32; 2]) -> [f32; 2] {
    [(a[0] + b[0]) * 0.5, (a[1] + b[1]) * 0.5]
}

/// Adds `piece` to `out`, halved across its longest edge as many times as it takes for every edge to be at most
/// [`MAX_EDGE`] (a piece of a huge triangle is left whole).
pub fn split(piece: Piece, out: &mut Vec<Piece>) {
    let mut stack = vec![(piece, 0u32)];
    while let Some((piece, depth)) = stack.pop() {
        let (edge, length) = longest(&piece.p);
        if length <= MAX_EDGE || length > HUGE_EDGE || depth >= MAX_DEPTH {
            out.push(piece);
            continue;
        }
        // Edge `edge` runs from corner `edge` to corner `edge + 1`; `far` is the corner opposite to it.
        let (i, j, far) = (edge, (edge + 1) % 3, (edge + 2) % 3);
        let mid = (piece.p[i] + piece.p[j]) * 0.5;
        let mid_uv = piece.uv.map(|uv| middle(uv[i], uv[j]));
        let part = |a: usize, b: usize, use_mid_first: bool| {
            let corners = if use_mid_first {
                [mid, piece.p[b], piece.p[far]]
            } else {
                [piece.p[a], mid, piece.p[far]]
            };
            let uv = piece.uv.zip(mid_uv).map(|(uv, m)| {
                if use_mid_first {
                    [m, uv[b], uv[far]]
                } else {
                    [uv[a], m, uv[far]]
                }
            });
            Piece { p: corners, uv }
        };
        stack.push((part(i, j, false), depth + 1));
        stack.push((part(i, j, true), depth + 1));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn area(p: &[Vec3; 3]) -> f32 {
        (p[1] - p[0]).cross(p[2] - p[0]).length() * 0.5
    }

    #[test]
    fn small_triangles_stay_whole() {
        let mut out = Vec::new();
        let p = [
            Vec3::new(0.0, 0.0, 0.0),
            Vec3::new(1.0, 0.0, 0.0),
            Vec3::new(0.0, 1.0, 0.0),
        ];
        split(Piece { p, uv: None }, &mut out);
        assert_eq!(out.len(), 1);
    }

    #[test]
    fn a_big_triangle_is_cut_into_pieces_with_the_same_total_area() {
        let p = [
            Vec3::new(0.0, 0.0, 0.0),
            Vec3::new(10.0, 0.0, 0.0),
            Vec3::new(0.0, 8.0, 0.0),
        ];
        let mut out = Vec::new();
        split(Piece { p, uv: None }, &mut out);
        assert!(out.len() > 16);
        let total: f32 = out.iter().map(|piece| area(&piece.p)).sum();
        assert!((total - area(&p)).abs() < 1e-3, "{total} vs {}", area(&p));
        for piece in &out {
            assert!(longest(&piece.p).1 <= MAX_EDGE + 1e-4);
        }
    }

    #[test]
    fn texture_coordinates_follow_the_cuts() {
        let p = [
            Vec3::new(0.0, 0.0, 0.0),
            Vec3::new(6.0, 0.0, 0.0),
            Vec3::new(0.0, 6.0, 0.0),
        ];
        // UVs equal to the position in meters / 6: every piece's UV must still match its corners.
        let uv = [[0.0, 0.0], [1.0, 0.0], [0.0, 1.0]];
        let mut out = Vec::new();
        split(Piece { p, uv: Some(uv) }, &mut out);
        for piece in &out {
            let uv = piece.uv.expect("uv kept");
            for k in 0..3 {
                assert!((uv[k][0] - piece.p[k].x / 6.0).abs() < 1e-4);
                assert!((uv[k][1] - piece.p[k].y / 6.0).abs() < 1e-4);
            }
        }
    }

    #[test]
    fn the_horizon_plane_is_left_alone() {
        let big = 30_000.0;
        let p = [
            Vec3::new(-big, -big, 0.0),
            Vec3::new(big, -big, 0.0),
            Vec3::new(big, big, 0.0),
        ];
        let mut out = Vec::new();
        split(Piece { p, uv: None }, &mut out);
        assert_eq!(out.len(), 1);
    }
}
