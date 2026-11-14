//! Pitched roofs: shed, gable and hip planes with their overhang, eave drop and the gable walls under the slopes.

use super::part::{Part, P3};
use super::surface::Surface;
use crate::features::import_export::logic::triangulate::{signed_area, triangulate};

/// Plan shape of a roof (the `Shape` of the document).
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Shape {
    Shed,
    Gable,
    Hip,
}

impl Shape {
    /// The shape stored as `value`; flat and unknown shapes are `None`.
    pub fn from_value(value: i64) -> Option<Self> {
        match value {
            1 => Some(Shape::Shed),
            2 => Some(Shape::Gable),
            3 => Some(Shape::Hip),
            _ => None,
        }
    }
}

/// Plan bounds `(min x, max x, min y, max y)`.
fn extent(points: &[[f64; 2]]) -> (f64, f64, f64, f64) {
    points.iter().fold(
        (f64::MAX, f64::MIN, f64::MAX, f64::MIN),
        |(a, b, c, d), p| (a.min(p[0]), b.max(p[0]), c.min(p[1]), d.max(p[1])),
    )
}

/// The boundary pushed out by `distance` along the bisector of every corner (exact for convex outlines).
pub fn grow(boundary: &[[f64; 2]], distance: f64) -> Vec<[f64; 2]> {
    let mut outline = boundary.to_vec();
    if signed_area(&outline) < 0.0 {
        outline.reverse();
    }
    let n = outline.len();
    let normal = |a: [f64; 2], b: [f64; 2]| {
        let (dx, dy) = (b[0] - a[0], b[1] - a[1]);
        let l = dx.hypot(dy).max(1e-12);
        [dy / l, -dx / l]
    };
    (0..n)
        .map(|i| {
            let (p, prev, next) = (outline[i], outline[(i + n - 1) % n], outline[(i + 1) % n]);
            let (n1, n2) = (normal(prev, p), normal(p, next));
            let k = distance / (1.0 + n1[0] * n2[0] + n1[1] * n2[1]).max(0.2);
            [p[0] + (n1[0] + n2[0]) * k, p[1] + (n1[1] + n2[1]) * k]
        })
        .collect()
}

/// Keeps the part of the convex-or-not `poly` where `a*x + b*y + c >= 0` (Sutherland-Hodgman).
fn clip(poly: &[[f64; 2]], a: f64, b: f64, c: f64) -> Vec<[f64; 2]> {
    let side = |p: [f64; 2]| a * p[0] + b * p[1] + c;
    let mut out = Vec::new();
    for i in 0..poly.len() {
        let (p, q) = (poly[i], poly[(i + 1) % poly.len()]);
        let (sp, sq) = (side(p), side(q));
        if sp >= 0.0 {
            out.push(p);
        }
        if (sp >= 0.0) != (sq >= 0.0) {
            let t = sp / (sp - sq);
            out.push([p[0] + (q[0] - p[0]) * t, p[1] + (q[1] - p[1]) * t]);
        }
    }
    out
}

/// A roof: its outline, shape, slope (degrees) and slab thickness.
pub struct Roof<'a> {
    pub outline: &'a [[f64; 2]],
    pub shape: Shape,
    pub pitch: f64,
    pub thickness: f64,
    /// Elevation of the eave (the roof plane's lowest edge).
    pub eave: f64,
}

