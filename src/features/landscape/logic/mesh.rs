//! Triangle meshes of the terrain, compatible with the imported-scene meshes (meters, Z up, CCW seen from above).

use super::generate::dominant_layer;
use super::sample::normalize;
use crate::features::import_export::models::{Material, Mesh};
use crate::features::landscape::models::{SoilLayer, Terrain};
use crate::features::materials::logic::derive::{derive, SurfaceInfo};
use crate::services::localization::{tr, trf};

/// Corner indices of the two triangles of cell `(column, row)`, split along its (0,0)-(1,1) diagonal.
fn cell_triangles(terrain: &Terrain, column: usize, row: usize) -> [[u32; 3]; 2] {
    let v00 = terrain.index(column, row) as u32;
    let v10 = terrain.index(column + 1, row) as u32;
    let v01 = terrain.index(column, row + 1) as u32;
    let v11 = terrain.index(column + 1, row + 1) as u32;
    [[v00, v10, v11], [v00, v11, v01]]
}

fn cells(terrain: &Terrain) -> impl Iterator<Item = (usize, usize)> + '_ {
    (0..terrain.rows.saturating_sub(1)).flat_map(move |row| {
        (0..terrain.columns.saturating_sub(1)).map(move |column| (column, row))
    })
}

/// The whole terrain as one mesh without material: one vertex per grid point, two triangles per cell.
pub fn terrain_mesh(terrain: &Terrain) -> Mesh {
    let mut positions = Vec::with_capacity(terrain.vertex_count());
    for row in 0..terrain.rows {
        for column in 0..terrain.columns {
            positions.push(terrain.position(column, row));
        }
    }
    let mut indices = Vec::with_capacity(terrain.triangle_count() * 3);
    for (column, row) in cells(terrain) {
        for triangle in cell_triangles(terrain, column, row) {
            indices.extend_from_slice(&triangle);
        }
    }
    Mesh {
        positions,
        indices,
        material: None,
        uvs: Vec::new(),
    }
}

/// Smooth per-vertex normals of [`terrain_mesh`], from the neighboring heights.
pub fn terrain_normals(terrain: &Terrain) -> Vec<[f32; 3]> {
    let mut normals = Vec::with_capacity(terrain.vertex_count());
    for row in 0..terrain.rows {
        for column in 0..terrain.columns {
            let h = |c: usize, r: usize| terrain.heights[terrain.index(c, r)];
            let (left, right) = (
                column.saturating_sub(1),
                (column + 1).min(terrain.columns - 1),
            );
            let (down, up) = (row.saturating_sub(1), (row + 1).min(terrain.rows - 1));
            let dx =
                (h(right, row) - h(left, row)) / ((right - left).max(1) as f32 * terrain.spacing);
            let dy =
                (h(column, up) - h(column, down)) / ((up - down).max(1) as f32 * terrain.spacing);
            normals.push(normalize([-dx, -dy, 1.0]));
        }
    }
    normals
}

/// Per-vertex RGBA colors of [`terrain_mesh`]: the soil layer colors blended by their weights.
pub fn terrain_vertex_colors(terrain: &Terrain) -> Vec<[f32; 4]> {
    terrain
        .layers
        .iter()
        .map(|weights| {
            let mut color = [0.0, 0.0, 0.0, 1.0];
            for layer in SoilLayer::ALL {
                let share = f32::from(weights[layer.index()]) / 255.0;
                let base = layer.color();
                for channel in 0..3 {
                    color[channel] += base[channel] * share;
                }
            }
            color
        })
        .collect()
}

/// Material of a soil layer, named as in the interface catalog.
pub fn layer_material(layer: SoilLayer) -> Material {
    let name = trf("landscape.material.terrain", &[tr(layer.label_key())]);
    // The lawn renders as a soft textured turf; the other layers stay plain colors.
    let pbr = (layer == SoilLayer::Grass).then(|| {
        let mut turf = derive(&SurfaceInfo {
            name: "grass",
            type_name: None,
            element: None,
            class: "IFCSLAB",
            color: layer.color(),
        });
        turf.name = name.clone();
        Box::new(turf)
    });
    Material {
        name,
        color: layer.color(),
        texture: None,
        pbr,
    }
}

/// The terrain split into one mesh per soil layer, for renderers that only know one material per mesh.
/// Each triangle goes to the layer with the largest summed weight over its corners. Mesh `i` uses material `i` of
/// the returned materials; layers with no triangles are left out.
pub fn terrain_layer_meshes(terrain: &Terrain) -> (Vec<Mesh>, Vec<Material>) {
    let mut buckets: Vec<Vec<[u32; 3]>> = vec![Vec::new(); 4];
    for (column, row) in cells(terrain) {
        for triangle in cell_triangles(terrain, column, row) {
            let mut sums = [0u32; 4];
            for corner in triangle {
                for (sum, weight) in sums.iter_mut().zip(terrain.layers[corner as usize]) {
                    *sum += u32::from(weight);
                }
            }
            let layer = dominant_layer(sums.map(|s| (s / 3).min(255) as u8));
            buckets[layer.index()].push(triangle);
        }
    }
    let mut meshes = Vec::new();
    let mut materials = Vec::new();
    for (layer, triangles) in SoilLayer::ALL.into_iter().zip(buckets) {
        if triangles.is_empty() {
            continue;
        }
        let mut remap = vec![u32::MAX; terrain.vertex_count()];
        let mut mesh = Mesh {
            material: Some(materials.len()),
            ..Mesh::default()
        };
        for corner in triangles.into_iter().flatten() {
            let slot = &mut remap[corner as usize];
            if *slot == u32::MAX {
                *slot = mesh.positions.len() as u32;
                let index = corner as usize;
                mesh.positions
                    .push(terrain.position(index % terrain.columns, index / terrain.columns));
            }
            mesh.indices.push(*slot);
        }
        meshes.push(mesh);
        materials.push(layer_material(layer));
    }
    (meshes, materials)
}
