//! Structural and architectural elements: stairs, guards, curtain walls and cobogó screens, ramps, gutters, beams,
//! foundations, components (furniture, cars) and plants with their detailed families.

use super::axis::{round, Axis};
use super::circulation::{curtain_wall, railing, span, stair, BREEZE_BLOCK};
use super::context::Context;
use super::families::Families;
use super::json::{int, list, number, point};
use super::part::Part;
use super::placed::{component, planting};
use super::slabs::element_footprint;
use super::surface::Surface::*;
use serde_json::Value;

/// Kinds that are drafting marks, voids or annotations: no solid.
fn has_solid(kind: i64) -> bool {
    !matches!(kind, 7..=10 | 19..=22 | 30..=33 | 35..=43)
}

/// Builds every structural element of the document.
pub fn build<'a>(ctx: &mut Context, document: &'a Value, families: &mut Families<'a>) {
    for element in list(document, "StructuralElements") {
        let kind = int(element, "Kind");
        if number(element, "Height") <= 0.0 || !has_solid(kind) {
            *ctx.report
                .skipped_types
                .entry("StructuralElement (sin altura)".to_owned())
                .or_insert(0) += 1;
            continue;
        }
        let datum = ctx.elevation(element);
        let mut part = Part::new();
        let class = match kind {
            5 => {
                component(ctx, &mut part, element, datum, families);
                "Component"
            }
            28 => {
                planting(ctx, &mut part, element, datum, families);
                "Planting"
            }
            12 => {
                stair(ctx, &mut part, element, datum);
                "Stair"
            }
            13 => {
                railing(&mut part, element, datum);
                "Railing"
            }
            17 => {
                curtain_wall(&mut part, element, datum);
                if int(element, "Style") == BREEZE_BLOCK {
                    "BreezeBlock"
                } else {
                    "CurtainWall"
                }
            }
            _ => {
                generic(&mut part, element, kind, datum);
                "StructuralElement"
            }
        };
        ctx.element(element, class, None, part);
    }
}

/// Beams, braces, foundations, ramps, gutters, sweeps and anything without a dedicated shape.
fn generic(part: &mut Part, element: &Value, kind: i64, datum: f64) {
    let (bottom, top) = span(element, datum);
    let (start, end) = (point(element, "Start"), point(element, "End"));
    let (width, style) = (number(element, "Width"), int(element, "Style"));
    let axis = Axis::new(start, end);
    match (kind, axis) {
        (14, Some(axis)) => {
            let toe = 0.08f64.min((top - bottom) / 2.0);
            axis.wedge(
                part,
                Structure,
                width,
                [bottom, bottom + toe],
                [bottom, top],
            );
        }
        (15, Some(axis)) => {
            let sheet = 0.008;
            axis.block(
                part,
                Metal,
                0.0,
                axis.length,
                -width / 2.0,
                width / 2.0,
                bottom,
                bottom + sheet,
            );
            for side in [-1.0, 1.0] {
                axis.block(
                    part,
                    Metal,
                    0.0,
                    axis.length,
                    side * width / 2.0 - sheet / 2.0,
                    side * width / 2.0 + sheet / 2.0,
                    bottom,
                    top,
                );
            }
        }
        (23 | 24, Some(axis)) => {
            let proud = if kind == 24 { 0.004 } else { 0.025 };
            let surface = if kind == 24 { OpeningFrame } else { Wall };
            for side in [-1.0, 1.0] {
                let c = side * (width / 2.0 + proud / 2.0);
                axis.block(
                    part,
                    surface,
                    0.0,
                    axis.length,
                    c - proud / 2.0,
                    c + proud / 2.0,
                    bottom,
                    top,
                );
            }
        }
        (0..=2, Some(axis)) if style == 2 || style == 3 => {
            let flange = ((top - bottom) * 0.12).clamp(0.01, (top - bottom) / 3.0);
            let web = (width * 0.12).max(0.01);
            if style == 2 {
                axis.block(
                    part,
                    Structure,
                    0.0,
                    axis.length,
                    -width / 2.0,
                    width / 2.0,
                    bottom,
                    bottom + flange,
                );
            }
            axis.block(
                part,
                Structure,
                0.0,
                axis.length,
                -web / 2.0,
                web / 2.0,
                bottom + if style == 2 { flange } else { 0.0 },
                top - flange,
            );
            axis.block(
                part,
                Structure,
                0.0,
                axis.length,
                -width / 2.0,
                width / 2.0,
                top - flange,
                top,
            );
        }
        (_, Some(axis)) => axis.block(
            part,
            Structure,
            0.0,
            axis.length,
            -width / 2.0,
            width / 2.0,
            bottom,
            top,
        ),
        (_, None) => {
            let outline = if style == 1 {
                round(start, width, width, 12)
            } else {
                element_footprint(element)
            };
            part.convex_prism(&outline, bottom, top, Structure, Structure);
        }
    }
}
