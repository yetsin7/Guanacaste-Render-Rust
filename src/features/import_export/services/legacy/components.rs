//! Residential components (furniture, appliances, sanitary ware, a parked car) drawn as small compositions of
//! boxes and prisms in their own material, scaled to the width, depth and height of the instance.

use super::axis::round;
use super::axis::Placement;
use super::part::Part;
use super::surface::Surface::{self, *};

/// The box that bounds a component, in meters, centered on its position with the front toward -Y.
pub struct Size {
    pub w: f64,
    pub d: f64,
    pub h: f64,
}

/// Draws the component with the stored `shape` number (the original's `ResidentialComponentShape`).
pub fn build(part: &mut Part, at: &Placement, shape: i64, s: &Size) {
    let (hw, hd) = (s.w / 2.0, s.d / 2.0);
    // A box given as fractions of the bounding box: x and y from -1 to 1, z from 0 to 1.
    let mut b = |key: Surface, x: [f64; 2], y: [f64; 2], z: [f64; 2]| {
        at.cuboid(
            part,
            key,
            [x[0] * hw, x[1] * hw],
            [y[0] * hd, y[1] * hd],
            [z[0] * s.h, z[1] * s.h],
        );
    };
    match shape {
        1 | 24 => {
            b(DoorLeaf, [-1.0, 1.0], [-1.0, 1.0], [0.94, 1.0]);
            for (x, y) in [(-0.9, -0.88), (0.9, -0.88), (-0.9, 0.88), (0.9, 0.88)] {
                b(
                    DoorLeaf,
                    [x - 0.06, x + 0.06],
                    [y - 0.06, y + 0.06],
                    [0.0, 0.94],
                );
            }
        }
        2 | 26 | 27 => {
            let seat = if shape == 27 { 0.9 } else { 0.5 };
            b(Upholstery, [-1.0, 1.0], [-1.0, 1.0], [seat - 0.08, seat]);
            if shape != 27 {
                b(DoorLeaf, [-1.0, 1.0], [0.8, 1.0], [seat, 1.0]);
            }
            for (x, y) in [(-0.88, -0.88), (0.88, -0.88), (-0.88, 0.88), (0.88, 0.88)] {
                b(
                    Metal,
                    [x - 0.07, x + 0.07],
                    [y - 0.07, y + 0.07],
                    [0.0, seat - 0.08],
                );
            }
        }
        3 | 25 | 28 => {
            b(DoorLeaf, [-1.0, 1.0], [-1.0, 1.0], [0.0, 0.12]);
            b(Upholstery, [-0.84, 0.84], [-1.0, 0.7], [0.12, 0.5]);
            if shape != 28 {
                b(Upholstery, [-0.84, 0.84], [0.62, 1.0], [0.12, 1.0]);
                b(Upholstery, [-1.0, -0.84], [-1.0, 1.0], [0.12, 0.72]);
                b(Upholstery, [0.84, 1.0], [-1.0, 1.0], [0.12, 0.72]);
            }
        }
        4 | 29 | 30 => {
            b(DoorLeaf, [-1.0, 1.0], [-1.0, 1.0], [0.0, 0.3]);
            b(Bedding, [-0.96, 0.96], [-0.96, 0.96], [0.3, 0.55]);
            b(DoorLeaf, [-1.0, 1.0], [0.92, 1.0], [0.0, 1.0]);
            for x in [-0.5, 0.5] {
                b(Bedding, [x - 0.4, x + 0.4], [0.5, 0.9], [0.55, 0.68]);
            }
        }
        5 => {
            b(Ceramic, [-0.5, 0.5], [-0.5, 0.5], [0.0, 0.28]);
            let foliage = round(at.at, s.w * 0.9, s.d * 0.9, 10);
            let placed: Vec<[f64; 2]> = foliage
                .iter()
                .map(|p| at.plan(p[0] - at.at[0], p[1] - at.at[1]))
                .collect();
            part.convex_prism(&placed, at.z + s.h * 0.28, at.z + s.h, Foliage, Foliage);
        }
        6 | 31 | 13 => {
            b(DoorLeaf, [-1.0, 1.0], [-1.0, 1.0], [0.0, 0.94]);
            b(WallStone, [-1.0, 1.0], [-1.0, 1.0], [0.94, 1.0]);
            b(Metal, [-0.8, 0.8], [-1.01, -1.0], [0.6, 0.64]);
            if shape != 6 {
                b(Ceramic, [-0.5, 0.5], [-0.6, 0.6], [0.96, 1.0]);
            }
        }
        7 => {
            b(DoorLeaf, [-1.0, 1.0], [-1.0, 1.0], [0.0, 1.0]);
            b(Metal, [-0.8, 0.8], [-1.01, -1.0], [0.1, 0.14]);
        }
        8 | 9 | 10 | 11 | 41 | 42 | 37 => {
            b(Metal, [-1.0, 1.0], [-1.0, 1.0], [0.0, 1.0]);
            b(TintedGlass, [-0.7, 0.7], [-1.02, -1.0], [0.55, 0.9]);
        }
        12 | 34 | 35 => {
            b(Ceramic, [-0.9, 0.9], [-0.2, 1.0], [0.0, 0.55]);
            b(Ceramic, [-0.8, 0.8], [-1.0, -0.1], [0.0, 0.4]);
            b(Ceramic, [-0.7, 0.7], [0.5, 1.0], [0.55, 1.0]);
        }
        14 => {
            b(Ceramic, [-1.0, 1.0], [-1.0, 1.0], [0.0, 1.0]);
            b(Water, [-0.85, 0.85], [-0.8, 0.8], [0.99, 1.0]);
        }
        15 => {
            for (x, y) in [([-1.0, 1.0], [-1.0, -0.97]), ([0.97, 1.0], [-1.0, 1.0])] {
                b(Glass, x, y, [0.05, 1.0]);
            }
            b(Ceramic, [-1.0, 1.0], [-1.0, 1.0], [0.0, 0.05]);
        }
        16 | 17 | 18 | 36 | 33 | 32 | 19 | 40 => furniture(part, at, shape, s),
        20 => b(Upholstery, [-1.0, 1.0], [-1.0, 1.0], [0.0, 1.0]),
        21 | 22 | 23 | 38 | 39 | 43 | 44 => accessories(part, at, shape, s),
        _ => b(Structure, [-1.0, 1.0], [-1.0, 1.0], [0.0, 1.0]),
    }
}

