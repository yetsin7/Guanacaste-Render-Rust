//! Rays against the placed objects and the ground: picking, surface snapping and dropping onto the surface.

use super::math::{self, Ray, Vec3};
use crate::features::build::models::ObjectsState;
use std::collections::BTreeSet;

/// Where a ray meets the scene.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SurfaceHit {
    /// World point.
    pub point: Vec3,
    /// Unit surface normal, facing the ray.
    pub normal: Vec3,
    /// The instance hit, or `None` for the ground plane (Z = 0).
    pub id: Option<u64>,
    /// Distance along the ray.
    pub distance: f64,
}

impl ObjectsState {
    /// The nearest instance (not in `exclude`) hit by `ray`, by exact triangle test.
    pub fn pick(&mut self, ray: &Ray, exclude: &BTreeSet<u64>) -> Option<SurfaceHit> {
        let candidates: Vec<_> = self
            .doc
            .instances
            .iter()
            .filter(|i| !exclude.contains(&i.id))
            .map(|i| (i.id, i.asset, i.transform))
            .collect();
        let mut best: Option<SurfaceHit> = None;
        for (id, asset, transform) in candidates {
            let Some(scene) = self.cache.scene(asset) else {
                continue;
            };
            let local = math::to_local_ray(&transform, ray);
            let Some(bounds) = scene.bounds() else {
                continue;
            };
            let (min, max) = (math::widen(bounds.min), math::widen(bounds.max));
            if math::ray_box(&local, min, max).is_none() {
                continue;
            }
            for mesh in &scene.meshes {
                for tri in mesh.indices.chunks_exact(3) {
                    let [a, b, c] = [0, 1, 2].map(|k| math::widen(mesh.positions[tri[k] as usize]));
                    let Some(t) = math::ray_triangle(&local, a, b, c) else {
                        continue;
                    };
                    if best.is_some_and(|hit| hit.distance <= t) {
                        continue;
                    }
                    let local_normal = math::cross(math::sub(b, a), math::sub(c, a));
                    let mut normal = math::normalize(math::mul(
                        &math::rotation(transform.rotation),
                        local_normal,
                    ));
                    if math::dot(normal, ray.direction) > 0.0 {
                        normal = math::scale(normal, -1.0);
                    }
                    best = Some(SurfaceHit {
                        point: ray.at(t),
                        normal,
                        id: Some(id),
                        distance: t,
                    });
                }
            }
        }
        best
    }

    /// The nearest of the instances (not in `exclude`) and the ground plane along `ray`.
    pub fn surface_hit(&mut self, ray: &Ray, exclude: &BTreeSet<u64>) -> Option<SurfaceHit> {
        let object = self.pick(ray, exclude);
        let ground = math::ray_plane(ray, [0.0; 3], [0.0, 0.0, 1.0]).map(|t| SurfaceHit {
            point: ray.at(t),
            normal: [0.0, 0.0, 1.0],
            id: None,
            distance: t,
        });
        match (object, ground) {
            (Some(o), Some(g)) => Some(if o.distance <= g.distance { o } else { g }),
            (o, g) => o.or(g),
        }
    }

    /// Height of the surface under the horizontal position `(x, y)`, looking down from far above; 0 over nothing.
    pub fn surface_height(&mut self, x: f64, y: f64, exclude: &BTreeSet<u64>) -> f64 {
        let ray = Ray::new([x, y, 10_000.0], [0.0, 0.0, -1.0]);
        self.surface_hit(&ray, exclude)
            .map_or(0.0, |hit| hit.point[2])
    }
}
