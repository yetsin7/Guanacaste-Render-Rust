//! Stairs, guards and curtain walls (glazed or cobogó), drawn as boxes along their axis.

use super::axis::Axis;
use super::context::Context;
use super::json::{int, number, point};
use super::part::Part;
use super::surface::Surface::*;
use crate::features::import_export::models::Severity;
use serde_json::Value;

const MAX_RISER: f64 = 0.175;
const STAIR_OPEN: i64 = 8;
/// The `Style` of a curtain wall that is a cobogó screen.
pub const BREEZE_BLOCK: i64 = 16;

/// Bottom and top elevation of an element standing on a level at `datum`.
pub fn span(element: &Value, datum: f64) -> (f64, f64) {
    let bottom = datum + number(element, "Offset");
    (bottom, bottom + number(element, "Height"))
}

/// Straight stairs: solid steps, or treads on stringers for the open style. L, U and spiral stairs are straight runs.
pub fn stair(ctx: &mut Context, part: &mut Part, element: &Value, datum: f64) {
    let (bottom, _) = span(element, datum);
    let height = number(element, "Height");
    let Some(axis) = Axis::new(point(element, "Start"), point(element, "End")) else {
        return;
    };
    if matches!(int(element, "Style"), 13..=15) {
        ctx.report
            .note(Severity::Info, "legacy.stair_straight", &[]);
    }
    let riser = number(element, "RiserHeight");
    let count = if riser > 0.0 {
        (height / riser).ceil()
    } else {
        (height / MAX_RISER - 1e-9).ceil()
    }
    .max(2.0) as usize;
    let width = number(element, "Width");
    let open = int(element, "Style") == STAIR_OPEN;
    for i in 0..count {
        let (from, to) = (
            axis.length * i as f64 / count as f64,
            axis.length * (i + 1) as f64 / count as f64,
        );
        let top = bottom + height * (i + 1) as f64 / count as f64;
        if open {
            let thick = 0.04f64.min(height / count as f64 / 2.0);
            axis.block(
                part,
                DoorLeaf,
                from,
                to,
                -width / 2.0,
                width / 2.0,
                top - thick,
                top,
            );
            for side in [-1.0, 1.0] {
                let c = side * (width / 2.0 - 0.03);
                axis.block(
                    part,
                    Metal,
                    from,
                    to,
                    c - 0.03,
                    c + 0.03,
                    (top - thick - 0.2).max(bottom),
                    top - thick,
                );
            }
        } else {
            axis.block(
                part,
                Structure,
                from,
                to,
                -width / 2.0,
                width / 2.0,
                bottom,
                top,
            );
        }
    }
}

