//! Profiles (cross sections) of swept solids, as 2D loops in meters.

use super::context::Context;
use crate::features::import_export::models::Severity;
use crate::features::import_export::services::step::Entity;

/// A closed outline with optional holes.
#[derive(Debug, Clone, PartialEq)]
pub struct Profile {
    /// Outer loop.
    pub outer: Vec<[f64; 2]>,
    /// Hole loops.
    pub holes: Vec<Vec<[f64; 2]>>,
}

fn rectangle(w: f64, h: f64) -> Vec<[f64; 2]> {
    vec![
        [-w / 2.0, -h / 2.0],
        [w / 2.0, -h / 2.0],
        [w / 2.0, h / 2.0],
        [-w / 2.0, h / 2.0],
    ]
}

fn circle(radius: f64) -> Vec<[f64; 2]> {
    (0..32)
        .map(|i| {
            let angle = std::f64::consts::TAU * i as f64 / 32.0;
            [radius * angle.cos(), radius * angle.sin()]
        })
        .collect()
}

fn place(points: &mut [[f64; 2]], location: [f64; 2], x_dir: [f64; 2]) {
    let length = x_dir[0].hypot(x_dir[1]).max(1e-12);
    let (cx, cy) = (x_dir[0] / length, x_dir[1] / length);
    for point in points {
        let (x, y) = (point[0], point[1]);
        *point = [location[0] + x * cx - y * cy, location[1] + x * cy + y * cx];
    }
}

fn dedupe_closing(mut points: Vec<[f64; 2]>) -> Vec<[f64; 2]> {
    if points.len() > 1 && points.first() == points.last() {
        points.pop();
    }
    points
}

/// Angle (radians, in the circle's local frame) of a trim value: a parameter or a point of the profile plane.
fn trim_angle(
    ctx: &Context,
    trims: &[crate::features::import_export::services::step::Value],
    center: [f64; 2],
    x_dir: [f64; 2],
) -> Option<f64> {
    use crate::features::import_export::services::step::Value;
    let length = x_dir[0].hypot(x_dir[1]).max(1e-12);
    let (cx, cy) = (x_dir[0] / length, x_dir[1] / length);
    for trim in trims {
        match trim {
            Value::Typed(name, args) if name == "IFCPARAMETERVALUE" => {
                return args.first().and_then(Value::as_f64).map(|v| v * ctx.angle)
            }
            Value::Ref(id) => {
                let point = ctx.point2(*id)?;
                let (dx, dy) = (point[0] - center[0], point[1] - center[1]);
                return Some(
                    (dx * -cy + dy * cx)
                        .atan2(dx * cx + dy * cy)
                        .rem_euclid(std::f64::consts::TAU),
                );
            }
            _ => {}
        }
    }
    None
}

/// Samples an `IFCTRIMMEDCURVE` over an `IFCCIRCLE` (the arcs Revit writes in rounded profiles).
fn arc_points(ctx: &mut Context, entity: &Entity) -> Option<Vec<[f64; 2]>> {
    let basis = ctx.file.follow(entity, 0)?;
    if basis.ty != "IFCCIRCLE" {
        return None;
    }
    let (center, x_dir) = ctx.axis2_of_placement(basis.arg(0).as_ref());
    let radius = basis.arg(1).as_f64()? * ctx.length;
    let start = trim_angle(ctx, entity.arg(1).items(), center, x_dir)?;
    let end = trim_angle(ctx, entity.arg(2).items(), center, x_dir)?;
    let forward = entity.arg(3).as_enum() != Some("F");
    let mut sweep = end - start;
    if forward && sweep <= 0.0 {
        sweep += std::f64::consts::TAU;
    }
    if !forward && sweep >= 0.0 {
        sweep -= std::f64::consts::TAU;
    }
    let steps = ((sweep.abs() / (std::f64::consts::PI / 18.0)).ceil() as usize).max(2);
    let length = x_dir[0].hypot(x_dir[1]).max(1e-12);
    let (cx, cy) = (x_dir[0] / length, x_dir[1] / length);
    Some(
        (0..=steps)
            .map(|i| {
                let a = start + sweep * i as f64 / steps as f64;
                let (x, y) = (radius * a.cos(), radius * a.sin());
                [center[0] + x * cx - y * cy, center[1] + x * cy + y * cx]
            })
            .collect(),
    )
}

