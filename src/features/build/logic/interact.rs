//! Pointer interaction over the 3D view, driven by rays the viewport computes: press, drag and release. The viewport
//! agent only has to convert the pointer to a [`Ray`]; picking, gizmo math and undo live here.

use super::math::{self, Ray, Vec3};
use crate::features::build::models::{AxisLock, Command, Drag, GizmoMode, ObjectsState};

impl ObjectsState {
    /// The pointer went down. In `Place` mode this places the active asset; otherwise it selects the instance under
    /// the ray (`additive` toggles it in the selection) and starts a drag of the selection.
    pub fn pointer_press(&mut self, ray: &Ray, additive: bool) {
        self.drag = None;
        if self.mode == GizmoMode::Place && self.browser.active.is_some() {
            self.place_at_ray(ray);
            return;
        }
        // Without an asset to place, a click selects and a drag moves, like the move tool.
        let mode = if self.mode == GizmoMode::Place {
            GizmoMode::Move
        } else {
            self.mode
        };
        let hit = self.pick(ray, &Default::default());
        match hit.and_then(|h| h.id) {
            Some(id) => {
                if !self.selection.contains(&id) || additive {
                    self.select(id, additive);
                }
            }
            None if !additive => self.clear_selection(),
            None => {}
        }
        if self.selection.is_empty() {
            return;
        }
        let selected = self.selection.clone();
        let anchor = hit
            .filter(|h| h.id.is_some_and(|id| selected.contains(&id)))
            .map(|h| h.point)
            .or_else(|| self.surface_hit(ray, &selected).map(|h| h.point));
        let Some(anchor) = anchor else { return };
        let start: Vec<_> = self
            .doc
            .instances
            .iter()
            .filter(|i| selected.contains(&i.id))
            .map(|i| (i.id, i.transform))
            .collect();
        let sum = start
            .iter()
            .fold([0.0; 3], |acc, (_, t)| math::add(acc, t.position));
        let pivot = math::scale(sum, 1.0 / start.len() as f64);
        let horizontal = math::normalize([ray.direction[0], ray.direction[1], 0.0]);
        self.drag = Some(Drag {
            mode,
            start,
            anchor,
            pivot,
            plane_normal: horizontal,
        });
    }

    /// The pointer moved while down: updates the selection live (not yet in the history).
    pub fn pointer_drag(&mut self, ray: &Ray) {
        let Some(drag) = self.drag.clone() else {
            return;
        };
        let selected: std::collections::BTreeSet<u64> =
            drag.start.iter().map(|(id, _)| *id).collect();
        let transforms = match drag.mode {
            GizmoMode::Move => self.dragged_move(&drag, ray, &selected),
            GizmoMode::Rotate => dragged_rotation(&drag, ray, self.placement.rotation_step),
            GizmoMode::Scale => dragged_scale(&drag, ray, self.axis),
            GizmoMode::Place => None,
        };
        let Some(transforms) = transforms else { return };
        for (id, transform) in transforms {
            if let Some(object) = self.doc.instances.iter_mut().find(|i| i.id == id) {
                object.transform = transform;
            }
        }
    }

    /// The pointer was released: the drag becomes one undoable command.
    pub fn pointer_release(&mut self) {
        let Some(drag) = self.drag.take() else {
            return;
        };
        let changes: Vec<_> = drag
            .start
            .iter()
            .filter_map(|(id, before)| {
                let after = self.instance(*id)?.transform;
                (after != *before).then_some((*id, *before, after))
            })
            .collect();
        if !changes.is_empty() {
            self.history.record(Command::Transform(changes));
            self.dirty = true;
        }
    }

    /// Cancels the drag, restoring the transforms from before it started.
    pub fn pointer_cancel(&mut self) {
        if let Some(drag) = self.drag.take() {
            for (id, before) in drag.start {
                if let Some(object) = self.doc.instances.iter_mut().find(|i| i.id == id) {
                    object.transform = before;
                }
            }
        }
    }

