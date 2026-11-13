//! Collada node transforms and the up-axis conversion.

use super::geometry::numbers;
use crate::features::import_export::logic::geometry::{normalize, Mat4, V3};
use roxmltree::Node;

/// The vertical axis a Collada file declares.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Up {
    /// X is up.
    X,
    /// Y is up (the Collada default).
    Y,
    /// Z is up.
    Z,
}

impl Up {
    /// Parses the `<up_axis>` text; anything unknown is the default, Y.
    pub fn parse(text: &str) -> Up {
        match text.trim() {
            "X_UP" => Up::X,
            "Z_UP" => Up::Z,
            _ => Up::Y,
        }
    }
}

/// Scales a point from file units to meters and converts it to Z-up.
pub fn upright(p: V3, up: Up, scale: f64) -> [f32; 3] {
    let p = [p[0] * scale, p[1] * scale, p[2] * scale];
    match up {
        Up::Z => [p[0] as f32, p[1] as f32, p[2] as f32],
        Up::Y => [p[0] as f32, -p[2] as f32, p[1] as f32],
        Up::X => [p[1] as f32, p[2] as f32, p[0] as f32],
    }
}

/// Whether the transform reverses orientation (a mirror), which flips triangle winding.
pub fn mirrored(m: &Mat4) -> bool {
    let a = &m.0;
    let det = a[0][0] * (a[1][1] * a[2][2] - a[1][2] * a[2][1])
        - a[0][1] * (a[1][0] * a[2][2] - a[1][2] * a[2][0])
        + a[0][2] * (a[1][0] * a[2][1] - a[1][1] * a[2][0]);
    det < 0.0
}

fn rotation(axis: V3, degrees: f64) -> Mat4 {
    let [x, y, z] = normalize(axis);
    let (s, c) = degrees.to_radians().sin_cos();
    let t = 1.0 - c;
    Mat4([
        [t * x * x + c, t * x * y - s * z, t * x * z + s * y, 0.0],
        [t * x * y + s * z, t * y * y + c, t * y * z - s * x, 0.0],
        [t * x * z - s * y, t * y * z + s * x, t * z * z + c, 0.0],
        [0.0, 0.0, 0.0, 1.0],
    ])
}

/// The node's own transform: its `matrix`, `translate`, `rotate` and `scale` children combined in order.
pub fn local_matrix(node: Node) -> Mat4 {
    let mut result = Mat4::IDENTITY;
    for child in node.children().filter(Node::is_element) {
        let v = numbers(child);
        let next = match child.tag_name().name() {
            "matrix" if v.len() == 16 => Mat4([
                [v[0], v[1], v[2], v[3]],
                [v[4], v[5], v[6], v[7]],
                [v[8], v[9], v[10], v[11]],
                [v[12], v[13], v[14], v[15]],
            ]),
            "translate" if v.len() >= 3 => Mat4::translation([v[0], v[1], v[2]]),
            "scale" if v.len() >= 3 => Mat4([
                [v[0], 0.0, 0.0, 0.0],
                [0.0, v[1], 0.0, 0.0],
                [0.0, 0.0, v[2], 0.0],
                [0.0, 0.0, 0.0, 1.0],
            ]),
            "rotate" if v.len() >= 4 => rotation([v[0], v[1], v[2]], v[3]),
            _ => continue,
        };
        result = result.mul(&next);
    }
    result
}
