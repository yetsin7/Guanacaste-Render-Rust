//! The surface a ray hit: flat material values, or the PBR material sampled with the texture coordinates
//! (albedo, roughness, metalness, occlusion, emission, transparency and the normal or bump map).

use super::bvh::{Hit, Triangle, NO_UV};
use super::geometry::RenderScene;
use super::types::NO_MAPS;
use super::vec3::Vec3;
use crate::features::materials::surface;

/// Everything the shader needs to know about the point that was hit.
#[derive(Debug, Clone, Copy)]
pub struct Surf {
    /// Linear base color.
    pub albedo: Vec3,
    /// Opacity, 0..=1.
    pub alpha: f32,
    /// Unit shading normal facing the viewer.
    pub normal: Vec3,
    /// Roughness, 0..=1.
    pub roughness: f32,
    /// Metalness, 0..=1.
    pub metallic: f32,
    /// Ambient occlusion, 1 = open.
    pub occlusion: f32,
    /// Emitted light in render units (without the lighting domain's nits).
    pub emission: Vec3,
    /// Index of refraction.
    pub ior: f32,
}

/// The tangent and bitangent of a triangle from its texture coordinates, or `None` when they are degenerate.
fn tangent_frame(tri: &Triangle, uv: &[[f32; 2]; 3]) -> Option<(Vec3, Vec3)> {
    let (du1, dv1) = (uv[1][0] - uv[0][0], uv[1][1] - uv[0][1]);
    let (du2, dv2) = (uv[2][0] - uv[0][0], uv[2][1] - uv[0][1]);
    let det = du1 * dv2 - du2 * dv1;
    if det.abs() < 1e-12 {
        return None;
    }
    let t = (tri.e1 * dv2 - tri.e2 * dv1) / det;
    let b = (tri.e2 * du1 - tri.e1 * du2) / det;
    Some((t, b))
}

/// Evaluates the material of `tri` at `hit`. `normal` is the geometric normal facing the viewer; `pixel_world` is
/// the size in meters of one pixel at the hit, which picks the texture detail level.
pub fn evaluate(
    scene: &RenderScene,
    tri: &Triangle,
    hit: &Hit,
    normal: Vec3,
    pixel_world: f32,
) -> Surf {
    let material = &scene.materials[tri.material as usize];
    let mut out = Surf {
        albedo: Vec3::from_array(material.color),
        alpha: 1.0 - material.transparency,
        normal,
        roughness: material.roughness,
        metallic: material.metallic,
        occlusion: 1.0,
        emission: Vec3::from_array(material.emission),
        ior: material.ior,
    };
    if material.maps == NO_MAPS || tri.uv == NO_UV {
        return out;
    }
    let start = tri.uv as usize;
    let Some(corners) = scene.uvs.get(start..start + 3) else {
        return out;
    };
    let uv = [corners[0], corners[1], corners[2]];
    let w0 = 1.0 - hit.u - hit.v;
    let point = [
        w0 * uv[0][0] + hit.u * uv[1][0] + hit.v * uv[2][0],
        w0 * uv[0][1] + hit.u * uv[1][1] + hit.v * uv[2][1],
    ];
    let frame = tangent_frame(tri, &uv);
    let area_world = tri.e1.cross(tri.e2).length();
    let area_uv = ((uv[1][0] - uv[0][0]) * (uv[2][1] - uv[0][1])
        - (uv[2][0] - uv[0][0]) * (uv[1][1] - uv[0][1]))
        .abs();
    let per_meter = (area_uv / area_world.max(1e-12)).sqrt();
    let footprint = pixel_world * per_meter;
    // Mesh coordinates have V up; the texture lookup counts rows downward.
    let s = surface(
        &scene.pbr[tri.material as usize],
        &scene.maps[material.maps as usize],
        [point[0], 1.0 - point[1]],
        footprint,
    );
    out.albedo = Vec3::from_array(s.albedo);
    out.alpha = s.alpha;
    out.roughness = s.roughness;
    out.metallic = s.metallic;
    out.occlusion = s.ao;
    out.emission = Vec3::from_array(s.emission);
    if s.normal != [0.0, 0.0, 1.0] {
        if let Some((t, bitangent)) = frame {
            let t = (t - normal * normal.dot(t)).normalized();
            let mut b = normal.cross(t);
            if b.dot(bitangent) < 0.0 {
                b = -b;
            }
            let n = (t * s.normal[0] + b * s.normal[1] + normal * s.normal[2]).normalized();
            if n.is_finite() && n.dot(normal) > 0.05 {
                out.normal = n;
            }
        }
    }
    out
}
