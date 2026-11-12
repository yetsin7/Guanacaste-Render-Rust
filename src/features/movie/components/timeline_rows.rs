//! The track list, ruler and the four timeline rows.

use crate::components::{icons, paint, sparkline, text};
use crate::config::theme::{colors, typography};
use crate::features::movie::logic;
use crate::features::movie::models::MovieState;
use crate::services::localization::tr;
use eframe::egui::{
    self, epaint::PathShape, pos2, vec2, Align2, CornerRadius, Pos2, Rect, Sense, Shape, Stroke, Ui,
};

const LIST_WIDTH: f32 = 192.0;
const RULER: f32 = 24.0;
const ROW: f32 = 28.0;
const TRACKS: [(&str, egui::Color32); 4] = [
    ("movie.track_camera", colors::PRIMARY),
    ("movie.track_height", colors::TERTIARY),
    ("movie.track_focal", colors::SECONDARY),
    ("movie.track_sun", colors::PRIMARY_CONTAINER),
];
const HEIGHT_CURVE: [f32; 12] = [
    0.40, 0.42, 0.45, 0.47, 0.46, 0.44, 0.43, 0.42, 0.44, 0.48, 0.52, 0.58,
];

/// Vertical top of the `index`-th row inside `area`, spread with equal space around the rows.
pub fn row_top(area: Rect, index: usize) -> f32 {
    let free = area.height() - ROW * 4.0;
    let gap = free / 4.0;
    area.top() + gap / 2.0 + index as f32 * (ROW + gap)
}

/// Draws the list and the timeline inside `area` (below the toolbar).
pub fn show(ui: &mut Ui, area: Rect, state: &mut MovieState) {
    let list = Rect::from_min_size(area.min, vec2(LIST_WIDTH, area.height()));
    let lane = Rect::from_min_max(pos2(list.right(), area.top()), area.max);
    ui.painter()
        .rect_filled(list, 0.0, colors::SURFACE_CONTAINER);
    ui.painter().rect_filled(lane, 0.0, colors::SURFACE_LOWEST);
    track_list(ui, list, state);
    let rows = Rect::from_min_max(pos2(lane.left(), lane.top() + RULER), lane.max);
    ruler(
        ui,
        Rect::from_min_size(lane.min, vec2(lane.width(), RULER)),
        state,
    );
    let content = Rect::from_min_max(
        pos2(rows.left() + 16.0, rows.top()),
        pos2(rows.right() - 16.0, rows.bottom()),
    );
    lane_rows(ui, rows, content, state);
    playhead(ui, lane, content, state);
}

fn track_list(ui: &mut Ui, list: Rect, state: &mut MovieState) {
    let header = Rect::from_min_size(list.min, vec2(list.width(), RULER));
    ui.painter().rect_filled(header, 0.0, colors::SURFACE_HIGH);
    text::paint_text(
        ui.painter(),
        pos2(header.left() + 12.0, header.center().y),
        Align2::LEFT_CENTER,
        &text::caps(tr("movie.tracks")),
        typography::label_sm(),
        colors::OUTLINE,
    );
    text::paint_icon(
        ui.painter(),
        pos2(header.right() - 14.0, header.center().y),
        icons::LAYERS,
        13.0,
        colors::OUTLINE,
    );
    let rows = Rect::from_min_max(pos2(list.left(), list.top() + RULER), list.max);
    for (index, (key, color)) in TRACKS.into_iter().enumerate() {
        let top = row_top(rows, index);
        let rect =
            Rect::from_min_size(pos2(list.left() + 8.0, top), vec2(list.width() - 16.0, ROW));
        ui.painter()
            .rect_filled(rect, CornerRadius::same(2), colors::SURFACE_LOW);
        ui.painter()
            .circle_filled(pos2(rect.left() + 12.0, rect.center().y), 4.0, color);
        text::paint_text(
            ui.painter(),
            pos2(rect.left() + 24.0, rect.center().y),
            Align2::LEFT_CENTER,
            tr(key),
            typography::label_sm(),
            colors::ON_SURFACE,
        );
        let toggle =
            Rect::from_center_size(pos2(rect.right() - 14.0, rect.center().y), vec2(20.0, 20.0));
        let response = ui.interact(toggle, ui.id().with(("track-flag", index)), Sense::click());
        let flag = state.track_flags[index];
        let glyph = match (index % 3 == 0, flag) {
            (true, true) => icons::LOCK,
            (true, false) => icons::LOCK_OPEN,
            (false, true) => icons::VISIBILITY,
            (false, false) => icons::VISIBILITY_OFF,
        };
        let ink = if response.hovered() {
            colors::ON_SURFACE
        } else {
            colors::OUTLINE
        };
        text::paint_icon(ui.painter(), toggle.center(), glyph, 13.0, ink);
        if response.clicked() {
            state.track_flags[index] = !flag;
        }
    }
}

