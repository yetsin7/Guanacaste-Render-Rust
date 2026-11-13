//! Creating instances: placing the active asset, duplicating in an array, scattering, and the selection tools
//! (align, snap to surface, random rotation).

use super::math::{Ray, Vec3};
use crate::features::build::models::{AlignMode, ObjectsState, Transform};
use crate::features::library::models::AssetRef;
use std::f64::consts::TAU;

impl ObjectsState {
    /// The asset a new instance shows: the active asset with the browser seed, or a fresh seed when seeds vary.
    fn fresh_asset(&mut self, asset: AssetRef) -> AssetRef {
        if self.placement.vary_seed {
            asset.with_seed(self.rng.next_u64() % 1_000_000)
        } else {
            asset.with_seed(self.browser.seed)
        }
    }

    /// Rounds `value` to the placement grid (unchanged when the grid is free).
    pub fn snap_to_grid(&self, value: f64) -> f64 {
        let grid = f64::from(self.placement.grid);
        if grid > 0.0 {
            (value / grid).round() * grid
        } else {
            value
        }
    }

    /// A transform at `point` with the random yaw and scale of the placement options.
    fn varied_transform(&mut self, point: Vec3) -> Transform {
        let mut transform = Transform::at(point);
        if self.placement.random_rotation {
            transform.rotation[2] = self.rng.range(0.0, 360.0);
        }
        let spread = self.placement.random_scale.clamp(0.0, 0.5);
        if spread > 0.0 {
            let factor = 1.0 + self.rng.range(-spread, spread);
            transform.scale = [factor; 3];
        }
        transform
    }

    /// Places the active asset at `point`. Returns the new id, or `None` without an active asset.
    pub fn place_active(&mut self, point: Vec3) -> Option<u64> {
        let asset = self.browser.active?;
        let asset = self.fresh_asset(asset);
        let point = [
            self.snap_to_grid(point[0]),
            self.snap_to_grid(point[1]),
            point[2],
        ];
        let transform = self.varied_transform(point);
        self.add_objects(vec![(asset, transform)]).first().copied()
    }

    /// Places the active asset where `ray` meets the scene (surface when snapping, otherwise the ground).
    pub fn place_at_ray(&mut self, ray: &Ray) -> Option<u64> {
        let hit = if self.placement.snap_to_surface {
            self.surface_hit(ray, &Default::default())?
        } else {
            let t = super::math::ray_plane(ray, [0.0; 3], [0.0, 0.0, 1.0])?;
            super::picking::SurfaceHit {
                point: ray.at(t),
                normal: [0.0, 0.0, 1.0],
                id: None,
                distance: t,
            }
        };
        self.place_active(hit.point)
    }

    /// Copies every selected instance `mass.copies` times, each copy offset by `mass.offset` from the previous.
    pub fn duplicate_array(&mut self) {
        let copies = self.mass.copies.clamp(1, 500);
        let offset = self.mass.offset.map(f64::from);
        let sources: Vec<_> = self
            .doc
            .instances
            .iter()
            .filter(|i| self.selection.contains(&i.id))
            .map(|i| (i.asset, i.transform))
            .collect();
        let mut objects = Vec::new();
        for step in 1..=copies {
            for (asset, transform) in &sources {
                let mut copy = *transform;
                for axis in 0..3 {
                    copy.position[axis] += offset[axis] * f64::from(step);
                }
                objects.push((*asset, copy));
            }
        }
        self.add_objects(objects);
    }

    /// Scatters `mass.scatter_count` instances of the active asset in a disk of `mass.scatter_radius` around
    /// `center`, dropped onto the surface when snapping is on.
    pub fn scatter(&mut self, center: Vec3) {
        let Some(active) = self.browser.active else {
            return;
        };
        let count = self.mass.scatter_count.clamp(1, 2000);
        let radius = f64::from(self.mass.scatter_radius.max(0.0));
        let mut objects = Vec::new();
        for _ in 0..count {
            let angle = f64::from(self.rng.unit()) * TAU;
            let distance = f64::from(self.rng.unit()).sqrt() * radius;
            let x = center[0] + angle.cos() * distance;
            let y = center[1] + angle.sin() * distance;
            let z = if self.placement.snap_to_surface {
                self.surface_height(x, y, &Default::default())
            } else {
                center[2]
            };
            let asset = self.fresh_asset(active);
            objects.push((asset, self.varied_transform([x, y, z])));
        }
        self.add_objects(objects);
    }

    /// Aligns the selection along `axis` (0 X, 1 Y, 2 Z) to the minimum, average or maximum position.
    pub fn align(&mut self, axis: usize, mode: AlignMode) {
        let values: Vec<f64> = self
            .doc
            .instances
            .iter()
            .filter(|i| self.selection.contains(&i.id))
            .map(|i| i.transform.position[axis])
            .collect();
        if values.len() < 2 {
            return;
        }
        let target = match mode {
            AlignMode::Min => values.iter().copied().fold(f64::MAX, f64::min),
            AlignMode::Max => values.iter().copied().fold(f64::MIN, f64::max),
            AlignMode::Center => values.iter().sum::<f64>() / values.len() as f64,
        };
        self.edit_selection(|_, t| t.position[axis] = target);
    }

    /// Drops every selected instance onto the surface below (or above) it.
    pub fn snap_selection_to_surface(&mut self) {
        let heights: Vec<(u64, f64)> = self
            .doc
            .instances
            .iter()
            .filter(|i| self.selection.contains(&i.id))
            .map(|i| (i.id, i.transform.position))
            .collect::<Vec<_>>()
            .into_iter()
            .map(|(id, p)| (id, self.surface_height(p[0], p[1], &self.selection.clone())))
            .collect();
        self.edit_selection(|object, t| {
            if let Some((_, z)) = heights.iter().find(|(id, _)| *id == object.id) {
                t.position[2] = *z;
            }
        });
    }

    /// Gives every selected instance a random yaw.
    pub fn randomize_rotation(&mut self) {
        let mut rng = self.rng.clone();
        self.edit_selection(|_, t| t.rotation[2] = rng.range(0.0, 360.0));
        self.rng = rng;
    }

    /// Resets rotation and scale of the selection.
    pub fn reset_transforms(&mut self) {
        self.edit_selection(|_, t| {
            t.rotation = [0.0; 3];
            t.scale = [1.0; 3];
        });
    }
}
