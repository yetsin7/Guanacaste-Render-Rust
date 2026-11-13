//! Procedural people, vehicles and street furniture, each varied by seed.

use super::builder::ShapeBuilder;
use super::rng::Rng;
use super::vegetation;
use crate::features::import_export::models::ImportedScene;

const CLOTHES: [u32; 8] = [
    0x2c3e66, 0x8a2f2f, 0x3d6b4a, 0x6b5a3d, 0xd9d4c7, 0x1f1f24, 0x7a4f8a, 0xc98a2e,
];
const SKIN: [u32; 5] = [0xf1c9a5, 0xd9a47e, 0xb07a52, 0x8a5a3a, 0x5e3d28];
const PAINT: [u32; 7] = [
    0xb3b6bb, 0x1e2126, 0xe8e8e6, 0x8c1c1c, 0x1f3f7a, 0x3f5f45, 0x6e6a62,
];

/// A standing person in simple silhouette: legs, torso, arms and head.
pub fn person(seed: u64) -> ImportedScene {
    let mut rng = Rng::new(seed);
    let mut shapes = ShapeBuilder::default();
    let top = shapes.material("Ropa superior", rng.pick(&CLOTHES));
    let bottom = shapes.material("Ropa inferior", rng.pick(&CLOTHES));
    let skin = shapes.material("Piel", rng.pick(&SKIN));
    let height = rng.range(1.55, 1.92);
    let width = rng.range(0.85, 1.15);
    let leg = height * 0.47;
    let torso = height * 0.33;
    for side in [-1.0, 1.0] {
        let x = side * 0.09 * width;
        shapes.cuboid(bottom, [x, 0.0, leg * 0.5], [0.07 * width, 0.08, leg * 0.5]);
        let arm_x = side * 0.24 * width;
        let arm_z = leg + torso * 0.55;
        shapes.cuboid(top, [arm_x, 0.0, arm_z], [0.05, 0.06, torso * 0.45]);
    }
    shapes.cuboid(
        top,
        [0.0, 0.0, leg + torso * 0.5],
        [0.18 * width, 0.11, torso * 0.5],
    );
    let head = height * 0.075;
    shapes.ellipsoid(
        skin,
        [0.0, 0.0, leg + torso + head * 1.3],
        [head * 0.85, head * 0.95, head * 1.15],
        4,
        6,
    );
    shapes.finish("Persona")
}

/// A car: body, cabin with glass and four wheels. Its length runs along +X.
pub fn car(seed: u64) -> ImportedScene {
    let mut rng = Rng::new(seed);
    let mut shapes = ShapeBuilder::default();
    let paint = shapes.material("Pintura", rng.pick(&PAINT));
    let glass = shapes.material("Vidrio", 0x2a3540);
    let rubber = shapes.material("Llanta", 0x161616);
    let length = rng.range(3.9, 4.9);
    let width = rng.range(1.7, 1.9);
    let body = rng.range(0.55, 0.75);
    let wheel = 0.33;
    let floor = 0.22;
    shapes.cuboid(
        paint,
        [0.0, 0.0, floor + body * 0.5],
        [length * 0.5, width * 0.5, body * 0.5],
    );
    let cabin_length = length * rng.range(0.45, 0.6);
    let cabin_height = rng.range(0.45, 0.6);
    let cabin_x = rng.range(-0.25, 0.05) * length * 0.3;
    shapes.cuboid(
        glass,
        [cabin_x, 0.0, floor + body + cabin_height * 0.5],
        [cabin_length * 0.5, width * 0.45, cabin_height * 0.5],
    );
    shapes.cuboid(
        paint,
        [cabin_x, 0.0, floor + body + cabin_height + 0.03],
        [cabin_length * 0.48, width * 0.44, 0.03],
    );
    for x in [-1.0, 1.0] {
        for y in [-1.0, 1.0] {
            let center = [x * length * 0.33, y * width * 0.5, wheel];
            let from = [center[0], center[1] - 0.11, wheel];
            let to = [center[0], center[1] + 0.11, wheel];
            shapes.frustum(rubber, from, to, wheel, wheel, 10);
        }
    }
    shapes.finish("Automóvil")
}