/// The outline of a curve (polyline, indexed poly curve or composite curve made of them), in meters.
pub fn curve_points(ctx: &mut Context, id: u32) -> Option<Vec<[f64; 2]>> {
    let entity = ctx.entity(id)?;
    match entity.ty.as_str() {
        "IFCPOLYLINE" => {
            let points: Vec<_> = entity
                .arg(0)
                .refs()
                .into_iter()
                .filter_map(|p| ctx.point2(p))
                .collect();
            Some(dedupe_closing(points))
        }
        "IFCINDEXEDPOLYCURVE" => {
            let list = ctx.file.follow(entity, 0)?;
            let points: Vec<[f64; 2]> = list
                .arg(0)
                .items()
                .iter()
                .filter_map(|row| {
                    let c = row.items();
                    Some([
                        c.first()?.as_f64()? * ctx.length,
                        c.get(1)?.as_f64()? * ctx.length,
                    ])
                })
                .collect();
            if entity.arg(1).items().iter().any(|segment| matches!(segment, crate::features::import_export::services::step::Value::Typed(name, _) if name == "IFCARCINDEX")) {
                ctx.note(Severity::Warning, "ifc.arcs_as_chords", &[]);
            }
            Some(dedupe_closing(points))
        }
        "IFCCOMPOSITECURVE" => {
            let mut points = Vec::new();
            for segment in entity.arg(0).refs() {
                let segment = ctx.entity(segment)?;
                let mut part = segment
                    .arg(2)
                    .as_ref()
                    .and_then(|curve| curve_points(ctx, curve))
                    .or_else(|| {
                        ctx.note(Severity::Warning, "ifc.unsupported_curve", &[&segment.ty]);
                        None
                    })?;
                if segment.arg(1).as_enum() == Some("F") {
                    part.reverse();
                }
                for point in part {
                    if points.last() != Some(&point) {
                        points.push(point);
                    }
                }
            }
            Some(dedupe_closing(points))
        }
        "IFCTRIMMEDCURVE" => {
            let points = arc_points(ctx, entity);
            if points.is_none() {
                ctx.note(Severity::Warning, "ifc.unsupported_curve", &[&entity.ty]);
            }
            points
        }
        other => {
            ctx.note(Severity::Warning, "ifc.unsupported_curve", &[other]);
            None
        }
    }
}

fn parameterized(ctx: &Context, entity: &Entity) -> Option<Profile> {
    let number = |index: usize| entity.arg(index).as_f64().map(|v| v * ctx.length);
    let (mut outer, mut holes) = match entity.ty.as_str() {
        "IFCRECTANGLEPROFILEDEF" | "IFCROUNDEDRECTANGLEPROFILEDEF" => {
            (rectangle(number(3)?, number(4)?), vec![])
        }
        "IFCRECTANGLEHOLLOWPROFILEDEF" => {
            let (w, h, t) = (number(3)?, number(4)?, number(5)?);
            (rectangle(w, h), vec![rectangle(w - 2.0 * t, h - 2.0 * t)])
        }
        "IFCCIRCLEPROFILEDEF" => (circle(number(3)?), vec![]),
        "IFCCIRCLEHOLLOWPROFILEDEF" => {
            let (r, t) = (number(3)?, number(4)?);
            (circle(r), vec![circle(r - t)])
        }
        "IFCISHAPEPROFILEDEF" => {
            let (w, d, tw, tf) = (number(3)?, number(4)?, number(5)?, number(6)?);
            let (hw, hd, hweb) = (w / 2.0, d / 2.0, tw / 2.0);
            (
                vec![
                    [-hw, -hd],
                    [hw, -hd],
                    [hw, -hd + tf],
                    [hweb, -hd + tf],
                    [hweb, hd - tf],
                    [hw, hd - tf],
                    [hw, hd],
                    [-hw, hd],
                    [-hw, hd - tf],
                    [-hweb, hd - tf],
                    [-hweb, -hd + tf],
                    [-hw, -hd + tf],
                ],
                vec![],
            )
        }
        "IFCLSHAPEPROFILEDEF" => {
            let depth = number(3)?;
            let width = number(4).unwrap_or(depth);
            let t = number(5)?;
            (
                vec![
                    [0.0, 0.0],
                    [width, 0.0],
                    [width, t],
                    [t, t],
                    [t, depth],
                    [0.0, depth],
                ],
                vec![],
            )
        }
        _ => return None,
    };
    let (location, x_dir) = ctx.axis2(entity.arg(2).as_ref());
    place(&mut outer, location, x_dir);
    for hole in &mut holes {
        place(hole, location, x_dir);
    }
    Some(Profile { outer, holes })
}

/// Reads the profile `id`; unsupported profile types are reported and give `None`.
pub fn read(ctx: &mut Context, id: u32) -> Option<Profile> {
    let entity = ctx.entity(id)?;
    let profile = match entity.ty.as_str() {
        "IFCARBITRARYCLOSEDPROFILEDEF" => entity
            .arg(2)
            .as_ref()
            .and_then(|curve| curve_points(ctx, curve))
            .map(|outer| Profile {
                outer,
                holes: vec![],
            }),
        "IFCARBITRARYPROFILEDEFWITHVOIDS" => {
            let outer = entity
                .arg(2)
                .as_ref()
                .and_then(|curve| curve_points(ctx, curve))?;
            let holes = entity
                .arg(3)
                .refs()
                .into_iter()
                .filter_map(|curve| curve_points(ctx, curve))
                .collect();
            Some(Profile { outer, holes })
        }
        _ => parameterized(ctx, entity),
    };
    if profile.is_none() && !entity.ty.starts_with("IFCARBITRARY") {
        ctx.note(Severity::Warning, "ifc.unsupported_profile", &[&entity.ty]);
    }
    profile.filter(|profile| profile.outer.len() >= 3)
}