fn ruler(ui: &mut Ui, rect: Rect, state: &mut MovieState) {
    ui.painter()
        .rect_filled(rect, 0.0, colors::SURFACE_CONTAINER);
    let inner = Rect::from_min_max(
        pos2(rect.left() + 16.0, rect.top()),
        pos2(rect.right() - 16.0, rect.bottom()),
    );
    for index in 0..=7 {
        let fraction = index as f32 / 7.0;
        let x = inner.left() + inner.width() * fraction;
        let label = format!("00:{:02}", index * 2);
        let anchor = if index == 7 {
            Align2::RIGHT_CENTER
        } else {
            Align2::LEFT_CENTER
        };
        text::paint_text(
            ui.painter(),
            pos2(x, rect.center().y - 2.0),
            anchor,
            &label,
            typography::mono(10.0),
            colors::OUTLINE,
        );
        ui.painter().line_segment(
            [pos2(x, rect.bottom() - 6.0), pos2(x, rect.bottom())],
            Stroke::new(1.0, colors::with_alpha(colors::OUTLINE, 80)),
        );
    }
    let response = ui.interact(rect, ui.id().with("ruler"), Sense::click_and_drag());
    if let Some(pointer) = response.interact_pointer_pos() {
        logic::seek(state, (pointer.x - inner.left()) / inner.width());
    }
}

fn lane_rows(ui: &mut Ui, rows: Rect, content: Rect, state: &MovieState) {
    let painter = ui.painter();
    for index in 0..8 {
        let x = content.left() + content.width() * index as f32 / 7.0;
        painter.line_segment(
            [pos2(x, rows.top()), pos2(x, rows.bottom())],
            Stroke::new(1.0, colors::with_alpha(colors::OUTLINE, 26)),
        );
    }
    let x_of = |fraction: f32| content.left() + content.width() * fraction;
    let strips: Vec<Rect> = (0..4)
        .map(|index| {
            Rect::from_min_size(
                pos2(rows.left() + 16.0, row_top(rows, index)),
                vec2(rows.width() - 32.0, ROW),
            )
        })
        .collect();
    for strip in &strips {
        painter.rect_filled(*strip, CornerRadius::same(2), colors::SURFACE_CONTAINER);
    }
    // Camera row: keyframes with eased spans.
    let mid = strips[0].center().y;
    painter.rect_filled(
        Rect::from_min_max(
            pos2(strips[0].left() + 8.0, mid - 2.0),
            pos2(strips[0].right() - 8.0, mid + 2.0),
        ),
        CornerRadius::same(2),
        colors::SURFACE_HIGH,
    );
    for (from, to) in [(0.0, 0.57), (0.57, 1.0)] {
        painter.rect_filled(
            Rect::from_min_max(pos2(x_of(from), mid - 4.0), pos2(x_of(to), mid + 4.0)),
            CornerRadius::same(2),
            colors::with_alpha(colors::PRIMARY, 51),
        );
    }
    for fraction in [0.0, 0.57, 1.0] {
        diamond(
            painter,
            pos2(x_of(fraction), mid),
            7.0,
            colors::PRIMARY,
            None,
        );
    }
    diamond(
        painter,
        pos2(x_of(0.30), mid),
        6.0,
        colors::ON_SURFACE,
        Some(colors::PRIMARY),
    );
    for index in 0..state.added_keyframes.min(6) {
        diamond(
            painter,
            pos2(x_of(0.12 + 0.1 * index as f32), mid),
            6.0,
            colors::PRIMARY_CONTAINER,
            None,
        );
    }
    // Height row: curve with dots.
    let height = strips[1].shrink2(vec2(0.0, 6.0));
    paint::vertical_gradient(
        &painter.with_clip_rect(strips[1]),
        strips[1],
        colors::with_alpha(colors::TERTIARY, 26),
        egui::Color32::TRANSPARENT,
    );
    let points = sparkline::points_in(
        &HEIGHT_CURVE,
        Rect::from_min_max(
            pos2(x_of(0.0), height.top()),
            pos2(x_of(1.0), height.bottom()),
        ),
        (0.0, 1.0),
    );
    painter.add(Shape::line(points, Stroke::new(1.5, colors::TERTIARY)));
    for fraction in [0.0, 0.57, 1.0] {
        painter.circle_filled(
            pos2(x_of(fraction), strips[1].center().y),
            5.0,
            colors::TERTIARY,
        );
    }
    text::paint_text(
        painter,
        pos2(x_of(0.30) + 2.0, strips[1].top() + 5.0),
        Align2::LEFT_CENTER,
        "H: 3.42m",
        typography::mono(9.0),
        colors::TERTIARY,
    );
    // Focal row: clip bar.
    let clip = Rect::from_min_max(
        pos2(strips[2].left() + 12.0, strips[2].center().y - 6.0),
        pos2(strips[2].right() - 12.0, strips[2].center().y + 6.0),
    );
    painter.rect_filled(
        clip,
        CornerRadius::same(2),
        colors::with_alpha(colors::SECONDARY_CONTAINER, 77),
    );
    text::paint_text(
        painter,
        pos2(clip.left() + 12.0, clip.center().y),
        Align2::LEFT_CENTER,
        tr("movie.focal_clip"),
        typography::mono(10.0),
        colors::SECONDARY,
    );
    text::paint_text(
        painter,
        pos2(clip.right() - 12.0, clip.center().y),
        Align2::RIGHT_CENTER,
        tr("movie.focal_dof"),
        typography::mono(9.0),
        colors::OUTLINE,
    );
    // Sun row: node and diamond.
    let sun_mid = strips[3].center().y;
    painter.rect_filled(
        Rect::from_min_max(
            pos2(strips[3].left() + 8.0, sun_mid - 2.0),
            pos2(strips[3].right() - 8.0, sun_mid + 2.0),
        ),
        CornerRadius::same(2),
        colors::SURFACE_HIGH,
    );
    let node = Rect::from_min_size(pos2(x_of(0.30) + 4.0, sun_mid - 10.0), vec2(160.0, 20.0));
    painter.rect_filled(node, CornerRadius::same(2), colors::PRIMARY_CONTAINER);
    text::paint_icon(
        painter,
        pos2(node.left() + 12.0, node.center().y),
        icons::SUN,
        11.0,
        colors::ON_PRIMARY_CONTAINER,
    );
    text::paint_text(
        painter,
        pos2(node.left() + 24.0, node.center().y),
        Align2::LEFT_CENTER,
        tr("movie.sun_node"),
        typography::mono_medium(9.0),
        colors::ON_PRIMARY_CONTAINER,
    );
    diamond(
        painter,
        pos2(x_of(0.70), sun_mid),
        6.0,
        colors::with_alpha(colors::PRIMARY_CONTAINER, 204),
        None,
    );
}

