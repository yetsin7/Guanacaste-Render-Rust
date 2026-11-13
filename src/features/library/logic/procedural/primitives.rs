//! Basic primitives, one meter in size, resting on the ground: cube, cylinder, sphere and plane.

use super::builder::ShapeBuilder;
use crate::features::import_export::models::ImportedScene;

const NEUTRAL: u32 = 0xc8c8c8;

/// A one-meter cube.
pub fn cube() -> ImportedScene {
    let mut shapes = ShapeBuilder::default();
    let base = shapes.material("Base", NEUTRAL);
    shapes.cuboid(base, [0.0, 0.0, 0.5], [0.5; 3]);
    shapes.finish("Cubo")
}

/// A cylinder one meter tall and one meter wide.
pub fn cylinder() -> ImportedScene {
    let mut shapes = ShapeBuilder::default();
    let base = shapes.material("Base", NEUTRAL);
    shapes.frustum(base, [0.0; 3], [0.0, 0.0, 1.0], 0.5, 0.5, 24);
    shapes.finish("Cilindro")
}

/// A sphere one meter wide.
pub fn sphere() -> ImportedScene {
    let mut shapes = ShapeBuilder::default();
    let base = shapes.material("Base", NEUTRAL);
    shapes.ellipsoid(base, [0.0, 0.0, 0.5], [0.5; 3], 12, 24);
    shapes.finish("Esfera")
}

/// A flat one-meter square facing up.
pub fn plane() -> ImportedScene {
    let mut shapes = ShapeBuilder::default();
    let base = shapes.material("Base", NEUTRAL);
    shapes.polygon(
        base,
        &[
            [-0.5, -0.5, 0.0],
            [0.5, -0.5, 0.0],
            [0.5, 0.5, 0.0],
            [-0.5, 0.5, 0.0],
        ],
    );
    shapes.finish("Plano")
}
