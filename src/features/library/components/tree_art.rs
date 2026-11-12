//! Paints placeholder tree thumbnails: a studio backdrop, a tree and a scale figure. They stand in for the real
//! asset previews.

use crate::components::paint;
use crate::config::theme::colors::hex;
use crate::features::library::models::{CanopyKind, TreeStyle};
use eframe::egui::{epaint::PathShape, vec2, Color32, Painter, Pos2, Rect, Shape, Stroke};

/// Paints the tree described by `style` filling `rect`; `height_m` sizes the scale figure.
pub fn paint_tree(painter: &Painter, rect: Rect, style: &TreeStyle, height_m: f32) {
    let painter = painter.with_clip_rect(rect.intersect(painter.clip_rect()));
    paint::vertical_gradient(&painter, rect, hex(0x3b3e44), hex(0x141518));
    let base_y = rect.top() + rect.height() * 0.84;
    let floor = Rect::from_min_max(
        Pos2::new(rect.left(), base_y - rect.height() * 0.05),
        rect.right_bottom(),
    );
    paint::vertical_gradient(
        &painter,
        floor,
        Color32::TRANSPARENT,
        Color32::from_black_alpha(120),
    );
    let center_x = rect.center().x - rect.width() * 0.03;
    let tree_height = rect.height() * 0.66;
    let trunk = hex(style.trunk);
    let [dark, light] = style.leaves.map(hex);
    match style.kind {
        CanopyKind::Conifer => {
            trunk_shape(
                &painter,
                center_x,
                base_y,
                rect.width() * 0.03,
                tree_height * 0.9,
                trunk,
            );
            for layer in 0..5 {
                let t = layer as f32 / 5.0;
                let y = base_y - tree_height * (0.22 + 0.17 * layer as f32);
                let half = rect.width() * (0.2 - 0.032 * layer as f32);
                let color = paint::mix(dark, light, t);
                painter.add(Shape::Path(PathShape::convex_polygon(
                    vec![
                        Pos2::new(center_x - half, y),
                        Pos2::new(center_x, y - tree_height * 0.3),
                        Pos2::new(center_x + half, y),
                    ],
                    color,
                    Stroke::NONE,
                )));
            }
        }
        kind => {
            trunk_shape(
                &painter,
                center_x,
                base_y,
                rect.width() * 0.035,
                tree_height * 0.55,
                trunk,
            );
            let spread = if kind == CanopyKind::Blossom {
                0.30
            } else {
                0.24
            };
            let center = Pos2::new(center_x, base_y - tree_height * 0.7);
            let radius = rect.width() * spread;
            let count = 40;
            for index in 0..count {
                let angle = index as f32 * 2.399;
                let distance = ((index as f32 + 0.5) / count as f32).sqrt();
                let offset = vec2(
                    angle.cos() * radius * distance,
                    angle.sin() * radius * 0.78 * distance,
                );
                let color = paint::mix(dark, light, (index % 5) as f32 / 4.0);
                painter.circle_filled(
                    center + offset,
                    radius * (0.24 + (index % 4) as f32 * 0.02),
                    color,
                );
            }
        }
    }
    figure(
        &painter,
        Pos2::new(center_x + rect.width() * 0.2, base_y),
        tree_height * 1.8 / height_m.max(1.0),
    );
}

fn trunk_shape(painter: &Painter, x: f32, base_y: f32, width: f32, height: f32, color: Color32) {
    let top = Pos2::new(x, base_y - height);
    painter.add(Shape::Path(PathShape::convex_polygon(
        vec![
            Pos2::new(x - width, base_y),
            Pos2::new(x - width * 0.45, top.y),
            Pos2::new(x + width * 0.45, top.y),
            Pos2::new(x + width, base_y),
        ],
        color,
        Stroke::NONE,
    )));
}

/// A person silhouette `height` points tall standing at `base`.
fn figure(painter: &Painter, base: Pos2, height: f32) {
    let ink = Color32::from_rgba_unmultiplied(12, 12, 14, 235);
    let head = height * 0.085;
    painter.circle_filled(Pos2::new(base.x, base.y - height + head), head, ink);
    painter.rect_filled(
        Rect::from_min_max(
            Pos2::new(base.x - head * 1.4, base.y - height + head * 2.2),
            Pos2::new(base.x + head * 1.4, base.y - height * 0.45),
        ),
        head * 0.6,
        ink,
    );
    painter.rect_filled(
        Rect::from_min_max(
            Pos2::new(base.x - head * 1.1, base.y - height * 0.47),
            Pos2::new(base.x + head * 1.1, base.y),
        ),
        head * 0.3,
        ink,
    );
}
