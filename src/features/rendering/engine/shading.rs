//! Light transport: sun and artificial lights with shadow rays, sky light, PBR surfaces, emission, glass and fog.

use super::atmosphere::Atmosphere;
use super::geometry::RenderScene;
use super::interior::room_lamps;
use super::lights::{Incoming, SceneLights};
use super::sampling::{basis, cosine_hemisphere, Rng};
use super::surface::{evaluate, Surf};
use super::types::RenderRequest;
use super::vec3::Vec3;

/// Offset that keeps secondary rays from hitting the surface they start on.
const BIAS: f32 = 1e-3;
/// Samples brighter than this are scaled down to tame isolated bright dots.
const FIREFLY_LIMIT: f32 = 12.0;
/// Radius of the sun disc as seen from the ground, in radians (softens shadow edges very slightly).
const SUN_RADIUS: f32 = 0.0047;
/// How far outside a window the sky must be reachable for its light to count, meters.
const PORTAL_REACH: f32 = 40.0;

/// Surfaces more transparent than this (opacity below it) are glass: light and view pass through deterministically.
const GLASS_ALPHA: f32 = 0.5;
/// Tint of the sky light that reaches a surface by a diffuse bounce: the sky is blue, but the light that fills the
/// shade of a house also bounces off warm ground and walls, so shaded stucco must not turn gray-blue.
const WARM_BOUNCE: Vec3 = Vec3 {
    x: 1.06,
    y: 1.0,
    z: 0.88,
};

/// What one camera ray brought back.
#[derive(Debug, Clone, Copy)]
pub struct Sample {
    /// Linear light.
    pub color: Vec3,
    /// False when the ray left the scene without touching anything.
    pub covered: bool,
    /// Depth along the view axis of the first surface, or the far plane for sky.
    pub depth: f32,
    /// Unit normal of the first surface (zero for sky).
    pub normal: Vec3,
}

/// Scene, atmosphere and lights prepared for tracing.
pub struct Shader<'a> {
    scene: &'a RenderScene,
    atmosphere: Atmosphere,
    lights: SceneLights<'a>,
    bounces: u32,
    far: f32,
    /// The picture is taken inside a room (interior mode).
    indoor: bool,
    /// Angle covered by one pixel, radians.
    pixel_angle: f32,
}

impl<'a> Shader<'a> {
    /// Prepares the atmosphere and lights of `request` for `scene`.
    pub fn new(scene: &'a RenderScene, request: &'a RenderRequest) -> Self {
        let atmosphere = Atmosphere::new(&request.environment);
        // A room lit only through its windows gets warm ceiling lamps, unless the project has lights of its own.
        let lamps = if request.post.auto_exposure && request.lights.is_empty() {
            room_lamps(scene, request.camera.position, atmosphere.reference_lux)
        } else {
            Vec::new()
        };
        let lights = if lamps.is_empty() {
            SceneLights::new(request.lights.as_slice(), atmosphere.reference_lux)
        } else {
            SceneLights::new(lamps, atmosphere.reference_lux)
        };
        let camera = &request.camera;
        let half = (camera.fov_y_degrees.clamp(1.0, 170.0).to_radians() * 0.5).tan();
        Self {
            scene,
            atmosphere,
            lights,
            bounces: request.settings.bounces,
            far: camera.far,
            indoor: request.post.auto_exposure,
            pixel_angle: 2.0 * half / camera.height.max(1) as f32,
        }
    }

    /// Whether `origin` sees `dir` for `max_t` meters; transparent surfaces block with probability equal to their
    /// opacity.
    fn visible(&self, mut origin: Vec3, dir: Vec3, mut max_t: f32, rng: &mut Rng) -> bool {
        for _ in 0..8 {
            let Some(hit) = self.scene.bvh.any_hit(origin, dir, 1e-4, max_t) else {
                return true;
            };
            let material =
                &self.scene.materials[self.scene.bvh.triangles[hit.triangle].material as usize];
            // Glass lets light through for sure (a random block made the light patches behind windows speckled).
            if material.transparency < 1.0 - GLASS_ALPHA && rng.next_f32() >= material.transparency
            {
                return false;
            }
            origin += dir * (hit.t + BIAS);
            max_t -= hit.t + BIAS;
        }
        true
    }

    /// The sun direction nudged inside the sun disc.
    fn jittered_sun(&self, rng: &mut Rng) -> Vec3 {
        let sun = self.atmosphere.sun_direction;
        let (t, b) = basis(sun);
        let (r, phi) = (
            SUN_RADIUS * rng.next_f32().sqrt(),
            std::f32::consts::TAU * rng.next_f32(),
        );
        (sun + t * (r * phi.cos()) + b * (r * phi.sin())).normalized()
    }

