//! Texture coordinates for meshes that have none (IFC, OBJ without UVs): each triangle is projected onto its own
//! plane in meters, so a texture keeps one real-world size on every wall, floor and roof pitch and never stretches.

use super::vec3::Vec3;

/// UV corners (meters) of the triangle `a`, `b`, `c`. U runs horizontally along the surface and V up the slope
/// (on level surfaces U follows X and V follows Y), so tile rows stay horizontal on walls and run down the pitch
/// on roofs. Triangles of one plane get continuous coordinates.
pub fn planar(a: Vec3, b: Vec3, c: Vec3) -> [[f32; 2]; 3] {
    let normal = (b - a).cross(c - a).normalized();
    let up = Vec3::new(0.0, 0.0, 1.0);
    let side = normal.cross(up);
    let level = side.length() < 0.05;
    let u = if level {
        Vec3::new(1.0, 0.0, 0.0)
    } else {
        side.normalized()
    };
    let v = if level {
        normal.cross(u)
    } else {
        u.cross(normal)
    }
    .normalized();
    [a, b, c].map(|p| [p.dot(u), p.dot(v)])
}

#[cfg(test)]
mod tests {
    use super::*;

    fn close(a: [f32; 2], b: [f32; 2]) {
        assert!(
            (a[0] - b[0]).abs() < 1e-4 && (a[1] - b[1]).abs() < 1e-4,
            "{a:?} {b:?}"
        );
    }

    #[test]
    fn a_wall_maps_horizontally_and_upward() {
        let uv = planar(
            Vec3::new(0.0, 0.0, 0.0),
            Vec3::new(2.0, 0.0, 0.0),
            Vec3::new(0.0, 0.0, 3.0),
        );
        // One meter along the wall is one UV unit; one meter up is one UV unit, whichever way the face points.
        let du = ((uv[1][0] - uv[0][0]).powi(2) + (uv[1][1] - uv[0][1]).powi(2)).sqrt();
        let dv = ((uv[2][0] - uv[0][0]).powi(2) + (uv[2][1] - uv[0][1]).powi(2)).sqrt();
        assert!((du - 2.0).abs() < 1e-4 && (dv - 3.0).abs() < 1e-4);
        assert!(
            uv[1][1].abs() < 1e-4,
            "the wall's horizontal edge keeps V constant"
        );
        assert!(uv[2][1] > uv[0][1], "V grows upward");
    }

    #[test]
    fn coplanar_triangles_share_continuous_coordinates() {
        let (p, q, r, s) = (
            Vec3::new(0.0, 1.0, 0.0),
            Vec3::new(4.0, 1.0, 0.0),
            Vec3::new(4.0, 1.0, 2.0),
            Vec3::new(0.0, 1.0, 2.0),
        );
        let first = planar(p, q, r);
        let second = planar(p, r, s);
        close(first[0], second[0]);
        close(first[2], second[1]);
    }

    #[test]
    fn a_floor_follows_x_and_y() {
        let uv = planar(
            Vec3::new(0.0, 0.0, 2.0),
            Vec3::new(1.0, 0.0, 2.0),
            Vec3::new(0.0, 1.0, 2.0),
        );
        close([uv[1][0] - uv[0][0], uv[1][1] - uv[0][1]], [1.0, 0.0]);
        close([uv[2][0] - uv[0][0], uv[2][1] - uv[0][1]], [0.0, 1.0]);
    }

    #[test]
    fn a_roof_pitch_runs_along_the_slope() {
        // Rises 1 m over 2 m in Y: the slope length is sqrt(5).
        let uv = planar(
            Vec3::new(0.0, 0.0, 0.0),
            Vec3::new(1.0, 0.0, 0.0),
            Vec3::new(0.0, 2.0, 1.0),
        );
        let dv = ((uv[2][0] - uv[0][0]).powi(2) + (uv[2][1] - uv[0][1]).powi(2)).sqrt();
        assert!((dv - 5f32.sqrt()).abs() < 1e-4);
    }
}
