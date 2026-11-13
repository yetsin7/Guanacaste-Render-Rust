//! Reading the terrain at any world position: height, normal and slope.

use crate::features::landscape::models::Terrain;

/// Grid coordinates (fractional column and row) of world `(x, y)`, clamped to the terrain.
pub fn grid_coords(terrain: &Terrain, x: f32, y: f32) -> (f32, f32) {
    let column =
        ((x - terrain.origin[0]) / terrain.spacing).clamp(0.0, (terrain.columns - 1) as f32);
    let row = ((y - terrain.origin[1]) / terrain.spacing).clamp(0.0, (terrain.rows - 1) as f32);
    (column, row)
}

/// Height in meters of the terrain surface at world `(x, y)`, matching the triangles of `terrain_mesh`.
/// Positions outside the terrain take the height of the nearest edge.
pub fn height_at(terrain: &Terrain, x: f32, y: f32) -> f32 {
    if terrain.heights.len() < terrain.vertex_count() || terrain.columns < 2 || terrain.rows < 2 {
        return 0.0;
    }
    let (column, row) = grid_coords(terrain, x, y);
    let c0 = (column.floor() as usize).min(terrain.columns - 2);
    let r0 = (row.floor() as usize).min(terrain.rows - 2);
    let (u, v) = (column - c0 as f32, row - r0 as f32);
    let h = |c: usize, r: usize| terrain.heights[terrain.index(c, r)];
    let (h00, h10, h01, h11) = (h(c0, r0), h(c0 + 1, r0), h(c0, r0 + 1), h(c0 + 1, r0 + 1));
    // Each cell is split along the (0,0)-(1,1) diagonal, as in the mesh.
    if u >= v {
        h00 + (h10 - h00) * u + (h11 - h10) * v
    } else {
        h00 + (h11 - h01) * u + (h01 - h00) * v
    }
}

/// Unit surface normal (Z up) at world `(x, y)`, from central differences.
pub fn normal_at(terrain: &Terrain, x: f32, y: f32) -> [f32; 3] {
    let step = terrain.spacing;
    let dx = (height_at(terrain, x + step, y) - height_at(terrain, x - step, y)) / (2.0 * step);
    let dy = (height_at(terrain, x, y + step) - height_at(terrain, x, y - step)) / (2.0 * step);
    normalize([-dx, -dy, 1.0])
}

/// Slope in degrees (0 = flat, 90 = vertical) at world `(x, y)`.
pub fn slope_deg_at(terrain: &Terrain, x: f32, y: f32) -> f32 {
    normal_at(terrain, x, y)[2]
        .clamp(-1.0, 1.0)
        .acos()
        .to_degrees()
}

/// `vector` scaled to unit length (or straight up when it is zero).
pub fn normalize(vector: [f32; 3]) -> [f32; 3] {
    let length = (vector[0] * vector[0] + vector[1] * vector[1] + vector[2] * vector[2]).sqrt();
    if length > f32::EPSILON {
        [vector[0] / length, vector[1] / length, vector[2] / length]
    } else {
        [0.0, 0.0, 1.0]
    }
}
