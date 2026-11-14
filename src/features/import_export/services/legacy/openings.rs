//! Doors and windows built in layers like the original: a lining recessed in the rough opening, then a door leaf
//! (panelled, flush, glazed) with its handle, or a window sash with glass, glazing bars and a projecting sill.

use super::json::{flag, int, number, text};
use super::leaves::{door, window, Span};
use super::part::Part;
use super::surface::Surface;
use super::walls::Wall;
use serde_json::Value;

/// The family type of a door or window.
pub struct OpeningType {
    /// Name shown in the hierarchy.
    pub display_name: String,
    pub depth: f64,
    pub frame_width: f64,
    pub glazed: bool,
    pub lights: i64,
    pub leaf_style: i64,
    pub operation: i64,
    pub bars: bool,
}

pub const SLIDING: i64 = 1;
pub const REVOLVING: i64 = 5;
pub const HUNG: i64 = 8;
pub const LOUVERED: i64 = 9;
const VOID: i64 = 10;

impl OpeningType {
    /// Reads a type from the document.
    pub fn read(value: &Value) -> Self {
        Self {
            display_name: [text(value, "DisplayName"), text(value, "Name")]
                .into_iter()
                .find(|n| !n.is_empty())
                .unwrap_or("Abertura")
                .to_owned(),
            depth: number(value, "Depth"),
            frame_width: number(value, "FrameWidth"),
            glazed: flag(value, "GlazedLeaf", false),
            lights: int(value, "GlazingLightCount"),
            leaf_style: int(value, "LeafStyle"),
            operation: int(value, "Operation"),
            bars: flag(value, "SecurityBars", false),
        }
    }
}

/// A pane of glass between two stations of its wall in aluminium head and sill channels, with a mullion at least
/// every 1.2 m.
pub fn glass_wall(part: &mut Part, wall: &Wall, from: f64, to: f64, z0: f64, z1: f64) {
    let (channel, member) = (0.06, 0.05);
    wall.block(
        part,
        Surface::Glass,
        from,
        to,
        -0.006,
        0.006,
        z0 + channel,
        z1 - channel,
    );
    for (a, b) in [(z0, z0 + channel), (z1 - channel, z1)] {
        wall.block(part, Surface::OpeningFrame, from, to, -0.03, 0.03, a, b);
    }
    let panes = ((to - from) / 1.2).ceil().max(1.0) as usize;
    for k in 0..=panes {
        let at = from + (to - from) * k as f64 / panes as f64;
        let (a, b) = ((at - member / 2.0).max(from), (at + member / 2.0).min(to));
        wall.block(part, Surface::OpeningFrame, a, b, -0.03, 0.03, z0, z1);
    }
}

/// Builds the assembly of the opening `item` hosted by `wall`.
pub fn assembly(part: &mut Part, wall: &Wall, item: &Value, t: &OpeningType) {
    if t.operation == VOID {
        return;
    }
    let (center, width) = (number(item, "Offset"), number(item, "Width"));
    let height = number(item, "Height");
    let (begin, end) = (center - width / 2.0, center + width / 2.0);
    let bottom = wall.base + number(item, "Sill");
    let top = bottom + height;
    let lining = t.frame_width.min(width.min(height) / 6.0);
    let half = wall.thickness / 2.0 - 0.01f64.min(wall.thickness / 4.0);
    let frame = Surface::OpeningFrame;
    wall.block(part, frame, begin, begin + lining, -half, half, bottom, top);
    wall.block(part, frame, end - lining, end, -half, half, bottom, top);
    wall.block(
        part,
        frame,
        begin + lining,
        end - lining,
        -half,
        half,
        top - lining,
        top,
    );
    let span = Span {
        from: begin + lining,
        to: end - lining,
        bottom: bottom + lining,
        top: top - lining,
    };
    if int(item, "Kind") == 1 {
        window(part, wall, t, &span, width);
        wall.block(
            part,
            frame,
            begin - 0.03,
            end + 0.03,
            -(half + 0.01),
            half + 0.01,
            bottom - 0.03,
            bottom,
        );
    } else {
        let depth = if t.operation == REVOLVING {
            t.depth
        } else if t.depth > 0.0 {
            t.depth.min(wall.thickness)
        } else {
            0.0
        };
        if depth > 0.0 {
            door(part, wall, t, item, &span, depth, bottom);
        }
    }
    if t.bars {
        security_bars(part, wall, begin, end, bottom, top);
    }
}

/// A grille of square bars across a window or door, on the outer face of the wall.
fn security_bars(part: &mut Part, wall: &Wall, begin: f64, end: f64, bottom: f64, top: f64) {
    let outer = wall.thickness / 2.0;
    let mut x = begin + 0.1;
    while x < end - 0.05 {
        wall.block(
            part,
            Surface::OpeningFrame,
            x - 0.008,
            x + 0.008,
            outer - 0.016,
            outer,
            bottom,
            top,
        );
        x += 0.1;
    }
    for z in [
        bottom + (top - bottom) * 0.33,
        bottom + (top - bottom) * 0.66,
    ] {
        wall.block(
            part,
            Surface::OpeningFrame,
            begin,
            end,
            outer - 0.02,
            outer,
            z,
            z + 0.03,
        );
    }
}