fn furniture(part: &mut Part, at: &Placement, shape: i64, s: &Size) {
    let (hw, hd) = (s.w / 2.0, s.d / 2.0);
    let mut b = |key: Surface, x: [f64; 2], y: [f64; 2], z: [f64; 2]| {
        at.cuboid(
            part,
            key,
            [x[0] * hw, x[1] * hw],
            [y[0] * hd, y[1] * hd],
            [z[0] * s.h, z[1] * s.h],
        );
    };
    match shape {
        16 => {
            b(DoorLeaf, [-1.0, 1.0], [-1.0, 1.0], [0.94, 1.0]);
            b(DoorLeaf, [-1.0, -0.88], [-1.0, 1.0], [0.0, 0.94]);
            b(DoorLeaf, [0.88, 1.0], [-1.0, 1.0], [0.0, 0.94]);
        }
        17 | 18 => {
            b(DoorLeaf, [-1.0, 1.0], [-1.0, 1.0], [0.0, 1.0]);
            let rows = if shape == 17 { 3 } else { 5 };
            for r in 1..rows {
                let z = f64::from(r) / f64::from(rows);
                b(Metal, [-1.0, 1.0], [-1.01, -1.0], [z - 0.01, z + 0.01]);
            }
        }
        36 => b(Mirror, [-1.0, 1.0], [-1.0, -0.9], [0.0, 1.0]),
        33 => {
            b(Ceramic, [-0.3, 0.3], [-0.2, 0.3], [0.0, 0.75]);
            b(Ceramic, [-1.0, 1.0], [-1.0, 1.0], [0.75, 1.0]);
        }
        32 => b(Metal, [-1.0, 1.0], [-1.0, 1.0], [0.0, 1.0]),
        19 => b(TintedGlass, [-1.0, 1.0], [-0.1, 0.1], [0.0, 1.0]),
        _ => {
            // Parasol: a pole and a wide canopy.
            b(Metal, [-0.03, 0.03], [-0.03, 0.03], [0.0, 1.0]);
            b(Upholstery, [-1.0, 1.0], [-1.0, 1.0], [0.9, 1.0]);
        }
    }
}

fn accessories(part: &mut Part, at: &Placement, shape: i64, s: &Size) {
    let (hw, hd) = (s.w / 2.0, s.d / 2.0);
    let mut b = |key: Surface, x: [f64; 2], y: [f64; 2], z: [f64; 2]| {
        at.cuboid(
            part,
            key,
            [x[0] * hw, x[1] * hw],
            [y[0] * hd, y[1] * hd],
            [z[0] * s.h, z[1] * s.h],
        );
    };
    match shape {
        21 => {
            b(Metal, [-0.05, 0.05], [-0.05, 0.05], [0.0, 0.85]);
            b(LightOff, [-0.5, 0.5], [-0.5, 0.5], [0.85, 1.0]);
        }
        22 => b(LightOff, [-1.0, 1.0], [-1.0, 1.0], [0.0, 1.0]),
        23 => {
            b(Metal, [-1.0, 1.0], [-1.0, 1.0], [0.55, 1.0]);
            b(Metal, [-0.8, -0.7], [-0.8, -0.7], [0.0, 0.55]);
            b(Metal, [0.7, 0.8], [0.7, 0.8], [0.0, 0.55]);
        }
        38 | 39 => {
            b(Upholstery, [-1.0, 1.0], [-1.0, 1.0], [0.3, 0.5]);
            b(Metal, [-0.9, 0.9], [-0.9, 0.9], [0.0, 0.3]);
        }
        _ => car(part, at, s, shape == 44),
    }
}

/// A parked sedan: paint body, a glazed cabin, four wheels.
fn car(part: &mut Part, at: &Placement, s: &Size, silver: bool) {
    let (hw, hd) = (s.w / 2.0, s.d / 2.0);
    let paint = if silver { CarPaintSilver } else { CarPaint };
    let mut b = |key: Surface, x: [f64; 2], y: [f64; 2], z: [f64; 2]| {
        at.cuboid(
            part,
            key,
            [x[0] * hw, x[1] * hw],
            [y[0] * hd, y[1] * hd],
            [z[0] * s.h, z[1] * s.h],
        );
    };
    b(paint, [-0.95, 0.95], [-1.0, 1.0], [0.18, 0.55]);
    b(TintedGlass, [-0.8, 0.8], [-0.45, 0.5], [0.55, 0.95]);
    b(paint, [-0.8, 0.8], [-0.4, 0.45], [0.95, 1.0]);
    for (x, y) in [(-1.0, -0.62), (1.0, -0.62), (-1.0, 0.62), (1.0, 0.62)] {
        b(
            Rubber,
            [x - 0.08, x + 0.08],
            [y - 0.16, y + 0.16],
            [0.0, 0.34],
        );
    }
}
