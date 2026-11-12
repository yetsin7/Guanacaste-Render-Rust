//! Reading the project's length and angle units.

use super::context::Context;
use crate::features::import_export::models::Severity;
use crate::features::import_export::services::step::{Entity, Value};

fn prefix_factor(prefix: &Value) -> f64 {
    match prefix.as_enum() {
        Some("MILLI") => 1e-3,
        Some("CENTI") => 1e-2,
        Some("DECI") => 1e-1,
        Some("KILO") => 1e3,
        Some("MICRO") => 1e-6,
        _ => 1.0,
    }
}

fn si_factor(entity: &Entity) -> f64 {
    prefix_factor(entity.arg(2))
}

fn conversion_factor(ctx: &Context, entity: &Entity) -> Option<f64> {
    let measure = ctx.file.follow(entity, 3)?;
    let value = measure.arg(0).as_f64()?;
    let base = measure
        .arg(1)
        .as_ref()
        .and_then(|id| ctx.entity(id))
        .map_or(1.0, si_factor);
    Some(value * base)
}

/// Text label of a length scale (meters per unit).
pub fn length_label(scale: f64) -> String {
    let known = [
        (1e-3, "mm"),
        (1e-2, "cm"),
        (1.0, "m"),
        (0.3048, "ft"),
        (0.0254, "in"),
        (1e3, "km"),
    ];
    known
        .iter()
        .find(|(factor, _)| (scale - factor).abs() < 1e-9)
        .map_or_else(|| format!("{scale} m"), |(_, label)| (*label).to_owned())
}

/// Reads `length` and `angle` from `IFCPROJECT.UnitsInContext`; warns when no length unit is declared.
pub fn read(ctx: &mut Context) {
    let project = ctx
        .file
        .entities
        .values()
        .find(|entity| entity.ty == "IFCPROJECT");
    let mut found_length = false;
    if let Some(assignment) = project.and_then(|project| ctx.file.follow(project, 8)) {
        for unit_id in assignment.arg(0).refs() {
            let Some(unit) = ctx.entity(unit_id) else {
                continue;
            };
            let kind = match unit.ty.as_str() {
                "IFCSIUNIT" => unit.arg(1).as_enum().map(|k| (k, si_factor(unit))),
                "IFCCONVERSIONBASEDUNIT" => unit
                    .arg(1)
                    .as_enum()
                    .and_then(|k| conversion_factor(ctx, unit).map(|f| (k, f))),
                _ => None,
            };
            match kind {
                Some(("LENGTHUNIT", factor)) => {
                    ctx.length = factor;
                    found_length = true;
                }
                Some(("PLANEANGLEUNIT", factor)) => ctx.angle = factor,
                _ => {}
            }
        }
    }
    if !found_length {
        ctx.note(Severity::Warning, "ifc.units_missing", &[]);
    }
}