impl Roof<'_> {
    /// Height of the roof plane above the eave at a plan point.
    pub fn rise(&self, p: [f64; 2]) -> f64 {
        let tan = self.pitch.to_radians().tan();
        let (min_x, max_x, min_y, max_y) = extent(self.outline);
        let along_x = max_x - min_x >= max_y - min_y;
        let (coordinate, minimum, span) = if along_x {
            (p[1], min_y, max_y - min_y)
        } else {
            (p[0], min_x, max_x - min_x)
        };
        let rise = match self.shape {
            Shape::Shed => tan * (coordinate - minimum),
            Shape::Gable => tan * (span / 2.0 - (coordinate - (minimum + span / 2.0)).abs()),
            Shape::Hip => {
                tan * (p[0] - min_x)
                    .min(max_x - p[0])
                    .min((p[1] - min_y).min(max_y - p[1]))
            }
        };
        rise.max(0.0)
    }

    /// The planar facets of the roof.
    fn facets(&self) -> Vec<Vec<[f64; 2]>> {
        let mut poly = self.outline.to_vec();
        if signed_area(&poly) < 0.0 {
            poly.reverse();
        }
        let (min_x, max_x, min_y, max_y) = extent(&poly);
        match self.shape {
            Shape::Shed => vec![poly],
            Shape::Gable => {
                if max_x - min_x >= max_y - min_y {
                    let m = (min_y + max_y) / 2.0;
                    vec![clip(&poly, 0.0, 1.0, -m), clip(&poly, 0.0, -1.0, m)]
                } else {
                    let m = (min_x + max_x) / 2.0;
                    vec![clip(&poly, 1.0, 0.0, -m), clip(&poly, -1.0, 0.0, m)]
                }
            }
            Shape::Hip => {
                let (sx, sy) = (min_x + max_x, min_y + max_y);
                vec![
                    // Left, right, bottom and top sectors: the edge whose distance is the smallest.
                    clip(
                        &clip(&clip(&poly, -2.0, 0.0, sx), -1.0, 1.0, min_x - min_y),
                        -1.0,
                        -1.0,
                        min_x + max_y,
                    ),
                    clip(
                        &clip(&clip(&poly, 2.0, 0.0, -sx), 1.0, 1.0, -(max_x + min_y)),
                        1.0,
                        -1.0,
                        max_y - max_x,
                    ),
                    clip(
                        &clip(&clip(&poly, 0.0, -2.0, sy), 1.0, -1.0, min_y - min_x),
                        -1.0,
                        -1.0,
                        min_y + max_x,
                    ),
                    clip(
                        &clip(&clip(&poly, 0.0, 2.0, -sy), 1.0, 1.0, -(min_x + max_y)),
                        -1.0,
                        1.0,
                        max_x - max_y,
                    ),
                ]
            }
        }
    }

    fn at(&self, p: [f64; 2], drop: f64) -> P3 {
        [
            p[0] as f32,
            p[1] as f32,
            (self.eave + self.rise(p) - drop) as f32,
        ]
    }

    /// Adds the tiles, the underside and the edge faces; with `gables` the walls that close the slopes.
    pub fn build(&self, part: &mut Part, gables: bool) {
        let boundary: Vec<[f64; 2]> = {
            let mut b = self.outline.to_vec();
            if signed_area(&b) < 0.0 {
                b.reverse();
            }
            b
        };
        for facet in self.facets().iter().filter(|f| f.len() >= 3) {
            for [a, b, c] in triangulate(facet, &[]) {
                let (a, b, c) = (facet[a], facet[b], facet[c]);
                part.tri(
                    Surface::Roof,
                    [self.at(a, 0.0), self.at(b, 0.0), self.at(c, 0.0)],
                );
                part.tri(
                    Surface::Floor,
                    [
                        self.at(c, self.thickness),
                        self.at(b, self.thickness),
                        self.at(a, self.thickness),
                    ],
                );
            }
            for i in 0..facet.len() {
                let (p, q) = (facet[i], facet[(i + 1) % facet.len()]);
                if on_boundary(&boundary, [(p[0] + q[0]) / 2.0, (p[1] + q[1]) / 2.0]) {
                    part.quad(
                        Surface::Floor,
                        self.at(p, self.thickness),
                        self.at(q, self.thickness),
                        self.at(q, 0.0),
                        self.at(p, 0.0),
                    );
                }
            }
        }
        if gables {
            self.gable_walls(part, &boundary);
        }
    }

    /// Along every boundary edge where the roof rises, a wall 20 cm thick from the eave underside to the roof underside.
    fn gable_walls(&self, part: &mut Part, boundary: &[[f64; 2]]) {
        let bottom = (self.eave - self.thickness) as f32;
        for i in 0..boundary.len() {
            let (from, to) = (boundary[i], boundary[(i + 1) % boundary.len()]);
            let length = (to[0] - from[0]).hypot(to[1] - from[1]);
            if length < 1e-6 {
                continue;
            }
            let inward = [
                -(to[1] - from[1]) / length * 0.2,
                (to[0] - from[0]) / length * 0.2,
            ];
            let steps = 8;
            for k in 0..steps {
                let (t0, t1) = (
                    f64::from(k) / f64::from(steps),
                    f64::from(k + 1) / f64::from(steps),
                );
                let lerp = |t: f64| {
                    [
                        from[0] + (to[0] - from[0]) * t,
                        from[1] + (to[1] - from[1]) * t,
                    ]
                };
                let (a, b) = (lerp(t0), lerp(t1));
                let top_a = (self.eave + self.rise(a) - self.thickness) as f32;
                let top_b = (self.eave + self.rise(b) - self.thickness) as f32;
                if top_a - bottom < 1e-3 && top_b - bottom < 1e-3 {
                    continue;
                }
                let outline = [
                    a,
                    b,
                    [b[0] + inward[0], b[1] + inward[1]],
                    [a[0] + inward[0], a[1] + inward[1]],
                ];
                part.convex_prism_h(
                    &outline,
                    &[bottom; 4],
                    &[top_a, top_b, top_b, top_a],
                    Surface::Wall,
                    Surface::Wall,
                );
            }
        }
    }
}