    fn dragged_move(
        &mut self,
        drag: &Drag,
        ray: &Ray,
        selected: &std::collections::BTreeSet<u64>,
    ) -> Option<Vec<(u64, crate::features::build::models::Transform)>> {
        let delta: Vec3 = match self.axis {
            AxisLock::Z => {
                let t = math::ray_plane(ray, drag.anchor, drag.plane_normal)?;
                [0.0, 0.0, ray.at(t)[2] - drag.anchor[2]]
            }
            lock => {
                let current = if self.placement.snap_to_surface && lock == AxisLock::Free {
                    self.surface_hit(ray, selected)?.point
                } else {
                    let t = math::ray_plane(ray, drag.anchor, [0.0, 0.0, 1.0])?;
                    ray.at(t)
                };
                let mut delta = math::sub(current, drag.anchor);
                match lock {
                    AxisLock::X => delta = [delta[0], 0.0, 0.0],
                    AxisLock::Y => delta = [0.0, delta[1], 0.0],
                    _ => {}
                }
                delta
            }
        };
        let follow_relief = self.axis != AxisLock::Z && !self.placement.snap_to_surface;
        Some(
            drag.start
                .iter()
                .map(|(id, start)| {
                    let mut transform = *start;
                    for axis in 0..3 {
                        let moved = start.position[axis] + delta[axis];
                        transform.position[axis] = if axis < 2 {
                            self.snap_to_grid(moved)
                        } else {
                            moved
                        };
                    }
                    if follow_relief {
                        // Moving along the ground keeps the height above it.
                        let [x, y, z] = transform.position;
                        transform.position[2] = z + self.ground_height(x, y)
                            - self.ground_height(start.position[0], start.position[1]);
                    }
                    (*id, transform)
                })
                .collect(),
        )
    }
}

/// Yaw of the selection around its center, following the angle of the pointer around the pivot.
fn dragged_rotation(
    drag: &Drag,
    ray: &Ray,
    step: f32,
) -> Option<Vec<(u64, crate::features::build::models::Transform)>> {
    let t = math::ray_plane(ray, drag.anchor, [0.0, 0.0, 1.0])?;
    let current = math::sub(ray.at(t), drag.pivot);
    let origin = math::sub(drag.anchor, drag.pivot);
    let mut degrees = (current[1].atan2(current[0]) - origin[1].atan2(origin[0])).to_degrees();
    if step > 0.0 {
        degrees = (degrees / f64::from(step)).round() * f64::from(step);
    }
    let (sin, cos) = degrees.to_radians().sin_cos();
    Some(
        drag.start
            .iter()
            .map(|(id, start)| {
                let mut transform = *start;
                let offset = math::sub(start.position, drag.pivot);
                transform.position[0] = drag.pivot[0] + offset[0] * cos - offset[1] * sin;
                transform.position[1] = drag.pivot[1] + offset[0] * sin + offset[1] * cos;
                transform.rotation[2] = (start.rotation[2] + degrees as f32).rem_euclid(360.0);
                (*id, transform)
            })
            .collect(),
    )
}

/// Scale by the ratio of the pointer's distance to the pivot now and at the start (one axis when locked).
fn dragged_scale(
    drag: &Drag,
    ray: &Ray,
    lock: AxisLock,
) -> Option<Vec<(u64, crate::features::build::models::Transform)>> {
    let t = math::ray_plane(ray, drag.anchor, [0.0, 0.0, 1.0])?;
    let now = math::sub(ray.at(t), drag.pivot);
    let start = math::sub(drag.anchor, drag.pivot);
    let length = |v: Vec3| (v[0] * v[0] + v[1] * v[1]).sqrt();
    let ratio = (length(now) / length(start).max(0.05)).clamp(0.05, 50.0) as f32;
    Some(
        drag.start
            .iter()
            .map(|(id, original)| {
                let mut transform = *original;
                for axis in 0..3 {
                    let wanted = match lock {
                        AxisLock::Free => true,
                        AxisLock::X => axis == 0,
                        AxisLock::Y => axis == 1,
                        AxisLock::Z => axis == 2,
                    };
                    if wanted {
                        transform.scale[axis] = (original.scale[axis] * ratio).max(0.01);
                    }
                }
                (*id, transform)
            })
            .collect(),
    )
}
