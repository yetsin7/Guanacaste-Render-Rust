//! Walls: solids split around the hosted openings, mitered where two walls meet, with the facade surface of the wall
//! type (plaster, brick, block, concrete, stone, timber, metal or a glass partition).

use super::context::Context;
use super::json::{int, list, number, point, text};
use super::openings::{self, OpeningType};
use super::part::{Key, Part};
use super::surface::Surface;
use serde_json::Value;
use std::collections::HashMap;

/// A wall with its derived frame.
pub struct Wall<'a> {
    pub item: &'a Value,
    pub start: [f64; 2],
    pub end: [f64; 2],
    pub thickness: f64,
    pub height: f64,
    /// Elevation of the level the wall stands on plus its base offset.
    pub base: f64,
    pub length: f64,
    pub dir: [f64; 2],
}

impl Wall<'_> {
    /// Left normal of the wall direction.
    pub fn normal(&self) -> [f64; 2] {
        [-self.dir[1], self.dir[0]]
    }

    /// Plan point at distance `along` from the start and `side` meters to the left of the axis.
    pub fn station(&self, along: f64, side: f64) -> [f64; 2] {
        let n = self.normal();
        [
            self.start[0] + self.dir[0] * along + n[0] * side,
            self.start[1] + self.dir[1] * along + n[1] * side,
        ]
    }

    /// A box in wall coordinates: `from..to` along the axis, `a..b` across it, `z0..z1` absolute.
    #[allow(clippy::too_many_arguments)]
    pub fn block(
        &self,
        part: &mut Part,
        key: impl Into<Key>,
        from: f64,
        to: f64,
        a: f64,
        b: f64,
        z0: f64,
        z1: f64,
    ) {
        if to - from < 1e-6 || (b - a).abs() < 1e-6 || (z1 - z0).abs() < 1e-6 {
            return;
        }
        let outline = [
            self.station(from, a),
            self.station(to, a),
            self.station(to, b),
            self.station(from, b),
        ];
        let key = key.into();
        part.convex_prism(&outline, z0, z1, key, key);
    }
}

/// Which end of a wall.
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
enum End {
    Start,
    End,
}

/// Pairs of wall ends that meet at a corner shared by exactly two walls.
struct Joins(HashMap<(usize, End), (usize, End)>);

fn corner(wall: &Wall, end: End) -> [f64; 2] {
    if end == End::Start {
        wall.start
    } else {
        wall.end
    }
}

impl Joins {
    fn build(walls: &[Wall], levels: &[&str]) -> Self {
        #[allow(clippy::type_complexity)]
        let mut cells: HashMap<(&str, i64, i64), Vec<(usize, End)>> = HashMap::new();
        for (index, wall) in walls.iter().enumerate() {
            for end in [End::Start, End::End] {
                let c = corner(wall, end);
                let key = (
                    levels[index],
                    (c[0] * 1e4).round() as i64,
                    (c[1] * 1e4).round() as i64,
                );
                cells.entry(key).or_default().push((index, end));
            }
        }
        let mut joins = HashMap::new();
        for ends in cells.values() {
            if let [a, b] = ends[..] {
                if a.0 != b.0 {
                    joins.insert(a, b);
                    joins.insert(b, a);
                }
            }
        }
        Self(joins)
    }

    /// Left and right corners of `end` of wall `index`, mitered with its neighbor when it has one.
    fn faces(&self, walls: &[Wall], index: usize, end: End) -> ([f64; 2], [f64; 2]) {
        let wall = &walls[index];
        let c = corner(wall, end);
        let (n, half) = (wall.normal(), wall.thickness / 2.0);
        let square = (
            [c[0] + n[0] * half, c[1] + n[1] * half],
            [c[0] - n[0] * half, c[1] - n[1] * half],
        );
        let Some(&(other, other_end)) = self.0.get(&(index, end)) else {
            return square;
        };
        let o = &walls[other];
        let mut on = o.normal();
        if end == other_end {
            on = [-on[0], -on[1]];
        }
        let oh = o.thickness / 2.0;
        let det = n[0] * on[1] - n[1] * on[0];
        if det.abs() < 1e-9 {
            return square;
        }
        let ox = (half * on[1] - oh * n[1]) / det;
        let oy = (n[0] * oh - on[0] * half) / det;
        if ox.hypot(oy) > (4.0 * half.max(oh)).min(wall.length.min(o.length) / 2.0) {
            return square;
        }
        ([c[0] + ox, c[1] + oy], [c[0] - ox, c[1] - oy])
    }
}

/// An opening of the document, hosted by a wall.
struct Hosted<'a> {
    item: &'a Value,
    begin: f64,
    end: f64,
    sill: f64,
    top: f64,
}

/// The facade surface a wall construction (the wall type's `Construction`) takes.
fn facade(construction: i64) -> Surface {
    match construction {
        3 => Surface::WallBrick,
        4 => Surface::WallBlock,
        5 => Surface::WallConcrete,
        6 => Surface::WallStone,
        7 => Surface::WallWood,
        8 => Surface::WallMetal,
        _ => Surface::Wall,
    }
}

