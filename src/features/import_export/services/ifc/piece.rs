//! A bag of triangles in world coordinates produced by one geometry item.

use crate::features::import_export::logic::geometry::{Mat4, V3};

/// Triangles of one geometry item with an optional style color.
#[derive(Debug, Clone, Default)]
pub struct Piece {
    /// Vertex positions.
    pub positions: Vec<V3>,
    /// Triangle indices.
    pub indices: Vec<u32>,
    /// Style color from the file, if any.
    pub color: Option<[f32; 4]>,
}

impl Piece {
    /// Adds a vertex and returns its index.
    pub fn vertex(&mut self, position: V3) -> u32 {
        self.positions.push(position);
        (self.positions.len() - 1) as u32
    }

    /// Adds a triangle over existing vertices.
    pub fn triangle(&mut self, a: u32, b: u32, c: u32) {
        self.indices.extend_from_slice(&[a, b, c]);
    }

    /// Adds a quad `a b c d` (counter-clockwise) as two triangles.
    pub fn quad(&mut self, a: u32, b: u32, c: u32, d: u32) {
        self.triangle(a, b, c);
        self.triangle(a, c, d);
    }

    /// Applies `matrix` to every vertex.
    pub fn transform(&mut self, matrix: &Mat4) {
        for position in &mut self.positions {
            *position = matrix.point(*position);
        }
    }

    /// Whether nothing was produced.
    pub fn is_empty(&self) -> bool {
        self.indices.is_empty()
    }
}
