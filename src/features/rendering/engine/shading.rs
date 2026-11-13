//! Light transport: hard sun shadows, sky light, diffuse and specular surfaces, bounces and transparency.

use super::geometry::RenderScene;
use super::sampling::{basis, cosine_hemisphere, Rng};
use super::types::{RenderMaterial, RenderRequest};
use super::vec3::Vec3;

/// Offset that keeps secondary rays from hitting the surface they start on.
const BIAS: f32 = 1e-3;
/// Samples brighter than this are scaled down to tame isolated bright dots.
const FIREFLY_LIMIT: f32 = 12.0;
/// Cosine of the sun's half angle: the visible disc.
const SUN_DISC_COS: f32 = 0.999_96;

/// What one camera ray brought back.
#[derive(Debug, Clone, Copy)]
pub struct Sample {
    /// Linear light.
    pub color: Vec3,
    /// False when the ray left the scene without touching anything.
    pub covered: bool,
}

/// Scene, materials and lights prepared for tracing.
pub struct Shader<'a> {
    scene: &'a RenderScene,
    materials: &'a [RenderMaterial],
    bounces: u32,
    sun_dir: Vec3,
    sun_light: Vec3,
    sun_color: Vec3,
    zenith: Vec3,
    horizon: Vec3,
}

impl<'a> Shader<'a> {
    /// Prepares the lights of `request` for `scene`.
    pub fn new(scene: &'a RenderScene, request: &'a RenderRequest) -> Self {
        let sky = &request.sun_sky;
        let sun_color = Vec3::from_array(sky.sun_color);
        Self {
            scene,
            materials: request.materials.as_deref().unwrap_or(&scene.materials),
            bounces: request.settings.bounces,
            sun_dir: Vec3::from_array(sky.sun_direction).normalized(),
            sun_light: sun_color * sky.sun_intensity,
            sun_color,
            zenith: Vec3::from_array(sky.zenith_color),
            horizon: Vec3::from_array(sky.horizon_color),
        }
    }

    fn material(&self, index: u32) -> RenderMaterial {
        self.materials
            .get(index as usize)
            .or_else(|| self.scene.materials.last())
            .copied()
            .unwrap_or_default()
    }

    /// Radiance of the sky seen along `dir`; the sun disc only shows when `with_sun` is set.
    pub fn sky(&self, dir: Vec3, with_sun: bool) -> Vec3 {
        let t = dir.z.clamp(0.0, 1.0).powf(0.6);
        let base = self.horizon.lerp(self.zenith, t);
        if with_sun && dir.dot(self.sun_dir) > SUN_DISC_COS {
            base + self.sun_color * 8.0
        } else {
            base
        }
    }

    /// Whether the sun can be seen from `origin`; transparent surfaces block it with probability equal to
    /// their opacity.
    fn sun_visible(&self, mut origin: Vec3, rng: &mut Rng) -> bool {
        for _ in 0..8 {
            let Some(hit) = self
                .scene
                .bvh
                .closest(origin, self.sun_dir, 1e-4, f32::INFINITY)
            else {
                return true;
            };
            let material = self.material(self.scene.bvh.triangles[hit.triangle].material);
            if rng.next_f32() >= material.transparency {
                return false;
            }
            origin += self.sun_dir * (hit.t + BIAS);
        }
        true
    }

