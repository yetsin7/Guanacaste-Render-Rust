//! Swept solids: extrusions, revolutions and swept disks.

use super::context::Context;
use super::piece::Piece;
use super::profile;
use crate::features::import_export::logic::geometry::{add, cross, dot, normalize, scale, sub, V3};
use crate::features::import_export::logic::triangulate::{signed_area, triangulate};
use crate::features::import_export::models::Severity;

fn oriented(points: &[[f64; 2]], counter_clockwise: bool) -> Vec<[f64; 2]> {
    let mut out = points.to_vec();
    if (signed_area(&out) > 0.0) != counter_clockwise {
        out.reverse();
    }
    out
}

/// `IFCEXTRUDEDAREASOLID`: the profile swept along a direction.
pub fn extruded(ctx: &mut Context, id: u32) -> Option<Piece> {
    let entity = ctx.entity(id)?;
    let profile = profile::read(ctx, entity.arg(0).as_ref()?)?;
    let position = ctx.axis3(entity.arg(1).as_ref());
    let direction = normalize(
        entity
            .arg(2)
            .as_ref()
            .and_then(|d| ctx.direction(d))
            .unwrap_or([0.0, 0.0, 1.0]),
    );
    let depth = entity.arg(3).as_f64()? * ctx.length;
    let outer = oriented(&profile.outer, true);
    let holes: Vec<Vec<[f64; 2]>> = profile
        .holes
        .iter()
        .map(|hole| oriented(hole, false))
        .collect();
    let mut piece = Piece::default();
    let offset = scale(direction, depth);
    let loops: Vec<&Vec<[f64; 2]>> = std::iter::once(&outer).chain(holes.iter()).collect();
    for ring in &loops {
        let bottom: Vec<u32> = ring
            .iter()
            .map(|p| piece.vertex([p[0], p[1], 0.0]))
            .collect();
        let top: Vec<u32> = ring
            .iter()
            .map(|p| piece.vertex(add([p[0], p[1], 0.0], offset)))
            .collect();
        for i in 0..ring.len() {
            let j = (i + 1) % ring.len();
            piece.quad(bottom[i], bottom[j], top[j], top[i]);
        }
    }
    let flat: Vec<[f64; 2]> = loops.iter().flat_map(|ring| ring.iter().copied()).collect();
    let caps = triangulate(&outer, &holes);
    let first_vertex = piece.positions.len() as u32;
    for p in &flat {
        piece.vertex([p[0], p[1], 0.0]);
    }
    for p in &flat {
        piece.vertex(add([p[0], p[1], 0.0], offset));
    }
    let count = flat.len() as u32;
    for t in &caps {
        let [a, b, c] = [t[0] as u32, t[1] as u32, t[2] as u32];
        piece.triangle(first_vertex + a, first_vertex + c, first_vertex + b);
        piece.triangle(
            first_vertex + count + a,
            first_vertex + count + b,
            first_vertex + count + c,
        );
    }
    if direction[2] < 0.0 {
        for tri in piece.indices.chunks_mut(3) {
            tri.swap(1, 2);
        }
    }
    piece.transform(&position);
    Some(piece)
}

fn rotate(point: V3, origin: V3, axis: V3, angle: f64) -> V3 {
    let v = sub(point, origin);
    let (sin, cos) = angle.sin_cos();
    let rotated = add(
        add(scale(v, cos), scale(cross(axis, v), sin)),
        scale(axis, dot(axis, v) * (1.0 - cos)),
    );
    add(rotated, origin)
}