fn playhead(ui: &mut Ui, lane: Rect, content: Rect, state: &MovieState) {
    let x = content.left() + content.width() * logic::fraction(state);
    let painter = ui.painter().with_clip_rect(lane);
    painter.line_segment(
        [pos2(x, lane.top() + RULER - 10.0), pos2(x, lane.bottom())],
        Stroke::new(1.0, colors::PRIMARY),
    );
    let head = Rect::from_min_size(pos2(x - 5.0, lane.top() + RULER - 12.0), vec2(10.0, 12.0));
    painter.rect_filled(
        head,
        CornerRadius {
            nw: 0,
            ne: 0,
            sw: 3,
            se: 3,
        },
        colors::PRIMARY,
    );
    painter.circle_filled(head.center(), 2.0, colors::ON_PRIMARY);
}

/// A rotated square keyframe marker, optionally with a ring.
fn diamond(
    painter: &egui::Painter,
    center: Pos2,
    half: f32,
    fill: egui::Color32,
    ring: Option<egui::Color32>,
) {
    let points = vec![
        center + vec2(0.0, -half),
        center + vec2(half, 0.0),
        center + vec2(0.0, half),
        center + vec2(-half, 0.0),
    ];
    let stroke = ring.map_or(Stroke::NONE, |color| Stroke::new(2.0, color));
    painter.add(Shape::Path(PathShape::convex_polygon(points, fill, stroke)));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rows_are_spread_evenly() {
        let area = Rect::from_min_size(pos2(0.0, 0.0), vec2(100.0, 152.0));
        let tops: Vec<f32> = (0..4).map(|i| row_top(area, i)).collect();
        assert_eq!(tops[0], 5.0);
        assert!(tops
            .windows(2)
            .all(|pair| (pair[1] - pair[0] - 38.0).abs() < 1e-3));
    }
}
