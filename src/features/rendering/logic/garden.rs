//! An optional garden around the building: a hedge of bushes close to the walls, palms at the corners and in front,
//! and a few trees at the back. Plants are ordinary library objects, so the user can move or delete them.

use super::world::project_site;
use crate::features::build::models::{PlacedObject, Transform};
use crate::features::library::models::{AssetRef, ProceduralKind};
use crate::features::projects::models::Project;

/// Distance from the walls to the line of bushes, meters.
const HEDGE: f32 = 1.5;
/// Distance between bushes along the hedge.
const SPACING: f32 = 2.2;
/// Width of the entrance left free of bushes in front of the main facade.
const ENTRANCE: f32 = 3.0;

/// A tiny deterministic generator, so the same model always gets the same garden.
struct Lcg(u64);

impl Lcg {
    fn next(&mut self) -> f32 {
        self.0 = self
            .0
            .wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(1_442_695_040_888_963_407);
        (self.0 >> 40) as f32 / (1u64 << 24) as f32
    }

    fn range(&mut self, low: f32, high: f32) -> f32 {
        low + (high - low) * self.next()
    }
}

struct Planter<'a> {
    project: &'a mut Project,
    rng: Lcg,
    ground: f32,
}

impl Planter<'_> {
    fn place(&mut self, kind: ProceduralKind, x: f32, y: f32, scale: f32) {
        let seed = (self.rng.next() * 1000.0) as u64;
        let objects = &mut self.project.document.objects;
        let id = objects.next_id.max(objects.instances.len() as u64 + 1);
        objects.next_id = id + 1;
        objects.instances.push(PlacedObject {
            id,
            asset: AssetRef::Procedural { kind, seed },
            transform: Transform {
                position: [f64::from(x), f64::from(y), f64::from(self.ground)],
                rotation: [0.0, 0.0, self.rng.range(0.0, 360.0)],
                scale: [scale; 3],
            },
        });
    }
}

