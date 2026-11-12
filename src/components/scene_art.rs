//! Paints the procedural placeholder thumbnails described by [`SceneArt`]. They stand in for real scene captures.

use super::paint;
use crate::config::theme::colors::hex;
use crate::models::scene_art::{SceneArt, SceneKind};
use eframe::egui::{epaint::PathShape, Color32, Painter, Pos2, Rect, Shape, Stroke, Vec2};

/// Paints `art` filling `rect`.
pub fn paint_scene(painter: &Painter, rect: Rect, art: &SceneArt) {
    let painter = painter.with_clip_rect(rect.intersect(painter.clip_rect()));
    let [top, bottom, ground, accent] = art.palette.map(hex);
    paint::vertical_gradient(&painter, rect, top, bottom);
    let horizon = rect.top() + rect.height() * 0.68;
    let at = |fx: f32, fy: f32| {
        Pos2::new(
            rect.left() + rect.width() * fx,
            rect.top() + rect.height() * fy,
        )
    };
    let dark = paint::mix(ground, Color32::BLACK, 0.5);
    match art.kind {
        SceneKind::Pavilion => {
            trees(&painter, rect, horizon, accent, 7);
            painter.rect_filled(Rect::from_min_max(at(0.22, 0.42), at(0.78, 0.7)), 2.0, dark);
            painter.rect_filled(
                Rect::from_min_max(at(0.25, 0.48), at(0.75, 0.68)),
                1.0,
                glow(accent),
            );
        }
        SceneKind::Retreat => {
            trees(&painter, rect, horizon, accent, 9);
            polygon(&painter, &[at(0.3, 0.5), at(0.5, 0.36), at(0.7, 0.5)], dark);
            painter.rect_filled(Rect::from_min_max(at(0.33, 0.5), at(0.67, 0.68)), 1.0, dark);
            painter.rect_filled(
                Rect::from_min_max(at(0.4, 0.54), at(0.6, 0.66)),
                1.0,
                glow(hex(0xf5c27a)),
            );
        }
        SceneKind::Cliff => {
            polygon(
                &painter,
                &[at(0.0, 0.45), at(0.55, 0.45), at(0.7, 1.0), at(0.0, 1.0)],
                dark,
            );
            painter.rect_filled(
                Rect::from_min_max(at(0.18, 0.32), at(0.62, 0.45)),
                1.0,
                paint::mix(dark, Color32::WHITE, 0.12),
            );
            painter.rect_filled(
                Rect::from_min_max(at(0.22, 0.36), at(0.58, 0.43)),
                1.0,
                glow(hex(0xf5c27a)),
            );
        }
        SceneKind::Chalet => {
            polygon(
                &painter,
                &[at(0.0, 0.7), at(0.25, 0.25), at(0.5, 0.7)],
                hex(0xdde6f2),
            );
            polygon(
                &painter,
                &[at(0.35, 0.7), at(0.7, 0.18), at(1.0, 0.7)],
                hex(0xc4d2e6),
            );
            polygon(
                &painter,
                &[at(0.34, 0.62), at(0.5, 0.48), at(0.66, 0.62)],
                dark,
            );
            painter.rect_filled(
                Rect::from_min_max(at(0.38, 0.62), at(0.62, 0.74)),
                1.0,
                dark,
            );
            painter.rect_filled(
                Rect::from_min_max(at(0.46, 0.65), at(0.54, 0.72)),
                1.0,
                glow(hex(0xf5c27a)),
            );
        }
    }
    painter.rect_filled(
        Rect::from_min_max(Pos2::new(rect.left(), horizon), rect.right_bottom()),
        0.0,
        ground,
    );
    if art.kind == SceneKind::Cliff {
        painter.rect_filled(
            Rect::from_min_max(Pos2::new(rect.left(), horizon + 6.0), rect.right_bottom()),
            0.0,
            paint::mix(top, ground, 0.5),
        );
    }
}

fn glow(color: Color32) -> Color32 {
    paint::mix(color, Color32::WHITE, 0.25)
}

fn polygon(painter: &Painter, points: &[Pos2], color: Color32) {
    painter.add(Shape::Path(PathShape::convex_polygon(
        points.to_vec(),
        color,
        Stroke::NONE,
    )));
}

/// Rows of round tree crowns along the horizon.
fn trees(painter: &Painter, rect: Rect, horizon: f32, accent: Color32, count: usize) {
    for index in 0..count {
        let t = (index as f32 + 0.5) / count as f32;
        let jitter = ((index * 37 % 11) as f32 / 11.0 - 0.5) * 0.4;
        let radius = rect.height() * (0.16 + (index * 13 % 7) as f32 * 0.012);
        let center = Pos2::new(
            rect.left() + rect.width() * t,
            horizon - radius * (0.7 + jitter),
        );
        let shade = paint::mix(accent, Color32::BLACK, 0.35 + (index % 3) as f32 * 0.1);
        painter.circle_filled(center, radius, shade);
        painter.circle_filled(
            center + Vec2::new(-radius * 0.25, -radius * 0.2),
            radius * 0.6,
            paint::mix(shade, accent, 0.3),
        );
    }
}
