//! Placed objects: plants and components, from their embedded family mesh or from generated models.

use super::axis::Placement;
use super::components::{self, Size};
use super::context::Context;
use super::families::Families;
use super::json::{int, number, point, text};
use super::part::Part;
use super::surface::Surface;
use crate::features::import_export::logic::prism::rectangle;
use crate::features::import_export::models::Severity;
use crate::features::library::logic::procedural;
use crate::features::library::models::ProceduralKind;
use crate::features::materials::models::Material as Pbr;
use serde_json::Value;

/// Adds `scene`, a generated model resting on the origin, scaled to `size` and placed at `at`.
fn add_procedural(
    ctx: &mut Context,
    part: &mut Part,
    kind: ProceduralKind,
    seed: u64,
    at: &Placement,
    size: [f64; 3],
) {
    let scene = procedural::generate(kind, seed);
    let Some(b) = scene.bounds() else {
        return;
    };
    let scale = [
        size[0] / f64::from((b.max[0] - b.min[0]).max(1e-3)),
        size[1] / f64::from((b.max[1] - b.min[1]).max(1e-3)),
        size[2] / f64::from((b.max[2] - b.min[2]).max(1e-3)),
    ];
    for mesh in &scene.meshes {
        let Some(source) = mesh.material.and_then(|m| scene.materials.get(m)) else {
            continue;
        };
        let name = format!("{}:{:?}", source.name, kind);
        let color = source.color;
        let index = ctx.book.named(&name, || {
            let mut m = Pbr::from_color(source.name.clone(), color);
            m.roughness = 0.7;
            m
        });
        let positions: Vec<[f32; 3]> = mesh
            .positions
            .iter()
            .map(|p| {
                let local = at.plan(
                    f64::from(p[0] - (b.min[0] + b.max[0]) / 2.0) * scale[0],
                    f64::from(p[1] - (b.min[1] + b.max[1]) / 2.0) * scale[1],
                );
                [
                    local[0] as f32,
                    local[1] as f32,
                    (at.z + f64::from(p[2] - b.min[2]) * scale[2]) as f32,
                ]
            })
            .collect();
        let uvs = vec![[0.0, 0.0]; positions.len()];
        part.indexed(
            super::part::Key::Custom(index),
            &positions,
            &uvs,
            &mesh.indices,
        );
    }
}

/// A plant: its detailed family when the document has one, otherwise a generated tree, palm, conifer or shrub.
pub fn planting<'a>(
    ctx: &mut Context,
    part: &mut Part,
    element: &'a Value,
    datum: f64,
    families: &mut Families<'a>,
) {
    let at = point(element, "Start");
    let (width, height) = (number(element, "Width"), number(element, "Height"));
    let z = datum + number(element, "Offset");
    if let Some(id) = element.get("FamilyId").and_then(Value::as_str) {
        if let Some(found) = families.place(
            ctx,
            id,
            at,
            number(element, "Rotation"),
            [width, width, height],
            true,
            z - 0.08,
        ) {
            part.absorb(found);
            return;
        }
        ctx.report.note(
            Severity::Warning,
            "legacy.family_missing",
            &[text(element, "Name")],
        );
    }
    let kind = match int(element, "Style") {
        10 => ProceduralKind::Palm,
        11 => ProceduralKind::Conifer,
        12 => ProceduralKind::Bush,
        _ => ProceduralKind::Tree,
    };
    // A plant without a family borrows a detailed one of its kind from the same project (scanned models with
    // real leaves) instead of the low-detail generated shape, which reads as a green blob in a render.
    if let Some(id) = families.stand_in(kind) {
        if let Some(found) = families.place(
            ctx,
            id,
            at,
            number(element, "Rotation"),
            [width, width, height],
            true,
            z - 0.08,
        ) {
            part.absorb(found);
            return;
        }
    }
    let placement = Placement {
        at,
        z,
        rotation: number(element, "Rotation"),
    };
    add_procedural(ctx, part, kind, 3, &placement, [width, width, height]);
}

/// A component: the family mesh stretched to its box, or a composition of boxes for its residential shape.
pub fn component<'a>(
    ctx: &mut Context,
    part: &mut Part,
    element: &'a Value,
    datum: f64,
    families: &mut Families<'a>,
) {
    let at = point(element, "Start");
    let width = number(element, "Width");
    let depth = if number(element, "Depth") > 0.0 {
        number(element, "Depth")
    } else {
        width
    };
    let height = number(element, "Height");
    let z = datum + number(element, "Offset");
    let rotation = number(element, "Rotation");
    if let Some(id) = element.get("FamilyId").and_then(Value::as_str) {
        if let Some(found) = families.place(ctx, id, at, rotation, [width, depth, height], false, z)
        {
            part.absorb(found);
            return;
        }
        ctx.report.note(
            Severity::Warning,
            "legacy.family_missing",
            &[text(element, "Name")],
        );
    }
    let placement = Placement { at, z, rotation };
    let shape = int(element, "ComponentShape");
    if shape == 0 {
        let outline = rectangle([0.0, 0.0], width, depth);
        let turned: Vec<[f64; 2]> = outline.iter().map(|p| placement.plan(p[0], p[1])).collect();
        part.convex_prism(
            &turned,
            z,
            z + height,
            Surface::Structure,
            Surface::Structure,
        );
    } else {
        components::build(
            part,
            &placement,
            shape,
            &Size {
                w: width,
                d: depth,
                h: height,
            },
        );
    }
}
