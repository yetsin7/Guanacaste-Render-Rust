//! Floors, ceilings, roofs, water basins and columns: slabs with the stair and shaft openings cut through them,
//! pitched roofs with overhang, and columns of every plan shape.

use super::context::Context;
use super::json::{flag, int, list, number, point, points, text};
use super::part::Part;
use super::roofs::{grow, Roof, Shape};
use super::surface::Surface;
use crate::features::import_export::logic::prism::{band, rectangle};
use crate::features::import_export::models::Severity;
use serde_json::Value;

/// Most points of one slab or roof outline (the ear-clipping triangulation is quadratic).
const MAX_OUTLINE_POINTS: usize = 5_000;

fn finish_surface(finish: i64) -> Surface {
    match finish {
        4 => Surface::WallConcrete,
        1 => Surface::Wall,
        5 => Surface::Asphalt,
        6 => Surface::Ground,
        _ => Surface::Floor,
    }
}

/// Whether `p` is inside the polygon (even-odd rule).
fn inside(polygon: &[[f64; 2]], p: [f64; 2]) -> bool {
    let mut odd = false;
    for i in 0..polygon.len() {
        let (a, b) = (polygon[i], polygon[(i + 1) % polygon.len()]);
        if (a[1] > p[1]) != (b[1] > p[1])
            && p[0] < (b[0] - a[0]) * (p[1] - a[1]) / (b[1] - a[1]) + a[0]
        {
            odd = !odd;
        }
    }
    odd
}

/// The footprint of a structural element: a band along its axis, or a rectangle for a point element.
pub fn element_footprint(element: &Value) -> Vec<[f64; 2]> {
    let (start, end) = (point(element, "Start"), point(element, "End"));
    let width = number(element, "Width");
    let band = band(start, end, width);
    if !band.is_empty() {
        return band;
    }
    let depth = number(element, "Depth");
    rectangle(start, width, if depth > 0.0 { depth } else { width })
}

/// The plan rectangles of the openings and shafts that cut a slab of `level` at `elevation`.
fn cut_outs(
    ctx: &Context,
    document: &Value,
    floor: &Value,
    outline: &[[f64; 2]],
) -> Vec<Vec<[f64; 2]>> {
    let kind = int(floor, "Kind");
    let level = text(floor, "LevelId");
    let floor_z = ctx.elevation(floor);
    list(document, "StructuralElements")
        .iter()
        .filter(|e| {
            let k = int(e, "Kind");
            (k == 8 && kind == 0 && text(e, "LevelId") == level)
                || (k == 9 && ctx.elevation(e) <= floor_z)
        })
        .map(element_footprint)
        .filter_map(|footprint| {
            let (mut min, mut max) = ([f64::MAX; 2], [f64::MIN; 2]);
            for p in &footprint {
                for axis in 0..2 {
                    min[axis] = min[axis].min(p[axis]);
                    max[axis] = max[axis].max(p[axis]);
                }
            }
            let rect = vec![min, [max[0], min[1]], max, [min[0], max[1]]];
            (rect.iter().all(|c| inside(outline, *c))
                && (max[0] - min[0]) * (max[1] - min[1]) > 1e-6)
                .then_some(rect)
        })
        .collect()
}

/// Builds every floor, ceiling, roof and water slab.
pub fn build_floors(ctx: &mut Context, document: &Value) {
    for floor in list(document, "Floors") {
        let boundary = points(floor, "Boundary");
        let outline = if boundary.len() > MAX_OUTLINE_POINTS {
            Vec::new()
        } else {
            boundary
        };
        let kind = int(floor, "Kind");
        let roof = kind == 2;
        let thickness = number(floor, "Thickness").max(0.01);
        let overhang = if roof {
            number(floor, "Overhang").max(0.0)
        } else {
            0.0
        };
        let shape =
            Shape::from_value(int(floor, "Shape")).filter(|_| roof && number(floor, "Pitch") > 0.0);
        let base = ctx.elevation(floor);
        let offset = number(floor, "ElevationOffset");
        let mut part = Part::new();
        if kind == 4 {
            ctx.report
                .note(Severity::Warning, "legacy.toposolid_flat", &[]);
        }
        if let (Some(shape), true) = (shape, outline.len() >= 3) {
            let pitch = number(floor, "Pitch").clamp(0.0, 85.0);
            let built = if overhang > 0.0 {
                grow(&outline, overhang)
            } else {
                outline.clone()
            };
            let eave = base + offset - overhang * pitch.to_radians().tan();
            let roof = Roof {
                outline: &built,
                shape,
                pitch,
                thickness,
                eave,
            };
            roof.build(&mut part, flag(floor, "GableWalls", true));
        } else {
            let top = base + offset;
            let outline = if overhang > 0.0 && outline.len() >= 3 {
                grow(&outline, overhang)
            } else {
                outline
            };
            let holes = cut_outs(ctx, document, floor, &outline);
            let surface = match kind {
                3 => Surface::Water,
                4 => Surface::Ground,
                _ => finish_surface(int(floor, "Finish")),
            };
            part.prism(&outline, &holes, top - thickness, top, surface, surface);
        }
        ctx.element(floor, if roof { "Roof" } else { "Floor" }, None, part);
    }
}

/// The plan outline of a column of the given `shape` (rectangular, round, L, T or octagonal).
fn column_outline(shape: i64, center: [f64; 2], width: f64, depth: f64) -> Vec<[f64; 2]> {
    let (hw, hd) = (width / 2.0, depth / 2.0);
    match shape {
        1 | 4 => {
            let sides = if shape == 1 { 16 } else { 8 };
            (0..sides)
                .map(|i| {
                    let a = std::f64::consts::TAU * f64::from(i) / f64::from(sides);
                    [center[0] + hw * a.cos(), center[1] + hd * a.sin()]
                })
                .collect()
        }
        _ => rectangle(center, width, depth),
    }
}

/// Builds the columns.
pub fn build_columns(ctx: &mut Context, document: &Value) {
    for column in list(document, "Columns") {
        let base = ctx.elevation(column);
        let shape = int(column, "Shape");
        if shape == 2 || shape == 3 {
            ctx.report.note(Severity::Info, "legacy.column_box", &[]);
        }
        let outline = column_outline(
            shape,
            point(column, "Position"),
            number(column, "Width"),
            number(column, "Depth"),
        );
        let mut part = Part::new();
        part.convex_prism(
            &outline,
            base,
            base + number(column, "Height"),
            Surface::Structure,
            Surface::Structure,
        );
        ctx.element(column, "Column", None, part);
    }
}
