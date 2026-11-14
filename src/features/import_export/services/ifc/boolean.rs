//! Boolean results: differences with solids, half-spaces and polygonal bounded half-spaces.

use super::context::Context;
use super::csg::{subtract_all, Convex, Plane};
use super::items;
use super::piece::Piece;
use crate::features::import_export::logic::geometry::{cross, normalize, sub, Mat4, V3};
use crate::features::import_export::models::Severity;
use crate::features::import_export::services::step::Entity;

fn polygon_points(ctx: &Context, id: u32) -> Vec<[f64; 2]> {
    let Some(curve) = ctx.entity(id) else {
        return Vec::new();
    };
    match curve.ty.as_str() {
        "IFCPOLYLINE" => curve
            .arg(0)
            .refs()
            .into_iter()
            .filter_map(|p| ctx.point2(p))
            .collect(),
        "IFCINDEXEDPOLYCURVE" => ctx
            .file
            .follow(curve, 0)
            .map(|list| {
                list.arg(0)
                    .items()
                    .iter()
                    .filter_map(|row| {
                        let c = row.items();
                        Some([
                            c.first()?.as_f64()? * ctx.length,
                            c.get(1)?.as_f64()? * ctx.length,
                        ])
                    })
                    .collect()
            })
            .unwrap_or_default(),
        _ => Vec::new(),
    }
}

/// The region removed by a half-space (optionally bounded by a convex polygon prism), in world space.
fn half_space(ctx: &Context, entity: &Entity, matrix: &Mat4) -> Option<Convex> {
    let surface = ctx.file.follow(entity, 0)?;
    if surface.ty != "IFCPLANE" {
        return None;
    }
    let local = ctx.axis3(surface.arg(0).as_ref());
    let point = matrix.point(local.point([0.0; 3]));
    let normal = normalize(matrix.direction(local.direction([0.0, 0.0, 1.0])));
    // `.T.` means the normal points away from the material, so the removed region is the negative side.
    let outward = if entity.arg(1).as_enum() == Some("T") {
        normal
    } else {
        [-normal[0], -normal[1], -normal[2]]
    };
    let mut planes = vec![Plane::new(point, outward)];
    if entity.ty == "IFCPOLYGONALBOUNDEDHALFSPACE" {
        let frame = matrix.mul(&ctx.axis3(entity.arg(2).as_ref()));
        let mut ring = polygon_points(ctx, entity.arg(3).as_ref()?);
        if ring.len() > 1 && ring.first() == ring.last() {
            ring.pop();
        }
        if ring.len() < 3 {
            return None;
        }
        let area: f64 = (0..ring.len())
            .map(|i| {
                let j = (i + 1) % ring.len();
                ring[i][0] * ring[j][1] - ring[j][0] * ring[i][1]
            })
            .sum();
        let z = frame.direction([0.0, 0.0, 1.0]);
        let mut sign = 0.0;
        for i in 0..ring.len() {
            let (a, b, c) = (
                ring[i],
                ring[(i + 1) % ring.len()],
                ring[(i + 2) % ring.len()],
            );
            let turn = (b[0] - a[0]) * (c[1] - b[1]) - (b[1] - a[1]) * (c[0] - b[0]);
            if turn.abs() > 1e-9 {
                if sign != 0.0 && turn * sign < 0.0 {
                    return None;
                }
                sign = turn;
            }
        }
        for i in 0..ring.len() {
            let a = frame.point([ring[i][0], ring[i][1], 0.0]);
            let b = frame.point([
                ring[(i + 1) % ring.len()][0],
                ring[(i + 1) % ring.len()][1],
                0.0,
            ]);
            let edge = sub(b, a);
            let mut out: V3 = cross(edge, z);
            if area < 0.0 {
                out = [-out[0], -out[1], -out[2]];
            }
            planes.push(Plane::new(a, out));
        }
    }
    Some(Convex::from_planes(planes, None))
}

fn cutters(ctx: &mut Context, second: u32, matrix: &Mat4, depth: usize) -> Option<Vec<Convex>> {
    let entity = ctx.entity(second)?;
    if matches!(
        entity.ty.as_str(),
        "IFCHALFSPACESOLID" | "IFCPOLYGONALBOUNDEDHALFSPACE"
    ) {
        return half_space(ctx, entity, matrix).map(|c| vec![c]);
    }
    let mut pieces: Vec<Piece> = Vec::new();
    items::item(ctx, second, matrix, None, depth + 1, &mut pieces);
    let mut out = Vec::new();
    for piece in &pieces {
        out.push(Convex::from_piece(piece)?);
    }
    (!out.is_empty()).then_some(out)
}

/// Resolves an `IFCBOOLEANRESULT` into `out`.
pub fn apply(
    ctx: &mut Context,
    entity: &Entity,
    matrix: &Mat4,
    color: Option<[f32; 4]>,
    depth: usize,
    out: &mut Vec<Piece>,
) {
    let mut first: Vec<Piece> = Vec::new();
    if let Some(id) = entity.arg(1).as_ref() {
        items::item(ctx, id, matrix, color, depth + 1, &mut first);
    }
    let Some(second) = entity.arg(2).as_ref() else {
        out.extend(first);
        return;
    };
    match entity.arg(0).as_enum() {
        Some("UNION") => {
            out.extend(first);
            items::item(ctx, second, matrix, color, depth + 1, out);
        }
        Some("DIFFERENCE") => match cutters(ctx, second, matrix, depth) {
            Some(cutters) => {
                for piece in first {
                    out.push(subtract_all(piece, &cutters));
                }
            }
            None => {
                ctx.note(Severity::Warning, "ifc.boolean_ignored", &[]);
                out.extend(first);
            }
        },
        _ => {
            ctx.note(Severity::Warning, "ifc.boolean_ignored", &[]);
            out.extend(first);
        }
    }
}
