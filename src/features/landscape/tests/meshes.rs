//! Terrain meshes and scene compatibility.

use crate::features::landscape::logic::actions::landscape_meshes;
use crate::features::landscape::logic::mesh::{
    terrain_layer_meshes, terrain_mesh, terrain_normals, terrain_vertex_colors,
};
use crate::features::landscape::models::{Landscape, SoilLayer, Terrain, WaterBody};

#[test]
fn mesh_counts_match_the_grid_and_indices_are_valid() {
    let mut terrain = Terrain::flat(9, 6, 2.0);
    terrain.heights[10] = 3.0;
    let mesh = terrain_mesh(&terrain);
    assert_eq!(mesh.positions.len(), 54);
    assert_eq!(mesh.indices.len(), terrain.triangle_count() * 3);
    assert!(mesh
        .indices
        .iter()
        .all(|i| (*i as usize) < mesh.positions.len()));
    assert_eq!(mesh.positions[10][2], 3.0);
}

#[test]
fn triangles_face_up() {
    let mesh = terrain_mesh(&Terrain::flat(4, 4, 1.0));
    for triangle in mesh.indices.chunks(3) {
        let [a, b, c] = [0, 1, 2].map(|i| mesh.positions[triangle[i] as usize]);
        let cross_z = (b[0] - a[0]) * (c[1] - a[1]) - (b[1] - a[1]) * (c[0] - a[0]);
        assert!(cross_z > 0.0);
    }
}

#[test]
fn normals_and_colors_cover_every_vertex() {
    let terrain = Terrain::flat(5, 5, 1.0);
    assert!(terrain_normals(&terrain)
        .iter()
        .all(|n| (n[2] - 1.0).abs() < 1e-6));
    let colors = terrain_vertex_colors(&terrain);
    assert_eq!(colors.len(), 25);
    assert_eq!(&colors[0][..3], &SoilLayer::Grass.color()[..3]);
}

#[test]
fn layer_meshes_split_by_soil_and_keep_all_triangles() {
    let mut terrain = Terrain::flat(9, 9, 1.0);
    for row in 0..9 {
        for column in 5..9 {
            let index = terrain.index(column, row);
            terrain.layers[index] = [0, 0, 255, 0];
        }
    }
    let (meshes, materials) = terrain_layer_meshes(&terrain);
    assert_eq!(meshes.len(), 2);
    assert_eq!(materials.len(), 2);
    let total: usize = meshes.iter().map(|m| m.indices.len() / 3).sum();
    assert_eq!(total, terrain.triangle_count());
    for (i, mesh) in meshes.iter().enumerate() {
        assert_eq!(mesh.material, Some(i));
        assert!(mesh
            .indices
            .iter()
            .all(|x| (*x as usize) < mesh.positions.len()));
    }
}

#[test]
fn whole_landscape_adds_water_meshes_with_translucent_materials() {
    let landscape = Landscape {
        waters: vec![WaterBody::plane([0.0, 0.0], 1.0)],
        ..Landscape::default()
    };
    let (meshes, materials) = landscape_meshes(&landscape, 0.0);
    assert_eq!(meshes.len(), materials.len());
    assert!(materials.last().is_some_and(|m| m.color[3] < 1.0));
}