    /// Reflected light of a surface for light arriving along `light` (unit, toward the light), per unit of
    /// illuminance on the surface plane.
    fn brdf(&self, s: &Surf, light: Vec3, view: Vec3) -> Vec3 {
        let (diffuse, fresnel) = surface_terms(s, view);
        let half = (light + view).normalized();
        let n = shininess(s.roughness);
        let lobe = s.normal.dot(half).max(0.0).powf(n) * (n + 2.0) / 8.0;
        diffuse.mul_v(Vec3::ONE - fresnel) + fresnel * lobe
    }

    /// Adds the direct light (sun and lamps) reaching `point` to `color`.
    #[allow(clippy::too_many_arguments)]
    fn direct(
        &self,
        s: &Surf,
        point: Vec3,
        origin: Vec3,
        view: Vec3,
        rng: &mut Rng,
        scratch: &mut Vec<Incoming>,
    ) -> Vec3 {
        let mut sum = Vec3::ZERO;
        let sun = self.jittered_sun(rng);
        let to_sun = s.normal.dot(sun);
        if to_sun > 0.0 && self.visible(origin, sun, f32::INFINITY, rng) {
            sum += self.brdf(s, sun, view).mul_v(self.atmosphere.sun_light) * to_sun;
        }
        if !self.lights.is_empty() {
            self.lights.gather(point, s.normal, rng, scratch);
            for incoming in scratch.iter() {
                let to = incoming.position - origin;
                let distance = to.length();
                if distance < 1e-4 {
                    continue;
                }
                let dir = to / distance;
                if incoming.shadows && !self.visible(origin, dir, distance - 2.0 * BIAS, rng) {
                    continue;
                }
                sum += self.brdf(s, dir, view).mul_v(incoming.irradiance);
            }
        }
        sum + self.sky_through_windows(s, origin, view, rng)
    }

    /// Sky light reaching `origin` through one randomly chosen window pane (next-event estimation): the pane is
    /// sampled as an area light of sky radiance, if the point is on its indoor side and both the way to the pane
    /// and the way from the pane out to the open air are clear.
    fn sky_through_windows(&self, s: &Surf, origin: Vec3, view: Vec3, rng: &mut Rng) -> Vec3 {
        let Some(portal) = self.scene.portals.sample(rng) else {
            return Vec3::ZERO;
        };
        let to = portal.point - origin;
        let distance2 = to.dot(to).max(0.01);
        let distance = distance2.sqrt();
        let dir = to / distance;
        let (cos_surface, cos_pane) = (s.normal.dot(dir), dir.dot(portal.outward));
        if cos_surface <= 0.02 || cos_pane <= 0.02 {
            return Vec3::ZERO;
        }
        if !self.visible(origin, dir, distance - 2.0 * BIAS, rng) {
            return Vec3::ZERO;
        }
        let outside = portal.point + dir * (2.0 * BIAS + 0.02);
        if !self.visible(outside, dir, PORTAL_REACH, rng) {
            return Vec3::ZERO;
        }
        let solid_angle = cos_pane * portal.total_area / distance2;
        self.brdf(s, dir, view)
            .mul_v(self.atmosphere.sky(dir, false))
            * (cos_surface * solid_angle.min(2.0 * std::f32::consts::PI) / std::f32::consts::PI)
    }