/// Splits `0..length` into the solids around `openings` as `(from, to, bottom, top)` relative to the wall base.
fn solids(length: f64, height: f64, openings: &[Hosted]) -> Vec<(f64, f64, f64, f64)> {
    let mut out = Vec::new();
    let mut cursor = 0.0;
    for o in openings {
        let (begin, end) = (o.begin.max(cursor), o.end.min(length));
        if end - begin < 1e-6 {
            continue;
        }
        if begin - cursor >= 1e-6 {
            out.push((cursor, begin, 0.0, height));
        }
        if o.sill > 1e-9 {
            out.push((begin, end, 0.0, o.sill.min(height)));
        }
        if o.top < height - 1e-9 {
            out.push((begin, end, o.top.max(0.0), height));
        }
        cursor = end;
    }
    if length - cursor >= 1e-6 {
        out.push((cursor, length, 0.0, height));
    }
    out
}

/// Reads one wall; zero-length walls are `None` so they are reported as degenerate.
fn read_wall<'a>(ctx: &Context, item: &'a Value) -> Option<Wall<'a>> {
    let (start, end) = (point(item, "Start"), point(item, "End"));
    let length = (end[0] - start[0]).hypot(end[1] - start[1]);
    if !(length > 1e-9 && length.is_finite()) {
        return None;
    }
    let base = ctx.elevation(item);
    let top = item
        .get("TopLevelId")
        .and_then(Value::as_str)
        .and_then(|id| ctx.level_elevation(id))
        .map(|z| z + number(item, "TopOffset"));
    Some(Wall {
        item,
        start,
        end,
        thickness: number(item, "Thickness").abs(),
        height: top.map_or(number(item, "Height"), |top| top - base),
        base: base + number(item, "BaseOffset"),
        length,
        dir: [(end[0] - start[0]) / length, (end[1] - start[1]) / length],
    })
}

/// Builds every wall and its hosted openings.
pub fn build(ctx: &mut Context, document: &Value) {
    let types: HashMap<&str, &Value> = list(document, "WallTypes")
        .iter()
        .map(|t| (text(t, "Id"), t))
        .collect();
    let opening_types: HashMap<&str, OpeningType> = list(document, "OpeningTypes")
        .iter()
        .map(|t| (text(t, "Id"), OpeningType::read(t)))
        .collect();
    let mut hosted: HashMap<&str, Vec<Hosted>> = HashMap::new();
    for item in list(document, "Openings") {
        let (center, width) = (number(item, "Offset"), number(item, "Width"));
        let (sill, height) = (number(item, "Sill"), number(item, "Height"));
        hosted
            .entry(text(item, "WallId"))
            .or_default()
            .push(Hosted {
                item,
                begin: center - width / 2.0,
                end: center + width / 2.0,
                sill,
                top: sill + height,
            });
    }
    for openings in hosted.values_mut() {
        openings.sort_by(|a, b| a.begin.total_cmp(&b.begin));
    }
    let mut walls: Vec<Wall> = Vec::new();
    let mut order: Vec<(&Value, Option<usize>)> = Vec::new();
    for item in list(document, "Walls") {
        let wall = read_wall(ctx, item);
        order.push((item, wall.as_ref().map(|_| walls.len())));
        walls.extend(wall);
    }
    let levels: Vec<&str> = walls.iter().map(|w| text(w.item, "LevelId")).collect();
    let joins = Joins::build(&walls, &levels);
    for (item, slot) in order {
        let Some(index) = slot else {
            ctx.element(item, "Wall", None, Part::new());
            continue;
        };
        let wall = &walls[index];
        let construction = types
            .get(text(item, "WallTypeId"))
            .map_or(0, |t| int(t, "Construction"));
        let surface = facade(construction);
        let openings = hosted.get(text(item, "Id")).map_or(&[][..], Vec::as_slice);
        let mut part = Part::new();
        for (from, to, bottom, top) in solids(wall.length, wall.height, openings) {
            let (z0, z1) = (wall.base + bottom, wall.base + top);
            if construction == 2 {
                openings::glass_wall(&mut part, wall, from, to, z0, z1);
                continue;
            }
            let half = wall.thickness / 2.0;
            let (start_left, start_right) = if from <= 1e-4 {
                joins.faces(&walls, index, End::Start)
            } else {
                (wall.station(from, half), wall.station(from, -half))
            };
            let (end_left, end_right) = if to >= wall.length - 1e-4 {
                joins.faces(&walls, index, End::End)
            } else {
                (wall.station(to, half), wall.station(to, -half))
            };
            part.convex_prism(
                &[start_left, end_left, end_right, start_right],
                z0,
                z1,
                surface,
                surface,
            );
        }
        ctx.element(item, "Wall", None, part);
        let level = text(item, "LevelId");
        for o in openings {
            let Some(kind) = opening_types.get(text(o.item, "OpeningTypeId")) else {
                continue;
            };
            let mut part = Part::new();
            openings::assembly(&mut part, wall, o.item, kind);
            let class = if int(o.item, "Kind") == 0 {
                "Door"
            } else {
                "Window"
            };
            ctx.element_on(o.item, level, class, Some(&kind.display_name), part);
        }
    }
}
