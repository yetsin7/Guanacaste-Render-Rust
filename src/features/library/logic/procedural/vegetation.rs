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

/// A bush: a dense mound of overlapping blobs in two greens, wider than it is tall, resting on the ground.
pub fn bush(seed: u64) -> ImportedScene {
    let mut rng = Rng::new(seed);
    let mut shapes = ShapeBuilder::default();
    let base = rng.pick(&LEAF_GREENS);
    let leaves = shapes.material("Follaje", base);
    let lighter = shapes.material("Follaje claro", lighten(base, 1.22));
    let blobs = rng.int(6, 9);
    for index in 0..blobs {
        let angle = TAU * index as f32 / blobs as f32 + rng.range(0.0, 0.8);
        let size = rng.range(0.32, 0.6);
        let reach = if index == 0 {
            0.0
        } else {
            rng.range(0.25, 0.6)
        };
        let lift = if index % 3 == 0 { 1.05 } else { 0.9 };
        let center = [angle.cos() * reach, angle.sin() * reach, size * lift];
        let material = if index % 2 == 0 { leaves } else { lighter };
        shapes.ellipsoid(material, center, [size, size, size * 0.85], 5, 10);
    }
    shapes.finish("Arbusto")
}

/// `rgb` (0xRRGGBB) with every channel multiplied by `factor`.
fn lighten(rgb: u32, factor: f32) -> u32 {
    let channel = |shift: u32| (((rgb >> shift) & 0xff) as f32 * factor).min(255.0) as u32;
    (channel(16) << 16) | (channel(8) << 8) | channel(0)
}

/// One arching frond: a strip of quads that rises from the crown, bows outward and droops at the tip, wide in the
/// middle and pointed at both ends.
fn frond(
    shapes: &mut ShapeBuilder,
    material: usize,
    crown: [f32; 3],
    azimuth: f32,
    length: f32,
    droop: f32,
) {
    let (dx, dy) = (azimuth.cos(), azimuth.sin());
    let side = (-dy, dx);
    let segments = 8;
    let at = |t: f32| -> ([f32; 3], f32) {
        let out = length * t;
        let rise =
            length * 0.42 * (t * std::f32::consts::PI * 0.8).sin() - droop * t * t * length * 0.55;
        let width = 0.34 * (length / 3.0) * (t * (1.0 - t) * 4.0).max(0.0).powf(0.7);
        (
            [crown[0] + dx * out, crown[1] + dy * out, crown[2] + rise],
            width,
        )
    };
    for step in 0..segments {
        let (t0, t1) = (
            step as f32 / segments as f32,
            (step + 1) as f32 / segments as f32,
        );
        let (a, wa) = at(t0);
        let (b, wb) = at(t1);
        let corner = |p: [f32; 3], w: f32, sign: f32| {
            [
                p[0] + side.0 * w * sign,
                p[1] + side.1 * w * sign,
                p[2] - w * 0.35,
            ]
        };
        shapes.double_sided(
            material,
            &[
                corner(a, wa, -1.0),
                corner(a, wa, 1.0),
                corner(b, wb, 1.0),
                corner(b, wb, -1.0),
            ],
        );
    }
}

/// A palm: a curved, leaning, ringed trunk and a lush crown of arching fronds.
pub fn palm(seed: u64) -> ImportedScene {
    let mut rng = Rng::new(seed);
    let mut shapes = ShapeBuilder::default();
    let bark = shapes.material("Tronco de palma", rng.pick(&[0x8a7355, 0x7d6649, 0x947b5a]));
    let fronds = shapes.material("Hojas de palma", rng.pick(&[0x4f8f34, 0x5c9a3a, 0x43802e]));
    let height = rng.range(5.0, 8.0);
    let lean = rng.range(0.4, 1.4);
    let heading = rng.range(0.0, TAU);
    let segments = 12;
    let mut previous = [0.0f32; 3];
    let mut radius = 0.26;
    for step in 1..=segments {
        let t = step as f32 / segments as f32;
        let offset = lean * t * t;
        let point = [heading.cos() * offset, heading.sin() * offset, height * t];
        shapes.frustum(bark, previous, point, radius, radius * 0.94, 7);
        previous = point;
        radius *= 0.94;
    }
    shapes.ellipsoid(
        bark,
        [previous[0], previous[1], previous[2] - 0.1],
        [0.3, 0.3, 0.28],
        3,
        7,
    );
    let count = rng.int(11, 14);
    for index in 0..count {
        let angle = TAU * index as f32 / count as f32 + rng.range(-0.15, 0.15);
        let length = rng.range(2.4, 3.4);
        let droop = rng.range(0.5, 1.1);
        let crown = [previous[0], previous[1], previous[2] + 0.05];
        frond(&mut shapes, fronds, crown, angle, length, droop);
    }
    shapes.finish("Palmera")
}
