//! Water surfaces: waves as a pure function of position and time, and their triangle meshes.

use super::sample::normalize;
use crate::features::import_export::models::{Material, Mesh};
use crate::features::landscape::models::WaterBody;
use crate::services::localization::tr;
use std::f32::consts::TAU;

/// Gravity used by the deep-water dispersion relation, in m/s².
const GRAVITY: f32 = 9.81;
/// Most cells per side of a water mesh.
pub const MAX_WATER_CELLS: usize = 256;

/// One sine wave train.
struct Train {
    amplitude: f32,
    k: f32,
    omega: f32,
    dx: f32,
    dy: f32,
    phase: f32,
}

impl Train {
    fn angle(&self, x: f32, y: f32, time: f32) -> f32 {
        self.k * (self.dx * x + self.dy * y) - self.omega * time + self.phase
    }
}

/// The three wave trains of a body.
fn trains(water: &WaterBody) -> [Train; 3] {
    const SHARES: [(f32, f32, f32, f32); 3] = [
        // (amplitude share, length factor, direction offset in degrees, phase)
        (0.6, 1.0, 0.0, 0.0),
        (0.28, 0.61, 25.0, 1.7),
        (0.12, 0.37, -35.0, 4.1),
    ];
    SHARES.map(|(share, length, offset, phase)| {
        let wavelength = (water.wave_length * length).max(0.05);
        let k = TAU / wavelength;
        let omega = (GRAVITY * k).sqrt();
        let angle = (water.wave_direction_deg + offset).to_radians();
        Train {
            amplitude: water.wave_amplitude.max(0.0) * share,
            k,
            omega,
            dx: angle.cos(),
            dy: angle.sin(),
            phase,
        }
    })
}

/// Height in meters of the water surface at world `(x, y)` and `time` seconds. Pure: the same inputs always give
/// the same height, so films and renders can sample any instant. The crest never exceeds `height + wave_amplitude`.
pub fn wave_height(water: &WaterBody, x: f32, y: f32, time: f32) -> f32 {
    water.height
        + trains(water)
            .iter()
            .map(|train| train.amplitude * train.angle(x, y, time).sin())
            .sum::<f32>()
}

/// Unit normal of the water surface at world `(x, y)` and `time`, from the analytic slope of the waves.
pub fn wave_normal(water: &WaterBody, x: f32, y: f32, time: f32) -> [f32; 3] {
    let (mut sx, mut sy) = (0.0, 0.0);
    for train in trains(water) {
        let slope = train.amplitude * train.k * train.angle(x, y, time).cos();
        sx += slope * train.dx;
        sy += slope * train.dy;
    }
    normalize([-sx, -sy, 1.0])
}

/// Cells per side used by [`water_mesh`]: about four per shortest wave, capped at [`MAX_WATER_CELLS`].
pub fn water_cells(water: &WaterBody) -> [usize; 2] {
    let step = if water.wave_amplitude > 0.0 {
        (water.wave_length * 0.37 / 4.0).max(0.05)
    } else {
        f32::INFINITY
    };
    water
        .size
        .map(|side| ((side / step).ceil() as usize).clamp(1, MAX_WATER_CELLS))
}

/// The water surface at `time` as a grid mesh of [`water_cells`] cells, centered on the body. No material.
pub fn water_mesh(water: &WaterBody, time: f32) -> Mesh {
    water_mesh_with(water, time, water_cells(water))
}

/// Like [`water_mesh`] with an explicit number of cells along X and Y (each clamped to `1..=MAX_WATER_CELLS`).
pub fn water_mesh_with(water: &WaterBody, time: f32, cells: [usize; 2]) -> Mesh {
    let [nx, ny] = cells.map(|c| c.clamp(1, MAX_WATER_CELLS));
    let start = [
        water.center[0] - water.size[0] / 2.0,
        water.center[1] - water.size[1] / 2.0,
    ];
    let mut positions = Vec::with_capacity((nx + 1) * (ny + 1));
    for j in 0..=ny {
        for i in 0..=nx {
            let x = start[0] + water.size[0] * i as f32 / nx as f32;
            let y = start[1] + water.size[1] * j as f32 / ny as f32;
            positions.push([x, y, wave_height(water, x, y, time)]);
        }
    }
    let mut indices = Vec::with_capacity(nx * ny * 6);
    let at = |i: usize, j: usize| (j * (nx + 1) + i) as u32;
    for j in 0..ny {
        for i in 0..nx {
            let (v00, v10, v01, v11) = (at(i, j), at(i + 1, j), at(i, j + 1), at(i + 1, j + 1));
            indices.extend_from_slice(&[v00, v10, v11, v00, v11, v01]);
        }
    }
    Mesh {
        positions,
        indices,
        material: None,
        uvs: Vec::new(),
    }
}

/// Material of a water body: its color with the opacity as alpha.
pub fn water_material(water: &WaterBody) -> Material {
    let [r, g, b] = water.color;
    Material {
        name: tr(water.kind.label_key()).to_owned(),
        color: [r, g, b, water.opacity.clamp(0.0, 1.0)],
        texture: None,
        pbr: None,
    }
}
