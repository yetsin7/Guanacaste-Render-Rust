//! The moving parts of an opening: window sashes with their glass and door leaves in each style.

use super::json::flag;
use super::openings::{OpeningType, HUNG, LOUVERED, SLIDING};
use super::part::{Key, Part};
use super::surface::Surface;
use super::walls::Wall;
use serde_json::Value;

/// The clear span inside a lining.
pub struct Span {
    pub from: f64,
    pub to: f64,
    pub bottom: f64,
    pub top: f64,
}

/// Window sashes: a frame of four members and a pane per light, in two tracks for sliding and hung windows and as
/// slats for louvered ones.
pub fn window(part: &mut Part, wall: &Wall, t: &OpeningType, span: &Span, width: f64) {
    let member = t.frame_width.clamp(0.02, 0.07);
    let tracked = t.operation == SLIDING || t.operation == HUNG;
    let louvered = t.operation == LOUVERED;
    let lights = if t.lights > 0 {
        t.lights
    } else if width > 1.0 {
        2
    } else {
        1
    };
    let columns = if t.operation == SLIDING {
        lights.max(2)
    } else if tracked || louvered {
        1
    } else {
        lights
    } as usize;
    let rows = if t.operation == HUNG { 2 } else { 1 };
    let (sash, glass) = if tracked {
        (0.02, 0.003)
    } else {
        (0.025, 0.003)
    };
    let (cell_w, cell_h) = (
        (span.to - span.from) / columns as f64,
        (span.top - span.bottom) / rows as f64,
    );
    for row in 0..rows {
        for column in 0..columns {
            let x0 = span.from + column as f64 * cell_w;
            let x1 = x0 + cell_w;
            let y0 = span.bottom + row as f64 * cell_h;
            let y1 = y0 + cell_h;
            let shift = if tracked && (row + column) % 2 == 1 {
                0.0225
            } else if tracked {
                -0.0225
            } else {
                0.0
            };
            let frame = Surface::OpeningFrame;
            wall.block(
                part,
                frame,
                x0,
                x0 + member,
                shift - sash,
                shift + sash,
                y0,
                y1,
            );
            wall.block(
                part,
                frame,
                x1 - member,
                x1,
                shift - sash,
                shift + sash,
                y0,
                y1,
            );
            wall.block(
                part,
                frame,
                x0 + member,
                x1 - member,
                shift - sash,
                shift + sash,
                y1 - member,
                y1,
            );
            wall.block(
                part,
                frame,
                x0 + member,
                x1 - member,
                shift - sash,
                shift + sash,
                y0,
                y0 + member,
            );
            if louvered {
                let mut level = y0 + member;
                while level + 0.045 <= y1 - member + 1e-9 {
                    wall.block(
                        part,
                        Surface::Glass,
                        x0 + member,
                        x1 - member,
                        -0.035,
                        0.035,
                        level,
                        level + 0.006,
                    );
                    level += 0.07;
                }
            } else {
                wall.block(
                    part,
                    Surface::Glass,
                    x0 + member,
                    x1 - member,
                    shift - glass,
                    shift + glass,
                    y0 + member,
                    y1 - member,
                );
            }
        }
    }
}

/// Door leaves: one, or two for a wide opening, in the style of the type, with a lever handle on both faces.
pub fn door(
    part: &mut Part,
    wall: &Wall,
    t: &OpeningType,
    item: &Value,
    span: &Span,
    depth: f64,
    bottom: f64,
) {
    let (clearance, gap) = (0.004, 0.012);
    let leaves = if span.to - span.from > 1.2 { 2 } else { 1 };
    let width = (span.to - span.from - 2.0 * clearance) / f64::from(leaves);
    let high = span.top - clearance;
    for leaf in 0..leaves {
        let from = span.from + clearance + f64::from(leaf) * width;
        leaf_geometry(part, wall, t, from, from + width, depth, bottom + gap, high);
    }
    if t.operation != 0 || t.leaf_style == 4 {
        return;
    }
    let flip = flag(item, "FlipHand", false);
    let half = depth / 2.0;
    let latches: Vec<f64> = if leaves == 2 {
        let mid = (span.from + span.to) / 2.0;
        vec![mid - 0.07, mid + 0.07]
    } else if flip {
        vec![span.from + 0.065]
    } else {
        vec![span.to - 0.065]
    };
    for at in latches {
        for face in [-1.0, 1.0] {
            let plate = face * half;
            let (lo, hi) = (
                plate.min(plate + face * 0.06),
                plate.max(plate + face * 0.06),
            );
            wall.block(
                part,
                Surface::Metal,
                at - 0.012,
                at + 0.012,
                lo,
                hi,
                bottom + 0.94,
                bottom + 0.96,
            );
            wall.block(
                part,
                Surface::Metal,
                at - 0.07,
                at + 0.07,
                lo.min(hi - 0.02),
                hi,
                bottom + 0.945,
                bottom + 0.955,
            );
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn leaf_geometry(
    part: &mut Part,
    wall: &Wall,
    t: &OpeningType,
    from: f64,
    to: f64,
    depth: f64,
    bottom: f64,
    top: f64,
) {
    let half = depth / 2.0;
    let door: Key = Surface::DoorLeaf.into();
    if t.operation == SLIDING || t.glazed || t.leaf_style == 4 {
        wall.block(
            part,
            Surface::Glass,
            from,
            to,
            -0.006,
            0.006,
            bottom + 0.01,
            top - 0.01,
        );
        let rail = 0.05;
        wall.block(
            part,
            Surface::OpeningFrame,
            from,
            to,
            -half,
            half,
            top - rail,
            top,
        );
        wall.block(
            part,
            Surface::OpeningFrame,
            from,
            to,
            -half,
            half,
            bottom,
            bottom + rail,
        );
        wall.block(
            part,
            Surface::OpeningFrame,
            from,
            from + rail,
            -half,
            half,
            bottom,
            top,
        );
        wall.block(
            part,
            Surface::OpeningFrame,
            to - rail,
            to,
            -half,
            half,
            bottom,
            top,
        );
        return;
    }
    if t.leaf_style == 1 || t.leaf_style >= 2 {
        // A flush leaf: one smooth slab.
        wall.block(part, door, from, to, -half, half, bottom, top);
        return;
    }
    // Panelled: stiles and rails around two recessed fields with a raised center.
    let (stile, rail) = (0.11, 0.11);
    wall.block(part, door, from, from + stile, -half, half, bottom, top);
    wall.block(part, door, to - stile, to, -half, half, bottom, top);
    wall.block(
        part,
        door,
        from + stile,
        to - stile,
        -half,
        half,
        top - rail,
        top,
    );
    wall.block(
        part,
        door,
        from + stile,
        to - stile,
        -half,
        half,
        bottom,
        bottom + 0.22,
    );
    let lock = (bottom + 0.88).min(top - rail - 0.4);
    wall.block(
        part,
        door,
        from + stile,
        to - stile,
        -half,
        half,
        lock,
        lock + 0.16,
    );
    for (low, high) in [(bottom + 0.22, lock), (lock + 0.16, top - rail)] {
        if high - low < 0.05 {
            continue;
        }
        wall.block(
            part,
            door,
            from + stile,
            to - stile,
            -half * 0.35,
            half * 0.35,
            low,
            high,
        );
        if to - from - 2.0 * (stile + 0.04) > 0.05 && high - low > 0.13 {
            wall.block(
                part,
                door,
                from + stile + 0.04,
                to - stile - 0.04,
                -half * 0.7,
                half * 0.7,
                low + 0.04,
                high - 0.04,
            );
        }
    }
}
