//! Light portals: the glass of windows and glazed doors that let the sky into a room. Surfaces inside sample the sky
//! through them directly, which lights interiors smoothly instead of waiting for a random bounce to find the window.

use super::bvh::Bvh;
use super::sampling::Rng;
use super::types::RenderMaterial;
use super::vec3::Vec3;

/// Glass must be at least this transparent to be a portal.
const MIN_TRANSPARENCY: f32 = 0.5;
/// Panes smaller than this (square meters) are ignored.
const MIN_AREA: f32 = 0.02;
/// How far outside a pane the sky must be reachable, meters.
const OUTSIDE: f32 = 30.0;

/// One pane triangle with the side that faces the outdoors.
#[derive(Debug, Clone, Copy)]
struct Portal {
    v0: Vec3,
    e1: Vec3,
    e2: Vec3,
    /// Unit normal pointing outdoors.
    outward: Vec3,
}

/// The portals of a scene, sampled in proportion to their area.
#[derive(Debug, Default)]
pub struct Portals {
    portals: Vec<Portal>,
    /// Running total of the areas, for sampling.
    cumulative: Vec<f32>,
}

/// A point chosen on a portal.
#[derive(Debug, Clone, Copy)]
pub struct PortalSample {
    /// Position on the pane.
    pub point: Vec3,
    /// Unit normal facing outdoors.
    pub outward: Vec3,
    /// Total area of all portals, which is the sampling weight of a uniformly chosen point.
    pub total_area: f32,
}

/// How many of a few rays leaving `center` along `dir` reach the open air.
fn open_rays(bvh: &Bvh, center: Vec3, dir: Vec3) -> usize {
    let up = Vec3::new(0.0, 0.0, 1.0);
    [
        dir,
        (dir + up * 0.6).normalized(),
        (dir + up * 1.5).normalized(),
    ]
    .iter()
    .filter(|d| bvh.closest(center + **d * 0.1, **d, 0.0, OUTSIDE).is_none())
    .count()
}

impl Portals {
    /// The glass triangles of `bvh` that separate a room from the open air.
    pub fn build(bvh: &Bvh, materials: &[RenderMaterial]) -> Self {
        let mut found = Self::default();
        let mut total = 0.0;
        for tri in &bvh.triangles {
            let transparent = materials
                .get(tri.material as usize)
                .is_some_and(|m| m.transparency >= MIN_TRANSPARENCY);
            let area = tri.e1.cross(tri.e2).length() * 0.5;
            if !transparent || area < MIN_AREA {
                continue;
            }
            let normal = tri.normal();
            let center = tri.v0 + (tri.e1 + tri.e2) * (1.0 / 3.0);
            let (front, back) = (
                open_rays(bvh, center, normal),
                open_rays(bvh, center, -normal),
            );
            let outward = match (front > 0, back > 0) {
                (true, false) => normal,
                (false, true) => -normal,
                // Open or blocked on both sides: a free-standing pane or a glass door between rooms.
                _ => continue,
            };
            total += area;
            found.cumulative.push(total);
            found.portals.push(Portal {
                v0: tri.v0,
                e1: tri.e1,
                e2: tri.e2,
                outward,
            });
        }
        found
    }

    /// Whether the scene has no portals.
    pub fn is_empty(&self) -> bool {
        self.portals.is_empty()
    }

    /// A uniformly random point on the portals, or `None` without portals.
    pub fn sample(&self, rng: &mut Rng) -> Option<PortalSample> {
        let total = *self.cumulative.last()?;
        let target = rng.next_f32() * total;
        let index = self
            .cumulative
            .partition_point(|c| *c < target)
            .min(self.portals.len() - 1);
        let portal = &self.portals[index];
        let (mut a, mut b) = (rng.next_f32(), rng.next_f32());
        if a + b > 1.0 {
            a = 1.0 - a;
            b = 1.0 - b;
        }
        Some(PortalSample {
            point: portal.v0 + portal.e1 * a + portal.e2 * b,
            outward: portal.outward,
            total_area: total,
        })
    }
}
