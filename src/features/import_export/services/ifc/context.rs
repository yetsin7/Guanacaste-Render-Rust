//! Shared state while reading an IFC model: the parsed file, unit scales, placements and style colors.

use crate::features::import_export::logic::geometry::{placement, Mat4, V3};
use crate::features::import_export::models::{ImportReport, Severity};
use crate::features::import_export::services::step::{Entity, StepFile, Value};
use std::collections::HashMap;

/// Reading context of one IFC file.
pub struct Context<'a> {
    /// The parsed file.
    pub file: &'a StepFile,
    /// Meters per file length unit.
    pub length: f64,
    /// Radians per file angle unit.
    pub angle: f64,
    /// Messages collected while importing.
    pub report: ImportReport,
    /// Style color of geometry items, by item id.
    pub item_colors: HashMap<u32, [f32; 4]>,
    /// Style color of materials, by material id.
    pub material_colors: HashMap<u32, [f32; 4]>,
    placements: HashMap<u32, Mat4>,
}

impl<'a> Context<'a> {
    /// Creates a context with default unit scales.
    pub fn new(file: &'a StepFile) -> Self {
        Self {
            file,
            length: 1.0,
            angle: 1.0,
            report: ImportReport::default(),
            item_colors: HashMap::new(),
            material_colors: HashMap::new(),
            placements: HashMap::new(),
        }
    }

    /// Records a report message.
    pub fn note(&mut self, severity: Severity, key: &'static str, args: &[&str]) {
        self.report.note(severity, key, args);
    }

    /// The entity with `id`.
    pub fn entity(&self, id: u32) -> Option<&'a Entity> {
        self.file.get(id)
    }

    /// A 3D point in meters.
    pub fn point3(&self, id: u32) -> Option<V3> {
        let entity = self.entity(id)?;
        if entity.ty != "IFCCARTESIANPOINT" {
            return None;
        }
        let c = entity.arg(0).items();
        Some([
            c.first()?.as_f64()? * self.length,
            c.get(1)?.as_f64()? * self.length,
            c.get(2).and_then(Value::as_f64).unwrap_or(0.0) * self.length,
        ])
    }

    /// A 2D point in meters.
    pub fn point2(&self, id: u32) -> Option<[f64; 2]> {
        let p = self.point3(id)?;
        Some([p[0], p[1]])
    }

    /// A direction (not scaled).
    pub fn direction(&self, id: u32) -> Option<V3> {
        let entity = self.entity(id)?;
        let c = entity.arg(0).items();
        Some([
            c.first()?.as_f64()?,
            c.get(1)?.as_f64()?,
            c.get(2).and_then(Value::as_f64).unwrap_or(0.0),
        ])
    }

    /// Transform of an `IFCAXIS2PLACEMENT3D` (identity when missing).
    pub fn axis3(&self, id: Option<u32>) -> Mat4 {
        let Some(entity) = id.and_then(|id| self.entity(id)) else {
            return Mat4::IDENTITY;
        };
        if entity.ty != "IFCAXIS2PLACEMENT3D" {
            return Mat4::IDENTITY;
        }
        let origin = entity
            .arg(0)
            .as_ref()
            .and_then(|id| self.point3(id))
            .unwrap_or([0.0; 3]);
        let axis = entity.arg(1).as_ref().and_then(|id| self.direction(id));
        let reference = entity.arg(2).as_ref().and_then(|id| self.direction(id));
        placement(origin, axis, reference)
    }

    /// Location and X direction of an `IFCAXIS2PLACEMENT2D`.
    pub fn axis2(&self, id: Option<u32>) -> ([f64; 2], [f64; 2]) {
        let default = ([0.0, 0.0], [1.0, 0.0]);
        let Some(entity) = id.and_then(|id| self.entity(id)) else {
            return default;
        };
        let location = entity
            .arg(0)
            .as_ref()
            .and_then(|id| self.point2(id))
            .unwrap_or(default.0);
        let direction = entity
            .arg(1)
            .as_ref()
            .and_then(|id| self.direction(id))
            .map_or(default.1, |d| [d[0], d[1]]);
        (location, direction)
    }

    /// Center and X direction of a 2D placement given as `IFCAXIS2PLACEMENT2D` (or the XY part of a 3D one).
    pub fn axis2_of_placement(&self, id: Option<u32>) -> ([f64; 2], [f64; 2]) {
        match id.and_then(|id| self.entity(id)) {
            Some(entity) if entity.ty == "IFCAXIS2PLACEMENT3D" => {
                let origin = entity
                    .arg(0)
                    .as_ref()
                    .and_then(|p| self.point3(p))
                    .unwrap_or([0.0; 3]);
                let direction = entity
                    .arg(2)
                    .as_ref()
                    .and_then(|d| self.direction(d))
                    .unwrap_or([1.0, 0.0, 0.0]);
                ([origin[0], origin[1]], [direction[0], direction[1]])
            }
            _ => self.axis2(id),
        }
    }

    /// World transform of an `IFCLOCALPLACEMENT` (composed with its parents).
    pub fn local_placement(&mut self, id: u32, depth: usize) -> Mat4 {
        if let Some(cached) = self.placements.get(&id) {
            return *cached;
        }
        let Some(entity) = self.entity(id) else {
            return Mat4::IDENTITY;
        };
        if entity.ty != "IFCLOCALPLACEMENT" || depth > 64 {
            return Mat4::IDENTITY;
        }
        let parent = entity.arg(0).as_ref().map_or(Mat4::IDENTITY, |parent| {
            self.local_placement(parent, depth + 1)
        });
        let matrix = parent.mul(&self.axis3(entity.arg(1).as_ref()));
        self.placements.insert(id, matrix);
        matrix
    }
}
