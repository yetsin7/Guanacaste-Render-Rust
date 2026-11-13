//! Tiny scenes for the renderer tests: a cube, a plane, a few quads.

use crate::features::import_export::models::{
    ImportedScene, Material, Mesh, SourceFormat, SourceInfo,
};

/// A scene with these meshes and materials.
pub fn scene(meshes: Vec<Mesh>, materials: Vec<Material>) -> ImportedScene {
    ImportedScene {
        source: SourceInfo {
            format: SourceFormat::Obj,
            version: String::new(),
            application: String::new(),
            length_unit: "m".into(),
            unit_scale: 1.0,
            file_size: 0,
        },
        nodes: Vec::new(),
        meshes,
        materials,
        levels: Vec::new(),
        origin_shift: [0.0; 3],
    }
}

/// A scene without geometry.
pub fn empty() -> ImportedScene {
    scene(Vec::new(), Vec::new())
}

/// An opaque material of the given sRGB color.
pub fn paint(r: f32, g: f32, b: f32) -> Material {
    Material {
        name: "paint".into(),
        color: [r, g, b, 1.0],
        texture: None,
    }
}

/// A rectangle in the plane `z = z0`, facing up.
pub fn horizontal_quad(x: [f32; 2], y: [f32; 2], z0: f32, material: usize) -> Mesh {
    Mesh {
        positions: vec![
            [x[0], y[0], z0],
            [x[1], y[0], z0],
            [x[1], y[1], z0],
            [x[0], y[1], z0],
        ],
        indices: vec![0, 1, 2, 0, 2, 3],
        material: Some(material),
        uvs: Vec::new(),
    }
}

/// A rectangle in the plane `y = y0` (a wall facing the -Y side).
pub fn wall_quad(x: [f32; 2], z: [f32; 2], y0: f32, material: usize) -> Mesh {
    Mesh {
        positions: vec![
            [x[0], y0, z[0]],
            [x[1], y0, z[0]],
            [x[1], y0, z[1]],
            [x[0], y0, z[1]],
        ],
        indices: vec![0, 1, 2, 0, 2, 3],
        material: Some(material),
        uvs: Vec::new(),
    }
}

/// An axis-aligned cube from `min` to `max`.
pub fn cube(min: [f32; 3], max: [f32; 3], material: usize) -> Mesh {
    let mut positions = Vec::new();
    for i in 0..8 {
        positions.push([
            if i & 1 == 0 { min[0] } else { max[0] },
            if i & 2 == 0 { min[1] } else { max[1] },
            if i & 4 == 0 { min[2] } else { max[2] },
        ]);
    }
    let faces: [[u32; 4]; 6] = [
        [0, 1, 3, 2],
        [4, 5, 7, 6],
        [0, 1, 5, 4],
        [2, 3, 7, 6],
        [0, 2, 6, 4],
        [1, 3, 7, 5],
    ];
    let mut indices = Vec::new();
    for f in faces {
        indices.extend([f[0], f[1], f[2], f[0], f[2], f[3]]);
    }
    Mesh {
        positions,
        indices,
        material: Some(material),
        uvs: Vec::new(),
    }
}

/// A 2 m cube standing on a 40 m ground plane, both light grey.
pub fn cube_on_ground() -> ImportedScene {
    scene(
        vec![
            horizontal_quad([-20.0, 20.0], [-20.0, 20.0], 0.0, 0),
            cube([-1.0, -1.0, 0.0], [1.0, 1.0, 2.0], 0),
        ],
        vec![paint(0.8, 0.8, 0.8)],
    )
}