/// Guards: posts and rails, balusters, a glass panel, cables or a solid parapet.
pub fn railing(part: &mut Part, element: &Value, datum: f64) {
    let (bottom, top) = span(element, datum);
    let Some(axis) = Axis::new(point(element, "Start"), point(element, "End")) else {
        return;
    };
    let (width, style) = (number(element, "Width").max(0.02), int(element, "Style"));
    let rise = number(element, "Rise");
    let z = |t: f64| rise * t / axis.length;
    if style == 7 {
        axis.block(
            part,
            Structure,
            0.0,
            axis.length,
            -0.075,
            0.075,
            bottom,
            top - 0.05,
        );
        axis.block(
            part,
            Structure,
            0.0,
            axis.length,
            -0.105,
            0.105,
            top - 0.05,
            top,
        );
        return;
    }
    let spacing = if style == 6 { 1.2 } else { 1.8 };
    let intervals = (axis.length / spacing).ceil().max(1.0) as usize;
    for i in 0..=intervals {
        let t = axis.length * i as f64 / intervals as f64;
        axis.block(
            part,
            Metal,
            t - 0.03,
            t + 0.03,
            -width.max(0.06) / 2.0,
            width.max(0.06) / 2.0,
            bottom + z(t),
            top + z(t),
        );
    }
    axis.block(
        part,
        Metal,
        0.0,
        axis.length,
        -width / 2.0 - 0.01,
        width / 2.0 + 0.01,
        top - 0.05,
        top,
    );
    match style {
        4 => {
            let count = (axis.length / 0.11).floor() as usize;
            for i in 1..count.min(2000) {
                let t = i as f64 * 0.11;
                axis.block(
                    part,
                    Metal,
                    t - 0.01,
                    t + 0.01,
                    -0.01,
                    0.01,
                    bottom + 0.13 + z(t),
                    top - 0.05 + z(t),
                );
            }
        }
        5 => axis.block(
            part,
            Glass,
            0.0,
            axis.length,
            -0.006,
            0.006,
            bottom + 0.1,
            top - 0.05,
        ),
        6 => {
            for i in 0..5 {
                let h = bottom + 0.12 + (top - bottom - 0.22) * f64::from(i) / 4.0;
                axis.block(part, Metal, 0.0, axis.length, -0.006, 0.006, h, h + 0.012);
            }
        }
        _ => axis.block(
            part,
            Metal,
            0.0,
            axis.length,
            -width / 2.0,
            width / 2.0,
            (bottom + top) / 2.0 - 0.05,
            (bottom + top) / 2.0,
        ),
    }
}

/// A glazed curtain wall with mullions and transoms, or a cobogó screen: thin masonry webs on a square module that
/// leave real holes for light and views.
pub fn curtain_wall(part: &mut Part, element: &Value, datum: f64) {
    let (bottom, top) = span(element, datum);
    let Some(axis) = Axis::new(point(element, "Start"), point(element, "End")) else {
        return;
    };
    let (width, grid_w, grid_h) = (
        number(element, "Width"),
        number(element, "GridWidth"),
        number(element, "GridHeight"),
    );
    let height = top - bottom;
    if int(element, "Style") == BREEZE_BLOCK {
        let web = 0.04;
        let columns = (axis.length / if grid_w > 0.0 { grid_w } else { 0.2 })
            .round()
            .max(1.0) as usize;
        for i in 0..=columns {
            let t = axis.length * i as f64 / columns as f64;
            axis.block(
                part,
                Wall,
                (t - web / 2.0).max(0.0),
                (t + web / 2.0).min(axis.length),
                -width / 2.0,
                width / 2.0,
                bottom,
                top,
            );
        }
        let rows = (height / if grid_h > 0.0 { grid_h } else { 0.2 })
            .round()
            .max(1.0) as usize;
        for i in 0..=rows {
            let c = bottom + height * i as f64 / rows as f64;
            axis.block(
                part,
                Wall,
                0.0,
                axis.length,
                -width / 2.0,
                width / 2.0,
                (c - web / 2.0).max(bottom),
                (c + web / 2.0).min(top),
            );
        }
        return;
    }
    let member = 0.06;
    axis.block(part, Glass, 0.0, axis.length, -0.006, 0.006, bottom, top);
    let columns = (axis.length / if grid_w > 0.0 { grid_w } else { 1.5 })
        .ceil()
        .max(1.0) as usize;
    for i in 0..=columns {
        let t = axis.length * i as f64 / columns as f64;
        axis.block(
            part,
            OpeningFrame,
            (t - member / 2.0).max(0.0),
            (t + member / 2.0).min(axis.length),
            -width / 2.0,
            width / 2.0,
            bottom,
            top,
        );
    }
    let rows = (height / if grid_h > 0.0 { grid_h } else { 1.2 })
        .ceil()
        .max(1.0) as usize;
    for i in 0..=rows {
        let c = bottom + height * i as f64 / rows as f64;
        axis.block(
            part,
            OpeningFrame,
            0.0,
            axis.length,
            -width / 2.0,
            width / 2.0,
            (c - member / 2.0).max(bottom),
            (c + member / 2.0).min(top),
        );
    }
}