/// `IFCREVOLVEDAREASOLID`: the profile rotated around an axis.
pub fn revolved(ctx: &mut Context, id: u32) -> Option<Piece> {
    let entity = ctx.entity(id)?;
    let profile = profile::read(ctx, entity.arg(0).as_ref()?)?;
    let position = ctx.axis3(entity.arg(1).as_ref());
    let axis_entity = ctx.file.follow(entity, 2)?;
    let origin = axis_entity
        .arg(0)
        .as_ref()
        .and_then(|p| ctx.point3(p))
        .unwrap_or([0.0; 3]);
    let axis = normalize(
        axis_entity
            .arg(1)
            .as_ref()
            .and_then(|d| ctx.direction(d))
            .unwrap_or([0.0, 0.0, 1.0]),
    );
    let angle =
        (entity.arg(3).as_f64()? * ctx.angle).clamp(-std::f64::consts::TAU, std::f64::consts::TAU);
    if !profile.holes.is_empty() {
        ctx.note(Severity::Warning, "ifc.revolve_holes", &[]);
    }
    let outer = oriented(&profile.outer, true);
    let steps = ((angle.abs() / (std::f64::consts::PI / 12.0)).ceil() as usize).max(4);
    let full = (angle.abs() - std::f64::consts::TAU).abs() < 1e-6;
    let mut piece = Piece::default();
    let rings = if full { steps } else { steps + 1 };
    let mut ids: Vec<Vec<u32>> = Vec::new();
    for step in 0..rings {
        let a = angle * step as f64 / steps as f64;
        ids.push(
            outer
                .iter()
                .map(|p| piece.vertex(rotate([p[0], p[1], 0.0], origin, axis, a)))
                .collect(),
        );
    }
    for step in 0..steps {
        let (current, next) = (&ids[step], &ids[(step + 1) % rings]);
        for i in 0..outer.len() {
            let j = (i + 1) % outer.len();
            piece.quad(current[i], current[j], next[j], next[i]);
        }
    }
    if !full {
        for (ring, reverse) in [(&ids[0], true), (&ids[rings - 1], false)] {
            for t in triangulate(&outer, &[]) {
                let (a, b, c) = (ring[t[0]], ring[t[1]], ring[t[2]]);
                if reverse {
                    piece.triangle(a, c, b);
                } else {
                    piece.triangle(a, b, c);
                }
            }
        }
    }
    if angle < 0.0 {
        for tri in piece.indices.chunks_mut(3) {
            tri.swap(1, 2);
        }
    }
    piece.transform(&position);
    Some(piece)
}

fn directrix_points(ctx: &mut Context, id: u32) -> Option<Vec<V3>> {
    let entity = ctx.entity(id)?;
    match entity.ty.as_str() {
        "IFCPOLYLINE" => Some(
            entity
                .arg(0)
                .refs()
                .into_iter()
                .filter_map(|p| ctx.point3(p))
                .collect(),
        ),
        "IFCINDEXEDPOLYCURVE" => {
            let list = ctx.file.follow(entity, 0)?;
            Some(
                list.arg(0)
                    .items()
                    .iter()
                    .filter_map(|row| {
                        let c = row.items();
                        Some([
                            c.first()?.as_f64()? * ctx.length,
                            c.get(1)?.as_f64()? * ctx.length,
                            c.get(2).and_then(|v| v.as_f64()).unwrap_or(0.0) * ctx.length,
                        ])
                    })
                    .collect(),
            )
        }
        other => {
            ctx.note(Severity::Warning, "ifc.unsupported_curve", &[other]);
            None
        }
    }
}

/// `IFCSWEPTDISKSOLID`: a tube along a polyline.
pub fn swept_disk(ctx: &mut Context, id: u32) -> Option<Piece> {
    let entity = ctx.entity(id)?;
    let path = directrix_points(ctx, entity.arg(0).as_ref()?)?;
    let radius = entity.arg(1).as_f64()? * ctx.length;
    if path.len() < 2 || radius <= 0.0 {
        return None;
    }
    let sides = 12;
    let mut piece = Piece::default();
    let mut rings: Vec<Vec<u32>> = Vec::new();
    for (index, point) in path.iter().enumerate() {
        let before = path[index.saturating_sub(1)];
        let after = path[(index + 1).min(path.len() - 1)];
        let tangent = normalize(sub(after, before));
        let hint = if tangent[2].abs() < 0.9 {
            [0.0, 0.0, 1.0]
        } else {
            [1.0, 0.0, 0.0]
        };
        let u = normalize(cross(tangent, hint));
        let v = cross(tangent, u);
        rings.push(
            (0..sides)
                .map(|s| {
                    let a = std::f64::consts::TAU * s as f64 / sides as f64;
                    piece.vertex(add(
                        *point,
                        add(scale(u, radius * a.cos()), scale(v, radius * a.sin())),
                    ))
                })
                .collect(),
        );
    }
    for pair in rings.windows(2) {
        for s in 0..sides {
            let t = (s + 1) % sides;
            piece.quad(pair[0][s], pair[1][s], pair[1][t], pair[0][t]);
        }
    }
    for (ring, reverse) in [(&rings[0], true), (&rings[rings.len() - 1], false)] {
        for s in 1..sides - 1 {
            if reverse {
                piece.triangle(ring[0], ring[s + 1], ring[s]);
            } else {
                piece.triangle(ring[0], ring[s], ring[s + 1]);
            }
        }
    }
    Some(piece)
}