/// Plants a garden around the model of `project` and marks it modified. Does nothing for a model without a
/// measurable building. Returns how many plants were added.
pub fn plant_around(project: &mut Project) -> usize {
    let site = project_site(project);
    let Some(focus) = site.focus else {
        return 0;
    };
    let before = project.document.objects.instances.len();
    let mut planter = Planter {
        project,
        rng: Lcg(0x5EED),
        ground: site.ground_z,
    };
    let (min, max) = (focus.min, focus.max);
    let (cx, cy) = ((min[0] + max[0]) * 0.5, (min[1] + max[1]) * 0.5);
    let front = (site.facade.cos(), site.facade.sin());
    let half_front = ((max[0] - min[0]) * front.0.abs() + (max[1] - min[1]) * front.1.abs()) * 0.5;
    // Hedge along the four sides, leaving the entrance of the main facade free.
    let (x0, x1, y0, y1) = (
        min[0] - HEDGE,
        max[0] + HEDGE,
        min[1] - HEDGE,
        max[1] + HEDGE,
    );
    let mut edge = |from: (f32, f32), to: (f32, f32)| {
        let length = ((to.0 - from.0).powi(2) + (to.1 - from.1).powi(2)).sqrt();
        let count = (length / SPACING).floor().max(1.0) as usize;
        for i in 0..=count {
            let t = i as f32 / count as f32;
            let (x, y) = (from.0 + (to.0 - from.0) * t, from.1 + (to.1 - from.1) * t);
            let along_front = (x - cx) * front.0 + (y - cy) * front.1;
            let across = (x - cx) * -front.1 + (y - cy) * front.0;
            if along_front > half_front && across.abs() < ENTRANCE {
                continue;
            }
            let jitter = (planter.rng.range(-0.3, 0.3), planter.rng.range(-0.3, 0.3));
            let scale = planter.rng.range(0.8, 1.3);
            planter.place(ProceduralKind::Bush, x + jitter.0, y + jitter.1, scale);
        }
    };
    edge((x0, y0), (x1, y0));
    edge((x1, y0), (x1, y1));
    edge((x1, y1), (x0, y1));
    edge((x0, y1), (x0, y0));
    // Palms at the corners and one in front of the entrance.
    let far = HEDGE + 3.0;
    for (sx, sy) in [(-1.0, -1.0), (1.0, -1.0), (1.0, 1.0), (-1.0, 1.0)] {
        let x = if sx < 0.0 { min[0] - far } else { max[0] + far };
        let y = if sy < 0.0 { min[1] - far } else { max[1] + far };
        let scale = planter.rng.range(0.9, 1.15);
        planter.place(ProceduralKind::Palm, x, y, scale);
    }
    let reach = half_front + far;
    let scale = planter.rng.range(0.9, 1.1);
    planter.place(
        ProceduralKind::Palm,
        cx + front.0 * reach + front.1 * 5.0,
        cy + front.1 * reach - front.0 * 5.0,
        scale,
    );
    // Shade trees behind the house.
    let back = -reach - 4.0;
    for side in [-1.0f32, 1.0] {
        let scale = planter.rng.range(0.9, 1.2);
        planter.place(
            ProceduralKind::Tree,
            cx + front.0 * back + front.1 * side * 9.0,
            cy + front.1 * back - front.0 * side * 9.0,
            scale,
        );
    }
    let added = planter.project.document.objects.instances.len() - before;
    planter.project.dirty = true;
    added
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::features::import_export::models::Mesh;
    use crate::features::projects::models::ProjectDocument;

    fn box_project() -> Project {
        // A 10 x 6 x 3 m block standing on Z = 0.
        let mut positions = Vec::new();
        for z in [0.0, 3.0] {
            for (x, y) in [(0.0, 0.0), (10.0, 0.0), (10.0, 6.0), (0.0, 6.0)] {
                positions.push([x, y, z]);
            }
        }
        let indices = vec![
            0, 1, 2, 0, 2, 3, 4, 6, 5, 4, 7, 6, 0, 4, 5, 0, 5, 1, 1, 5, 6, 1, 6, 2, 2, 6, 7, 2, 7,
            3, 3, 7, 4, 3, 4, 0,
        ];
        Project {
            id: [1; 16],
            document: ProjectDocument::default(),
            meshes: vec![Mesh {
                positions,
                indices,
                material: None,
                uvs: Vec::new(),
            }],
            path: None,
            legacy_source: None,
            read_only: false,
            dirty: false,
        }
    }

    #[test]
    fn the_garden_surrounds_the_building_and_never_touches_it() {
        let mut project = box_project();
        let added = plant_around(&mut project);
        assert!(added > 12, "{added} plants");
        assert!(project.dirty);
        for plant in &project.document.objects.instances {
            let [x, y, _] = plant.transform.position;
            let inside = (0.0..=10.0).contains(&x) && (0.0..=6.0).contains(&y);
            assert!(!inside, "a plant at ({x}, {y}) is inside the house");
        }
    }

    #[test]
    fn palms_bushes_and_trees_are_all_planted_and_it_is_deterministic() {
        let mut a = box_project();
        let mut b = box_project();
        plant_around(&mut a);
        plant_around(&mut b);
        assert_eq!(a.document.objects, b.document.objects);
        let kinds: std::collections::HashSet<_> = a
            .document
            .objects
            .instances
            .iter()
            .filter_map(|o| match o.asset {
                AssetRef::Procedural { kind, .. } => Some(kind),
                AssetRef::Imported { .. } => None,
            })
            .collect();
        for kind in [
            ProceduralKind::Palm,
            ProceduralKind::Bush,
            ProceduralKind::Tree,
        ] {
            assert!(kinds.contains(&kind), "{kind:?}");
        }
    }

    #[test]
    fn an_empty_project_gets_no_garden() {
        let mut project = box_project();
        project.meshes.clear();
        assert_eq!(plant_around(&mut project), 0);
    }
}
