//! Ray casting of the preview shapes: an orthographic camera looking down -Z at a sphere or a cube.

use super::vec3::{cross, dot, normalize, Rotation, V3};
use super::PreviewShape;

/// Radius of the sphere in view units (the view spans -1..1).
const SPHERE_RADIUS: f32 = 0.9;
/// Half the edge of the cube.
const CUBE_HALF: f32 = 0.55;

/// Where a pixel's ray meets the shape, in view space.
#[derive(Debug, Clone, Copy)]
pub struct Hit {
    /// Surface normal.
    pub normal: V3,
    /// Tangent along +U.
    pub tangent: V3,
    /// Bitangent along "up in the texture" (-V).
    pub bitangent: V3,
    /// Texture coordinates; V grows downward.
    pub uv: [f32; 2],
    /// Fraction of the pixel covered by the shape (antialiased silhouette of the sphere).
    pub coverage: f32,
}

/// UV size of one pixel, for mip selection.
pub fn footprint(shape: PreviewShape, size: usize) -> f32 {
    let pixel = 2.0 / size.max(1) as f32;
    match shape {
        PreviewShape::Sphere => pixel / (std::f32::consts::PI * SPHERE_RADIUS),
        PreviewShape::Cube => pixel / (2.0 * CUBE_HALF),
    }
}

/// Casts the ray through view-space point `(x, y)`; `pixel` is the size of a pixel in view units.
pub fn cast(shape: PreviewShape, rotation: &Rotation, x: f32, y: f32, pixel: f32) -> Option<Hit> {
    match shape {
        PreviewShape::Sphere => sphere(rotation, x, y, pixel),
        PreviewShape::Cube => cube(rotation, x, y),
    }
}

fn sphere(rotation: &Rotation, x: f32, y: f32, pixel: f32) -> Option<Hit> {
    let distance = (x * x + y * y).sqrt();
    let coverage = ((SPHERE_RADIUS - distance) / pixel + 0.5).clamp(0.0, 1.0);
    if coverage <= 0.0 {
        return None;
    }
    let z = (SPHERE_RADIUS * SPHERE_RADIUS - distance * distance)
        .max(0.002)
        .sqrt();
    let normal = normalize([x, y, z]);
    let object = rotation.inverse_apply(normal);
    let u = object[0].atan2(object[2]) / std::f32::consts::TAU + 0.5;
    let v = 0.5 - object[1].clamp(-1.0, 1.0).asin() / std::f32::consts::PI;
    let tangent_object = normalize([object[2], 0.0, -object[0]]);
    let tangent_object = if dot(tangent_object, tangent_object) > 0.5 {
        tangent_object
    } else {
        [1.0, 0.0, 0.0]
    };
    let bitangent_object = cross(object, tangent_object);
    Some(Hit {
        normal,
        tangent: rotation.apply(tangent_object),
        bitangent: rotation.apply(bitangent_object),
        uv: [u, v],
        coverage,
    })
}

fn cube(rotation: &Rotation, x: f32, y: f32) -> Option<Hit> {
    let origin = rotation.inverse_apply([x, y, 2.0]);
    let direction = rotation.inverse_apply([0.0, 0.0, -1.0]);
    let (mut near, mut far) = (f32::MIN, f32::MAX);
    let mut axis = 0;
    let mut sign = 1.0;
    for a in 0..3 {
        if direction[a].abs() < 1e-8 {
            if origin[a].abs() > CUBE_HALF {
                return None;
            }
            continue;
        }
        let t1 = (-CUBE_HALF - origin[a]) / direction[a];
        let t2 = (CUBE_HALF - origin[a]) / direction[a];
        let (enter, leave, face) = if t1 < t2 {
            (t1, t2, -1.0)
        } else {
            (t2, t1, 1.0)
        };
        if enter > near {
            near = enter;
            axis = a;
            sign = face;
        }
        far = far.min(leave);
    }
    if near > far || far < 0.0 {
        return None;
    }
    let point = [
        origin[0] + direction[0] * near,
        origin[1] + direction[1] * near,
        origin[2] + direction[2] * near,
    ];
    let (a, b) = match axis {
        0 => (2, 1),
        1 => (0, 2),
        _ => (0, 1),
    };
    let mut normal_object = [0.0; 3];
    normal_object[axis] = sign;
    let mut tangent_object = [0.0; 3];
    tangent_object[a] = 1.0;
    let mut bitangent_object = [0.0; 3];
    bitangent_object[b] = 1.0;
    let u = point[a] / CUBE_HALF * 0.5 + 0.5;
    let v = 0.5 - point[b] / CUBE_HALF * 0.5;
    Some(Hit {
        normal: rotation.apply(normal_object),
        tangent: rotation.apply(tangent_object),
        bitangent: rotation.apply(bitangent_object),
        uv: [u, v],
        coverage: 1.0,
    })
}