    /// Traces one ray: `t_min`/`t_max` bound the first segment only.
    pub fn trace(
        &self,
        mut origin: Vec3,
        mut dir: Vec3,
        mut t_min: f32,
        mut t_max: f32,
        rng: &mut Rng,
    ) -> Sample {
        let mut throughput = Vec3::ONE;
        let mut color = Vec3::ZERO;
        let mut covered = false;
        let mut mirror_chain = true;
        let mut depth = 0;
        for _ in 0..self.bounces + 24 {
            let Some(hit) = self.scene.bvh.closest(origin, dir, t_min, t_max) else {
                color += throughput.mul_v(self.sky(dir, mirror_chain));
                break;
            };
            if depth == 0 {
                covered = true;
            }
            let tri = &self.scene.bvh.triangles[hit.triangle];
            let material = self.material(tri.material);
            let point = origin + dir * hit.t;
            if rng.next_f32() < material.transparency {
                let tint = Vec3::ONE.lerp(Vec3::from_array(material.color), 0.5);
                throughput = throughput.mul_v(tint);
                origin = point + dir * BIAS;
                t_min = 0.0;
                t_max = (t_max - hit.t).max(0.0);
                continue;
            }
            let mut normal = tri.normal();
            if normal.dot(dir) > 0.0 {
                normal = -normal;
            }
            let view = -dir;
            let (diffuse, fresnel) = self.surface(&material, normal, view);
            let shadow_origin = point + normal * BIAS;
            let to_sun = normal.dot(self.sun_dir);
            if to_sun > 0.0 && self.sun_visible(shadow_origin, rng) {
                let half = (self.sun_dir + view).normalized();
                let shininess = shininess(material.roughness);
                let lobe = normal.dot(half).max(0.0).powf(shininess) * (shininess + 2.0) / 8.0;
                let surface = diffuse.mul_v(Vec3::ONE - fresnel) + fresnel * lobe;
                color += throughput.mul_v(surface.mul_v(self.sun_light)) * to_sun;
            }
            if depth >= self.bounces {
                break;
            }
            depth += 1;
            let probability = if material.metallic >= 0.999 {
                1.0
            } else {
                fresnel.max_component().clamp(0.05, 0.95)
            };
            origin = shadow_origin;
            t_min = 1e-4;
            t_max = f32::INFINITY;
            if rng.next_f32() < probability {
                throughput = throughput.mul_v(fresnel) / probability;
                dir = glossy_reflection(dir, normal, material.roughness, rng);
            } else {
                mirror_chain = false;
                throughput =
                    throughput.mul_v(diffuse.mul_v(Vec3::ONE - fresnel)) / (1.0 - probability);
                dir = cosine_hemisphere(normal, rng);
            }
            if !throughput.is_finite() || throughput.max_component() < 1e-4 {
                break;
            }
        }
        if !color.is_finite() {
            color = Vec3::ZERO;
        }
        let peak = color.max_component();
        if peak > FIREFLY_LIMIT {
            color *= FIREFLY_LIMIT / peak;
        }
        Sample { color, covered }
    }

    /// Diffuse color and Fresnel reflectance of a surface seen from `view`.
    fn surface(&self, m: &RenderMaterial, normal: Vec3, view: Vec3) -> (Vec3, Vec3) {
        let albedo = Vec3::from_array(m.color);
        let metal = m.metallic.clamp(0.0, 1.0);
        let f0 = Vec3::splat(0.04).lerp(albedo, metal);
        let cos = normal.dot(view).clamp(0.0, 1.0);
        let weight = (1.0 - cos).powi(5);
        let fresnel = f0 + (Vec3::ONE - f0) * weight;
        (albedo * (1.0 - metal), fresnel)
    }
}

/// Blinn-Phong exponent that matches a roughness value.
fn shininess(roughness: f32) -> f32 {
    let alpha = (roughness.clamp(0.02, 1.0)).powi(2);
    (2.0 / (alpha * alpha) - 2.0).clamp(1.0, 4000.0)
}

/// A mirror direction spread by the roughness of the surface; never points into the surface.
fn glossy_reflection(dir: Vec3, normal: Vec3, roughness: f32, rng: &mut Rng) -> Vec3 {
    let mirror = dir.reflect(normal);
    let spread = roughness * roughness;
    if spread < 1e-3 {
        return mirror;
    }
    let (t, b) = basis(mirror);
    let (u1, u2) = (rng.next_f32() - 0.5, rng.next_f32() - 0.5);
    let bent = (mirror + t * (u1 * 2.0 * spread) + b * (u2 * 2.0 * spread)).normalized();
    if bent.dot(normal) > 0.0 {
        bent
    } else {
        mirror
    }
}