/// Whether `p` lies on the outline (within a millimeter).
fn on_boundary(boundary: &[[f64; 2]], p: [f64; 2]) -> bool {
    (0..boundary.len()).any(|i| {
        let (a, b) = (boundary[i], boundary[(i + 1) % boundary.len()]);
        let (dx, dy) = (b[0] - a[0], b[1] - a[1]);
        let l2 = dx * dx + dy * dy;
        if l2 < 1e-12 {
            return false;
        }
        let t = (((p[0] - a[0]) * dx + (p[1] - a[1]) * dy) / l2).clamp(0.0, 1.0);
        (a[0] + dx * t - p[0]).hypot(a[1] + dy * t - p[1]) < 1e-3
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    const RECT: [[f64; 2]; 4] = [[0.0, 0.0], [10.0, 0.0], [10.0, 6.0], [0.0, 6.0]];

    fn roof(shape: Shape) -> Roof<'static> {
        Roof {
            outline: &RECT,
            shape,
            pitch: 30.0,
            thickness: 0.2,
            eave: 3.0,
        }
    }

    #[test]
    fn a_gable_rises_to_a_ridge_along_the_long_side() {
        let r = roof(Shape::Gable);
        let ridge = 3.0f64 * 30f64.to_radians().tan();
        assert!((r.rise([5.0, 3.0]) - ridge).abs() < 1e-9);
        assert_eq!(r.rise([5.0, 0.0]), 0.0);
        let mut part = Part::new();
        r.build(&mut part, true);
        let top = part
            .into_groups()
            .iter()
            .flat_map(|(_, m)| m.positions.iter().map(|p| p[2]))
            .fold(f32::MIN, f32::max);
        assert!(
            (f64::from(top) - (3.0 + ridge)).abs() < 1e-4,
            "ridge at {top}"
        );
    }

    #[test]
    fn a_hip_has_four_facets_and_a_shed_one() {
        assert_eq!(roof(Shape::Hip).facets().len(), 4);
        assert_eq!(roof(Shape::Shed).facets().len(), 1);
        let area: f64 = roof(Shape::Hip)
            .facets()
            .iter()
            .map(|f| signed_area(f).abs())
            .sum();
        assert!(
            (area - 60.0).abs() < 1e-6,
            "facets cover the plan, got {area}"
        );
    }

    #[test]
    fn the_overhang_grows_the_outline() {
        let grown = grow(&RECT, 0.5);
        let (a, b, c, d) = extent(&grown);
        assert!(
            (a + 0.5).abs() < 1e-9
                && (b - 10.5).abs() < 1e-9
                && (c + 0.5).abs() < 1e-9
                && (d - 6.5).abs() < 1e-9
        );
    }
}
