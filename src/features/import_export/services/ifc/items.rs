//! Dispatch from a representation item to its geometry, including mapped items (Revit families) and booleans.

use super::context::Context;
use super::piece::Piece;
use super::{brep, solids};
use crate::features::import_export::logic::geometry::{Mat4, V3};
use crate::features::import_export::models::Severity;

fn operator(ctx: &Context, id: Option<u32>) -> Mat4 {
    let Some(entity) = id.and_then(|id| ctx.entity(id)) else {
        return Mat4::IDENTITY;
    };
    let axis = |index: usize| entity.arg(index).as_ref().and_then(|d| ctx.direction(d));
    let origin = entity
        .arg(2)
        .as_ref()
        .and_then(|p| ctx.point3(p))
        .unwrap_or([0.0; 3]);
    let scale = entity.arg(3).as_f64().unwrap_or(1.0);
    let x = crate::features::import_export::logic::geometry::normalize(
        axis(0).unwrap_or([1.0, 0.0, 0.0]),
    );
    let z = crate::features::import_export::logic::geometry::normalize(
        axis(4).unwrap_or([0.0, 0.0, 1.0]),
    );
    let y = crate::features::import_export::logic::geometry::cross(z, x);
    let s = |v: V3| crate::features::import_export::logic::geometry::scale(v, scale);
    Mat4::from_axes(origin, s(x), s(y), s(z))
}

/// Appends the geometry of item `id`, transformed by `matrix`, to `out`. Unsupported items are reported.
pub fn item(
    ctx: &mut Context,
    id: u32,
    matrix: &Mat4,
    inherited: Option<[f32; 4]>,
    depth: usize,
    out: &mut Vec<Piece>,
) {
    let Some(entity) = ctx.entity(id) else { return };
    if depth > 8 {
        return;
    }
    let color = ctx.item_colors.get(&id).copied().or(inherited);
    let ty = entity.ty.as_str();
    let piece = match ty {
        "IFCEXTRUDEDAREASOLID" => solids::extruded(ctx, id),
        "IFCREVOLVEDAREASOLID" => solids::revolved(ctx, id),
        "IFCSWEPTDISKSOLID" => solids::swept_disk(ctx, id),
        "IFCFACETEDBREP"
        | "IFCFACETEDBREPWITHVOIDS"
        | "IFCFACEBASEDSURFACEMODEL"
        | "IFCSHELLBASEDSURFACEMODEL" => brep::shells(ctx, id),
        "IFCTRIANGULATEDFACESET" | "IFCPOLYGONALFACESET" => brep::face_set(ctx, id),
        "IFCMAPPEDITEM" => {
            let Some(source) = ctx.file.follow(entity, 0) else {
                return;
            };
            let origin = ctx.axis3(source.arg(0).as_ref());
            let combined = matrix
                .mul(&operator(ctx, entity.arg(1).as_ref()))
                .mul(&origin.inverse_rigid());
            if let Some(rep) = ctx.file.follow(source, 1) {
                for child in rep.arg(3).refs() {
                    item(ctx, child, &combined, color, depth + 1, out);
                }
            }
            return;
        }
        "IFCBOOLEANRESULT" | "IFCBOOLEANCLIPPINGRESULT" => {
            ctx.note(Severity::Warning, "ifc.boolean_ignored", &[]);
            if let Some(first) = entity.arg(1).as_ref() {
                item(ctx, first, matrix, color, depth + 1, out);
            }
            return;
        }
        "IFCGEOMETRICSET"
        | "IFCGEOMETRICCURVESET"
        | "IFCBOUNDINGBOX"
        | "IFCPOLYLINE"
        | "IFCCARTESIANPOINT"
        | "IFCTEXTLITERALWITHEXTENT"
        | "IFCCIRCLE"
        | "IFCTRIMMEDCURVE"
        | "IFCCOMPOSITECURVE"
        | "IFCINDEXEDPOLYCURVE" => return,
        other => {
            ctx.note(Severity::Warning, "ifc.unsupported_item", &[other]);
            return;
        }
    };
    match piece {
        Some(mut piece) => {
            piece.color = color;
            piece.transform(matrix);
            out.push(piece);
        }
        None => ctx.note(Severity::Warning, "ifc.item_failed", &[ty]),
    }
}