    /// Traces one ray: `t_min`/`t_max` bound the first segment only; `axis_cos` is the cosine between the ray and
    /// the view axis (it turns distances into depths).
    #[allow(clippy::too_many_arguments)]
    pub fn trace(
        &self,
        mut origin: Vec3,
        mut dir: Vec3,
        mut t_min: f32,
        mut t_max: f32,
        axis_cos: f32,
        rng: &mut Rng,
        scratch: &mut Vec<Incoming>,
    ) -> Sample {
        let mut throughput = Vec3::ONE;
        let mut color = Vec3::ZERO;
        let mut covered = false;
        let mut mirror_chain = true;
        // A diffuse path that went through a pane: the sky it reaches is already counted by the window sampling.
        let mut via_pane = false;
        let mut depth = 0;
        let mut travelled = 0.0f32;
        let mut first: Option<(f32, Vec3)> = None;
        for _ in 0..self.bounces + 24 {
            let Some(hit) = self.scene.bvh.closest(origin, dir, t_min, t_max) else {
                if !(via_pane && !self.scene.portals.is_empty()) {
                    let sky = self.atmosphere.sky(dir, mirror_chain);
                    color += throughput.mul_v(if mirror_chain {
                        sky
                    } else {
                        sky.mul_v(WARM_BOUNCE)
                    });
                }
                break;
            };
            let tri = &self.scene.bvh.triangles[hit.triangle];
            let mut normal = tri.normal();
            if normal.dot(dir) > 0.0 {
                normal = -normal;
            }
            travelled += hit.t;
            let s = evaluate(
                self.scene,
                tri,
                &hit,
                normal,
                travelled * self.pixel_angle,
                depth == 0 || mirror_chain,
            );
            let point = origin + dir * hit.t;
            let view = -dir;
            if s.alpha < GLASS_ALPHA || rng.next_f32() > s.alpha {
                // A pane always lets the view through, tinted, and adds a smooth reflection of the sky: picking
                // between the two (or treating a few samples as an opaque surface) at random speckled the glass.
                let fresnel = if s.roughness < 0.5 && s.ior > 1.05 {
                    schlick(s.ior, normal.dot(view))
                } else {
                    0.0
                };
                // Outdoors only: from inside a room, glass would mirror the blue sky onto lamps and furniture.
                if fresnel > 0.0 && !self.indoor {
                    let mirror = dir.reflect(normal);
                    let upward = (mirror.z * 2.0).clamp(0.0, 1.0);
                    color +=
                        throughput.mul_v(self.atmosphere.sky(mirror, false)) * (fresnel * upward);
                }
                throughput = throughput.mul_v(Vec3::ONE.lerp(s.albedo, 0.5)) * (1.0 - fresnel);
                via_pane |= !mirror_chain;
                origin = point + dir * BIAS;
                t_min = 0.0;
                t_max = (t_max - hit.t).max(0.0);
                continue;
            }
            if depth == 0 {
                covered = true;
            }
            if first.is_none() {
                // The geometric normal guides the denoiser: bump detail must not look like an edge.
                first = Some((travelled, normal));
            }
            let material = &self.scene.materials[tri.material as usize];
            let glow = s.emission
                + Vec3::from_array(
                    material
                        .emission_nits
                        .map(|n| self.atmosphere.nits_to_units(n)),
                ) * (std::f32::consts::PI / self.atmosphere.reference_lux);
            color += throughput.mul_v(glow);
            let mut s = s;
            if s.normal.dot(view) <= 0.0 {
                s.normal = normal;
            }
            let shadow_origin = point + normal * BIAS;
            color += throughput.mul_v(self.direct(&s, point, shadow_origin, view, rng, scratch));
            if depth >= self.bounces {
                break;
            }
            depth += 1;
            let (diffuse, fresnel) = surface_terms(&s, view);
            let probability = if s.metallic >= 0.999 {
                1.0
            } else {
                fresnel.max_component().clamp(0.05, 0.95)
            };
            origin = shadow_origin;
            t_min = 1e-4;
            t_max = f32::INFINITY;
            if rng.next_f32() < probability {
                throughput = throughput.mul_v(fresnel) / probability;
                dir = glossy_reflection(dir, s.normal, s.roughness, rng);
            } else {
                mirror_chain = false;
                throughput = throughput.mul_v(diffuse.mul_v(Vec3::ONE - fresnel) * s.occlusion)
                    / (1.0 - probability);
                dir = cosine_hemisphere(s.normal, rng);
            }
            if !throughput.is_finite() || throughput.max_component() < 1e-4 {
                break;
            }
        }
        let (distance, normal) = first.unwrap_or((self.far, Vec3::ZERO));
        if first.is_some() {
            color = self.atmosphere.fog(color, distance);
        }
        if !color.is_finite() {
            color = Vec3::ZERO;
        }
        let peak = color.max_component();
        if peak > FIREFLY_LIMIT {
            color *= FIREFLY_LIMIT / peak;
        }
        Sample {
            color,
            covered,
            depth: if first.is_some() {
                distance * axis_cos
            } else {
                self.far
            },
            normal,
        }
    }
}

/// Diffuse color and Fresnel reflectance of a surface seen from `view`.
fn surface_terms(s: &Surf, view: Vec3) -> (Vec3, Vec3) {
    let metal = s.metallic.clamp(0.0, 1.0);
    let f_dielectric = ((s.ior - 1.0) / (s.ior + 1.0)).powi(2).clamp(0.02, 0.2);
    let f0 = Vec3::splat(f_dielectric).lerp(s.albedo, metal);
    let cos = s.normal.dot(view).clamp(0.0, 1.0);
    // Rough surfaces do not mirror the sky at grazing angles: the Fresnel peak shrinks with the roughness.
    let peak = Vec3::splat(1.0 - s.roughness).max(f0);
    let fresnel = f0 + (peak - f0) * (1.0 - cos).powi(5);
    (s.albedo * (1.0 - metal), fresnel)
}

/// Schlick reflectance of a dielectric with index `ior` at the cosine `cos`.
fn schlick(ior: f32, cos: f32) -> f32 {
    let f0 = ((ior - 1.0) / (ior + 1.0)).powi(2);
    f0 + (1.0 - f0) * (1.0 - cos.clamp(0.0, 1.0)).powi(5)
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
