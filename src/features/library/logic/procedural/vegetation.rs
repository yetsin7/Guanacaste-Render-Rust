//! Procedural vegetation: broadleaf and conifer trees, bushes and palms, each varied by seed.

use super::builder::ShapeBuilder;
use super::rng::Rng;
use crate::features::import_export::models::ImportedScene;
use std::f32::consts::TAU;

const LEAF_GREENS: [u32; 6] = [0x3f7a2e, 0x4d8a34, 0x2f6b3a, 0x5a8f2c, 0x6b9a3a, 0x356b2a];
const BARKS: [u32; 4] = [0x5b4030, 0x6a4a33, 0x4e3a2c, 0x75553c];

/// A broadleaf tree: a tapered trunk and a crown of overlapping low-poly blobs.
pub fn tree(seed: u64) -> ImportedScene {
    let mut rng = Rng::new(seed);
    let mut shapes = ShapeBuilder::default();
    let bark = shapes.material("Corteza", rng.pick(&BARKS));
    let leaves = shapes.material("Follaje", rng.pick(&LEAF_GREENS));
    let height = rng.range(2.2, 4.0);
    let radius = rng.range(0.12, 0.24);
    let top = [rng.range(-0.2, 0.2), rng.range(-0.2, 0.2), height];
    shapes.frustum(bark, [0.0; 3], top, radius, radius * 0.6, 7);
    let crown = rng.range(1.4, 2.6);
    let blobs = rng.int(3, 5);
    let center_z = height + crown * 0.55;
    shapes.ellipsoid(
        leaves,
        [top[0], top[1], center_z],
        [crown, crown, crown * rng.range(0.7, 0.95)],
        4,
        7,
    );
    for index in 0..blobs {
        let angle = TAU * index as f32 / blobs as f32 + rng.range(0.0, 0.8);
        let reach = crown * rng.range(0.45, 0.75);
        let size = crown * rng.range(0.45, 0.65);
        let center = [
            top[0] + angle.cos() * reach,
            top[1] + angle.sin() * reach,
            center_z + rng.range(-0.4, 0.5) * crown,
        ];
        shapes.ellipsoid(leaves, center, [size, size, size * 0.85], 3, 6);
    }
    shapes.finish("Árbol")
}

/// A conifer: a straight trunk under stacked cones.
pub fn conifer(seed: u64) -> ImportedScene {
    let mut rng = Rng::new(seed);
    let mut shapes = ShapeBuilder::default();
    let bark = shapes.material("Corteza", rng.pick(&BARKS));
    let needles = shapes.material("Acículas", rng.pick(&[0x2b5a34, 0x24502e, 0x31623b]));
    let trunk = rng.range(1.0, 1.8);
    shapes.frustum(bark, [0.0; 3], [0.0, 0.0, trunk], 0.2, 0.14, 6);
    let tiers = rng.int(3, 5);
    let mut base = trunk * 0.8;
    let mut radius = rng.range(1.6, 2.4);
    for _ in 0..tiers {
        let height = radius * rng.range(1.2, 1.6);
        shapes.frustum(
            needles,
            [0.0, 0.0, base],
            [0.0, 0.0, base + height],
            radius,
            0.0,
            8,
        );
        base += height * 0.5;
        radius *= rng.range(0.68, 0.8);
    }
    shapes.finish("Conífera")
}

/// A bush: a few blobs resting on the ground.
pub fn bush(seed: u64) -> ImportedScene {
    let mut rng = Rng::new(seed);
    let mut shapes = ShapeBuilder::default();
    let leaves = shapes.material("Follaje", rng.pick(&LEAF_GREENS));
    let blobs = rng.int(2, 4);
    for index in 0..blobs {
        let angle = TAU * index as f32 / blobs as f32 + rng.range(0.0, 1.0);
        let size = rng.range(0.35, 0.7);
        let reach = if index == 0 { 0.0 } else { rng.range(0.2, 0.5) };
        let center = [angle.cos() * reach, angle.sin() * reach, size * 0.8];
        shapes.ellipsoid(leaves, center, [size, size, size * 0.8], 3, 6);
    }
    shapes.finish("Arbusto")
}

/// A palm: a curved, leaning trunk and a crown of drooping fronds.
pub fn palm(seed: u64) -> ImportedScene {
    let mut rng = Rng::new(seed);
    let mut shapes = ShapeBuilder::default();
    let bark = shapes.material("Tronco de palma", rng.pick(&[0x8a7355, 0x7d6649, 0x947b5a]));
    let fronds = shapes.material("Hojas de palma", rng.pick(&[0x4f8f34, 0x5c9a3a, 0x43802e]));
    let height = rng.range(5.0, 8.0);
    let lean = rng.range(0.4, 1.4);
    let heading = rng.range(0.0, TAU);
    let segments = 6;
    let mut previous = [0.0f32; 3];
    let mut radius = 0.22;
    for step in 1..=segments {
        let t = step as f32 / segments as f32;
        let offset = lean * t * t;
        let point = [heading.cos() * offset, heading.sin() * offset, height * t];
        shapes.frustum(bark, previous, point, radius, radius * 0.92, 6);
        previous = point;
        radius *= 0.92;
    }
    let count = rng.int(7, 10);
    for index in 0..count {
        let angle = TAU * index as f32 / count as f32 + rng.range(-0.15, 0.15);
        let length = rng.range(2.2, 3.2);
        let (dx, dy) = (angle.cos(), angle.sin());
        let (side_x, side_y) = (-dy * 0.28, dx * 0.28);
        let at = |distance: f32, drop: f32| {
            [
                previous[0] + dx * distance,
                previous[1] + dy * distance,
                previous[2] + drop,
            ]
        };
        let mid = at(length * 0.5, 0.25);
        let tip = at(length, rng.range(-1.4, -0.7));
        let widen = |p: [f32; 3], sign: f32| [p[0] + side_x * sign, p[1] + side_y * sign, p[2]];
        shapes.double_sided(fronds, &[previous, widen(mid, -1.0), mid, widen(mid, 1.0)]);
        shapes.double_sided(fronds, &[widen(mid, -1.0), tip, mid]);
        shapes.double_sided(fronds, &[mid, tip, widen(mid, 1.0)]);
    }
    shapes.finish("Palmera")
}
