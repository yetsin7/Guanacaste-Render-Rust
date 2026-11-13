//! The ground grid and the coordinate axes, generated as line vertices (same 16-byte layout as the scene).

use crate::features::import_export::models::Bounds;

/// Grid size when there is no model, in meters.
const EMPTY_EXTENT: f32 = 20.0;
/// Most lines the grid may have per direction.
const MAX_LINES: usize = 401;

const MINOR: [u8; 4] = [74, 82, 98, 110];
const MAJOR: [u8; 4] = [112, 124, 146, 190];
const AXIS_X: [u8; 4] = [235, 72, 72, 255];
const AXIS_Y: [u8; 4] = [92, 205, 96, 255];
const AXIS_Z: [u8; 4] = [84, 136, 245, 255];

/// Line vertices of the grid and of the axes.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Overlay {
    /// Vertex bytes: the grid first, then the axes.
    pub bytes: Vec<u8>,
    /// Vertices that belong to the grid.
    pub grid_vertices: u32,
    /// Vertices that belong to the axes (follow the grid).
    pub axes_vertices: u32,
    /// Distance between minor grid lines in meters.
    pub step: f32,
}

/// Rounds `value` up to 1, 2 or 5 times a power of ten.
pub fn nice_step(value: f32) -> f32 {
    let value = value.max(1e-3);
    let magnitude = 10f32.powf(value.log10().floor());
    let unit = value / magnitude;
    let nice = if unit <= 1.0 {
        1.0
    } else if unit <= 2.0 {
        2.0
    } else if unit <= 5.0 {
        5.0
    } else {
        10.0
    };
    nice * magnitude
}

fn push(bytes: &mut Vec<u8>, point: [f32; 3], color: [u8; 4]) {
    for value in point {
        bytes.extend_from_slice(&value.to_ne_bytes());
    }
    bytes.extend_from_slice(&color);
}

/// Builds the overlay for a scene with the given bounds (a default neighbourhood when `None`). The grid lies on
/// `z = 0` when the model crosses it, otherwise on the model's lowest point.
pub fn overlay(bounds: Option<&Bounds>) -> Overlay {
    let (center, extent, ground) = match bounds {
        Some(b) => {
            let size = [b.max[0] - b.min[0], b.max[1] - b.min[1]];
            let ground = if b.min[2] <= 0.0 && b.max[2] >= 0.0 {
                0.0
            } else {
                b.min[2]
            };
            (
                [(b.min[0] + b.max[0]) * 0.5, (b.min[1] + b.max[1]) * 0.5],
                size[0].max(size[1]).max(1.0),
                ground,
            )
        }
        None => ([0.0, 0.0], EMPTY_EXTENT, 0.0),
    };
    let step = nice_step(extent / 20.0);
    let major = step * 10.0;
    let span = (extent * 1.5).max(step * 20.0);
    let half = ((span * 0.5 / major).ceil() * major).min(step * (MAX_LINES / 2) as f32);
    let origin = [
        (center[0] / major).round() * major,
        (center[1] / major).round() * major,
    ];
    let lines = (half / step).round() as i32;
    let mut bytes = Vec::new();
    for i in -lines..=lines {
        let offset = i as f32 * step;
        let color = if i % 10 == 0 { MAJOR } else { MINOR };
        let (x, y) = (origin[0] + offset, origin[1] + offset);
        push(&mut bytes, [x, origin[1] - half, ground], color);
        push(&mut bytes, [x, origin[1] + half, ground], color);
        push(&mut bytes, [origin[0] - half, y, ground], color);
        push(&mut bytes, [origin[0] + half, y, ground], color);
    }
    let grid_vertices = (bytes.len() / 16) as u32;
    let length = (step * 5.0).max(2.0);
    for (end, color) in [
        ([length, 0.0, 0.0], AXIS_X),
        ([0.0, length, 0.0], AXIS_Y),
        ([0.0, 0.0, length], AXIS_Z),
    ] {
        push(&mut bytes, [0.0, 0.0, ground], color);
        push(&mut bytes, [end[0], end[1], ground + end[2]], color);
    }
    let axes_vertices = (bytes.len() / 16) as u32 - grid_vertices;
    Overlay {
        bytes,
        grid_vertices,
        axes_vertices,
        step,
    }
}
