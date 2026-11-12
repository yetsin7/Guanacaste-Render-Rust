//! Boundary representations and face sets: faceted breps, surface models and tessellated face sets.

use super::context::Context;
use super::piece::Piece;
use crate::features::import_export::logic::geometry::V3;
use crate::features::import_export::logic::triangulate::triangulate_3d;
use crate::features::import_export::services::step::Value;

fn loop_points(ctx: &Context, bound_id: u32) -> Option<Vec<V3>> {
    let bound = ctx.entity(bound_id)?;
    let polygon = ctx.file.follow(bound, 0)?;
    if polygon.ty != "IFCPOLYLOOP" {
        return None;
    }
    let mut points: Vec<V3> = polygon
        .arg(0)
        .refs()
        .into_iter()
        .filter_map(|p| ctx.point3(p))
        .collect();
    if bound.arg(1).as_enum() == Some("F") {
        points.reverse();
    }
    Some(points)
}

fn add_face(ctx: &Context, face_id: u32, piece: &mut Piece) {
    let Some(face) = ctx.entity(face_id) else {
        return;
    };
    let bounds = face.arg(0).refs();
    let outer_index = bounds
        .iter()
        .position(|b| ctx.entity(*b).is_some_and(|e| e.ty == "IFCFACEOUTERBOUND"))
        .unwrap_or(0);
    let Some(outer) = bounds.get(outer_index).and_then(|b| loop_points(ctx, *b)) else {
        return;
    };
    if outer.len() < 3 {
        return;
    }
    let holes: Vec<Vec<V3>> = bounds
        .iter()
        .enumerate()
        .filter(|(i, _)| *i != outer_index)
        .filter_map(|(_, b)| loop_points(ctx, *b))
        .collect();
    let (points, triangles) = triangulate_3d(&outer, &holes);
    let base: Vec<u32> = points.iter().map(|p| piece.vertex(*p)).collect();
    for t in triangles {
        piece.triangle(base[t[0]], base[t[1]], base[t[2]]);
    }
}

fn add_shell(ctx: &Context, shell_id: u32, piece: &mut Piece) {
    if let Some(shell) = ctx.entity(shell_id) {
        for face in shell.arg(0).refs() {
            add_face(ctx, face, piece);
        }
    }
}

/// `IFCFACETEDBREP`, `IFCFACETEDBREPWITHVOIDS` (outer shell only), surface models and connected face sets.
pub fn shells(ctx: &Context, id: u32) -> Option<Piece> {
    let entity = ctx.entity(id)?;
    let mut piece = Piece::default();
    match entity.ty.as_str() {
        "IFCFACETEDBREP" | "IFCFACETEDBREPWITHVOIDS" => {
            add_shell(ctx, entity.arg(0).as_ref()?, &mut piece)
        }
        "IFCFACEBASEDSURFACEMODEL" | "IFCSHELLBASEDSURFACEMODEL" => {
            for shell in entity.arg(0).refs() {
                add_shell(ctx, shell, &mut piece);
            }
        }
        _ => return None,
    }
    (!piece.is_empty()).then_some(piece)
}

fn coordinates(ctx: &Context, id: u32) -> Option<Vec<V3>> {
    let list = ctx.entity(id)?;
    Some(
        list.arg(0)
            .items()
            .iter()
            .filter_map(|row| {
                let c = row.items();
                Some([
                    c.first()?.as_f64()? * ctx.length,
                    c.get(1)?.as_f64()? * ctx.length,
                    c.get(2).and_then(Value::as_f64).unwrap_or(0.0) * ctx.length,
                ])
            })
            .collect(),
    )
}

fn indices(row: &Value) -> Vec<usize> {
    row.items()
        .iter()
        .filter_map(|v| v.as_i64())
        .filter(|i| *i >= 1)
        .map(|i| (i - 1) as usize)
        .collect()
}

/// `IFCTRIANGULATEDFACESET` and `IFCPOLYGONALFACESET` (IFC4).
pub fn face_set(ctx: &Context, id: u32) -> Option<Piece> {
    let entity = ctx.entity(id)?;
    let points = coordinates(ctx, entity.arg(0).as_ref()?)?;
    let mut piece = Piece::default();
    let base: Vec<u32> = points.iter().map(|p| piece.vertex(*p)).collect();
    let valid = |i: usize| i < base.len();
    match entity.ty.as_str() {
        "IFCTRIANGULATEDFACESET" => {
            for row in entity.arg(3).items().iter() {
                let t = indices(row);
                if t.len() == 3 && t.iter().all(|i| valid(*i)) {
                    piece.triangle(base[t[0]], base[t[1]], base[t[2]]);
                }
            }
        }
        "IFCPOLYGONALFACESET" => {
            for face_id in entity.arg(2).refs() {
                let Some(face) = ctx.entity(face_id) else {
                    continue;
                };
                let outer = indices(face.arg(0));
                let holes: Vec<Vec<usize>> = if face.ty == "IFCINDEXEDPOLYGONALFACEWITHVOIDS" {
                    face.arg(1).items().iter().map(indices).collect()
                } else {
                    vec![]
                };
                if outer.len() < 3
                    || !outer
                        .iter()
                        .chain(holes.iter().flatten())
                        .all(|i| valid(*i))
                {
                    continue;
                }
                let outer_points: Vec<V3> = outer.iter().map(|i| points[*i]).collect();
                let hole_points: Vec<Vec<V3>> = holes
                    .iter()
                    .map(|h| h.iter().map(|i| points[*i]).collect())
                    .collect();
                let (flat, triangles) = triangulate_3d(&outer_points, &hole_points);
                let local: Vec<u32> = flat.iter().map(|p| piece.vertex(*p)).collect();
                for t in triangles {
                    piece.triangle(local[t[0]], local[t[1]], local[t[2]]);
                }
            }
        }
        _ => return None,
    }
    (!piece.is_empty()).then_some(piece)
}
