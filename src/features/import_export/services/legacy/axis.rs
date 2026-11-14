//! A straight plan axis (from a start to an end) with helpers to place boxes along and across it.

use super::part::{Key, Part};

/// A line in plan with its direction and left normal.
pub struct Axis {
    pub start: [f64; 2],
    pub length: f64,
    pub dir: [f64; 2],
}

impl Axis {
    /// The axis from `start` to `end`; `None` when they coincide.
    pub fn new(start: [f64; 2], end: [f64; 2]) -> Option<Self> {
        let length = (end[0] - start[0]).hypot(end[1] - start[1]);
        (length > 1e-9 && length.is_finite()).then(|| Self {
            start,
            length,
            dir: [(end[0] - start[0]) / length, (end[1] - start[1]) / length],
        })
    }

    /// Plan point at distance `along` and `side` meters to the left.
    pub fn at(&self, along: f64, side: f64) -> [f64; 2] {
        [
            self.start[0] + self.dir[0] * along - self.dir[1] * side,
            self.start[1] + self.dir[1] * along + self.dir[0] * side,
        ]
    }

    /// A box between distances `from..to` along the axis, `a..b` across it and heights `z0..z1`.
    #[allow(clippy::too_many_arguments)]
    pub fn block(
        &self,
        part: &mut Part,
        key: impl Into<Key>,
        from: f64,
        to: f64,
        a: f64,
        b: f64,
        z0: f64,
        z1: f64,
    ) {
        if (to - from).abs() < 1e-6 || (b - a).abs() < 1e-6 || (z1 - z0).abs() < 1e-6 {
            return;
        }
        let outline = [
            self.at(from, a),
            self.at(to, a),
            self.at(to, b),
            self.at(from, b),
        ];
        let key = key.into();
        part.convex_prism(&outline, z0, z1, key, key);
    }

    /// A box whose floor and ceiling rise linearly: `z0`/`z1` at the start, `e0`/`e1` at the end.
    #[allow(clippy::too_many_arguments)]
    pub fn wedge(
        &self,
        part: &mut Part,
        key: impl Into<Key>,
        width: f64,
        z: [f64; 2],
        end_z: [f64; 2],
    ) {
        let outline = [
            self.at(0.0, width / 2.0),
            self.at(0.0, -width / 2.0),
            self.at(self.length, -width / 2.0),
            self.at(self.length, width / 2.0),
        ];
        let key = key.into();
        let (b, t) = (
            [z[0] as f32, z[0] as f32, end_z[0] as f32, end_z[0] as f32],
            [z[1] as f32, z[1] as f32, end_z[1] as f32, end_z[1] as f32],
        );
        part.convex_prism_h(&outline, &b, &t, key, key);
    }
}

/// A regular polygon (a round section) of `sides` sides around `center`.
pub fn round(center: [f64; 2], width: f64, depth: f64, sides: u32) -> Vec<[f64; 2]> {
    (0..sides)
        .map(|i| {
            let a = std::f64::consts::TAU * f64::from(i) / f64::from(sides);
            [
                center[0] + width / 2.0 * a.cos(),
                center[1] + depth / 2.0 * a.sin(),
            ]
        })
        .collect()
}

/// A plan position, base elevation and turn that places geometry authored in local coordinates (X to the right, Y
/// deep, Z up, front toward -Y).
#[derive(Debug, Clone, Copy)]
pub struct Placement {
    /// Plan position of the local origin.
    pub at: [f64; 2],
    /// Elevation of the local floor.
    pub z: f64,
    /// Counter-clockwise turn in radians.
    pub rotation: f64,
}

impl Placement {
    /// The plan position of a local plan point.
    pub fn plan(&self, x: f64, y: f64) -> [f64; 2] {
        let (sin, cos) = self.rotation.sin_cos();
        [
            self.at[0] + x * cos - y * sin,
            self.at[1] + x * sin + y * cos,
        ]
    }

    /// A box in local coordinates (`x0..x1`, `y0..y1`, `z0..z1`), turned and lifted.
    pub fn cuboid(
        &self,
        part: &mut Part,
        key: impl Into<Key>,
        x: [f64; 2],
        y: [f64; 2],
        z: [f64; 2],
    ) {
        let key = key.into();
        let outline = [
            self.plan(x[0], y[0]),
            self.plan(x[1], y[0]),
            self.plan(x[1], y[1]),
            self.plan(x[0], y[1]),
        ];
        part.convex_prism(&outline, self.z + z[0], self.z + z[1], key, key);
    }
}