/// A park bench: seat slats, backrest and two side frames. Its length runs along +X.
pub fn bench(seed: u64) -> ImportedScene {
    let mut rng = Rng::new(seed);
    let mut shapes = ShapeBuilder::default();
    let wood = shapes.material("Madera", rng.pick(&[0x8a6040, 0x9c7048, 0x6e4c33]));
    let metal = shapes.material("Metal", rng.pick(&[0x2a2c30, 0x3d4a3f, 0x5a5f66]));
    let length = rng.range(1.5, 2.2);
    for slat in 0..3 {
        let y = -0.18 + slat as f32 * 0.16;
        shapes.cuboid(wood, [0.0, y, 0.45], [length * 0.5, 0.065, 0.02]);
    }
    for slat in 0..2 {
        let z = 0.62 + slat as f32 * 0.16;
        shapes.cuboid(wood, [0.0, 0.25, z], [length * 0.5, 0.02, 0.055]);
    }
    for side in [-1.0, 1.0] {
        let x = side * (length * 0.5 - 0.12);
        shapes.cuboid(metal, [x, -0.15, 0.22], [0.025, 0.025, 0.22]);
        shapes.cuboid(metal, [x, 0.22, 0.4], [0.025, 0.025, 0.4]);
        shapes.cuboid(metal, [x, 0.03, 0.42], [0.025, 0.22, 0.02]);
    }
    shapes.finish("Banca")
}

/// A street light: pole, arm and lamp head.
pub fn street_light(seed: u64) -> ImportedScene {
    let mut rng = Rng::new(seed);
    let mut shapes = ShapeBuilder::default();
    let metal = shapes.material("Metal", rng.pick(&[0x3a3d42, 0x22262a, 0x5d6268]));
    let lamp = shapes.material("Luminaria", 0xfff2c4);
    let height = rng.range(4.0, 6.5);
    let reach = rng.range(0.6, 1.3);
    shapes.frustum(metal, [0.0; 3], [0.0, 0.0, 0.3], 0.16, 0.14, 8);
    shapes.frustum(metal, [0.0, 0.0, 0.3], [0.0, 0.0, height], 0.08, 0.05, 8);
    shapes.cuboid(
        metal,
        [reach * 0.5, 0.0, height - 0.05],
        [reach * 0.5, 0.03, 0.03],
    );
    shapes.cuboid(metal, [reach, 0.0, height - 0.08], [0.25, 0.12, 0.05]);
    shapes.cuboid(lamp, [reach, 0.0, height - 0.15], [0.22, 0.1, 0.02]);
    shapes.finish("Farola")
}

/// A planter: a tapered pot with soil and a bush on top.
pub fn planter(seed: u64) -> ImportedScene {
    let mut rng = Rng::new(seed);
    let plant = vegetation::bush(seed.wrapping_add(7));
    let mut shapes = ShapeBuilder::default();
    let pot = shapes.material(
        "Maceta",
        rng.pick(&[0xa8573a, 0x8f8a80, 0x3b3b3e, 0xd8d0c0]),
    );
    let soil = shapes.material("Tierra", 0x3d2b1f);
    let leaves = shapes.material("Follaje", 0x4d8a34);
    let height = rng.range(0.45, 0.8);
    let radius = rng.range(0.3, 0.5);
    shapes.frustum(pot, [0.0; 3], [0.0, 0.0, height], radius * 0.75, radius, 10);
    shapes.frustum(
        soil,
        [0.0, 0.0, height - 0.06],
        [0.0, 0.0, height - 0.04],
        radius * 0.95,
        radius * 0.95,
        10,
    );
    let shrink = radius / 0.9;
    for mesh in &plant.meshes {
        for triangle in mesh.indices.chunks_exact(3) {
            let points: Vec<[f32; 3]> = triangle
                .iter()
                .map(|&i| {
                    let p = mesh.positions[i as usize];
                    [p[0] * shrink, p[1] * shrink, p[2] * shrink + height - 0.05]
                })
                .collect();
            shapes.polygon(leaves, &points);
        }
    }
    shapes.finish("Maceta")
}

/// A two-meter fence section: two posts and horizontal rails (or pickets). Its length runs along +X.
pub fn fence(seed: u64) -> ImportedScene {
    let mut rng = Rng::new(seed);
    let mut shapes = ShapeBuilder::default();
    let wood = shapes.material("Cerca", rng.pick(&[0x8a6a4a, 0xe8e4da, 0x5a4636, 0x3a3d40]));
    let height = rng.range(0.9, 1.6);
    for x in [-1.0f32, 1.0] {
        shapes.cuboid(wood, [x, 0.0, height * 0.5], [0.05, 0.05, height * 0.5]);
    }
    if rng.unit() < 0.5 {
        for rail in 0..3 {
            let z = height * (0.25 + rail as f32 * 0.3);
            shapes.cuboid(wood, [0.0, 0.0, z], [1.0, 0.02, 0.05]);
        }
    } else {
        shapes.cuboid(wood, [0.0, 0.0, height * 0.3], [1.0, 0.02, 0.04]);
        shapes.cuboid(wood, [0.0, 0.0, height * 0.75], [1.0, 0.02, 0.04]);
        for picket in 0..11 {
            let x = -0.9 + picket as f32 * 0.18;
            shapes.cuboid(wood, [x, 0.03, height * 0.48], [0.04, 0.012, height * 0.48]);
        }
    }
    shapes.finish("Cerca")
}
