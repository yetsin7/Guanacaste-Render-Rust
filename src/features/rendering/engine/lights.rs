//! Artificial lights for the tracer: picks the lights worth a shadow ray at a surface point and hands over the
//! light the lighting domain computes (point, spot, IES, rectangular area and linear lights).
//!
//! The lighting domain works in a Y-up world; the scene is Z up. Points are converted on the way in and out with
//! the same axis change the importers use for Y-up files: `(x, y, z)` (Y up) is `(x, -z, y)` (Z up).

use super::sampling::Rng;
use super::vec3::Vec3;
use crate::features::lighting::logic::{light_samples, LightSample};
use crate::features::lighting::models::Light;

/// Most lights that get a shadow ray at one surface point; more are picked in proportion to their light.
const MAX_LIGHT_RAYS: usize = 6;
/// Samples per axis on area lights (squared), one of which is traced per visit.
const AREA_SAMPLES_PER_AXIS: u32 = 3;

/// Scene space (Z up) to the lighting world (Y up).
fn to_lighting(v: Vec3) -> [f32; 3] {
    [v.x, v.z, -v.y]
}

/// The lighting world to scene space.
fn from_lighting(v: [f32; 3]) -> Vec3 {
    Vec3::new(v[0], -v[2], v[1])
}

/// Light arriving at a point from one sample of a light.
#[derive(Debug, Clone, Copy)]
pub struct Incoming {
    /// The point of the light to aim the shadow ray at, scene space.
    pub position: Vec3,
    /// Illuminance in render units (already includes the cosine at the surface and the sampling weight).
    pub irradiance: Vec3,
    /// Whether the light is blocked by geometry.
    pub shadows: bool,
}

/// The enabled lights of a render.
pub struct SceneLights<'a> {
    lights: &'a [Light],
    unit: f32,
}

impl<'a> SceneLights<'a> {
    /// Lights whose illuminance in lux is divided by `reference_lux` to get render units.
    pub fn new(lights: &'a [Light], reference_lux: f32) -> Self {
        Self {
            lights,
            unit: 1.0 / reference_lux.max(1e-3),
        }
    }

    /// Whether there is nothing to light the scene.
    pub fn is_empty(&self) -> bool {
        self.lights.is_empty()
    }

    /// Fills `out` with the samples to trace at `point` with unit `normal`.
    pub fn gather(&self, point: Vec3, normal: Vec3, rng: &mut Rng, out: &mut Vec<Incoming>) {
        out.clear();
        let p = to_lighting(point);
        let n = to_lighting(normal);
        let mut weights: Vec<f32> = Vec::new();
        for light in self.lights {
            let samples = light_samples(light, p, n, AREA_SAMPLES_PER_AXIS);
            if samples.is_empty() {
                continue;
            }
            let count = samples.len();
            let pick: &LightSample = &samples[(rng.next_u32() as usize) % count];
            let e = pick.irradiance;
            let scale = self.unit * count as f32;
            let irradiance = Vec3::new(e.r, e.g, e.b) * scale;
            let luminance = irradiance.max_component();
            if luminance <= 1e-6 {
                continue;
            }
            out.push(Incoming {
                position: from_lighting(pick.position),
                irradiance,
                shadows: light.cast_shadows,
            });
            weights.push(luminance);
        }
        if out.len() > MAX_LIGHT_RAYS {
            thin_out(out, &weights, rng);
        }
    }
}

/// Keeps `MAX_LIGHT_RAYS` picks chosen in proportion to their weight, each re-weighted so the sum stays unbiased.
fn thin_out(out: &mut Vec<Incoming>, weights: &[f32], rng: &mut Rng) {
    let total: f32 = weights.iter().sum();
    let mut kept = Vec::with_capacity(MAX_LIGHT_RAYS);
    for _ in 0..MAX_LIGHT_RAYS {
        let mut target = rng.next_f32() * total;
        let mut chosen = out.len() - 1;
        for (i, w) in weights.iter().enumerate() {
            if target < *w {
                chosen = i;
                break;
            }
            target -= w;
        }
        // Probability of this pick is `w / total`; K picks are averaged.
        let mut pick = out[chosen];
        pick.irradiance *= total / (MAX_LIGHT_RAYS as f32 * weights[chosen].max(1e-9));
        kept.push(pick);
    }
    *out = kept;
}
