//! The linear picture the tracer produces, with the guide channels the post-processing passes use.

/// Linear-light RGBA plus depth and normal per pixel.
#[derive(Debug, Clone, PartialEq)]
pub struct FrameBuffers {
    /// Width in pixels.
    pub width: usize,
    /// Height in pixels.
    pub height: usize,
    /// Interleaved linear RGBA; alpha is coverage.
    pub color: Vec<f32>,
    /// Depth along the view axis in meters (the far plane for sky).
    pub depth: Vec<f32>,
    /// Interleaved unit normals of the first surface (zero for sky).
    pub normal: Vec<f32>,
}

impl FrameBuffers {
    /// An empty (black) picture.
    pub fn new(width: usize, height: usize) -> Self {
        let count = width * height;
        Self {
            width,
            height,
            color: vec![0.0; count * 4],
            depth: vec![0.0; count],
            normal: vec![0.0; count * 3],
        }
    }

    /// Stores one pixel.
    pub fn set(&mut self, x: usize, y: usize, rgba: [f32; 4], depth: f32, normal: [f32; 3]) {
        let at = y * self.width + x;
        self.color[at * 4..at * 4 + 4].copy_from_slice(&rgba);
        self.depth[at] = depth;
        self.normal[at * 3..at * 3 + 3].copy_from_slice(&normal);
    }
}
