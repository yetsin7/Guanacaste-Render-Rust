//! The camera path over the 3D view: a dashed spline with keyframe nodes and the active camera marker.

use crate::components::{tag::Tag, text};
use crate::config::theme::{colors, typography};
use crate::features::movie::logic::timecode;
use crate::features::movie::models::MovieState;
use crate::services::localization::tr;
use eframe::egui::{pos2, vec2, Pos2, Rect, Shape, Stroke, Ui, UiBuilder};

/// Path control points as fractions of the 3D view.
const POINTS: [(f32, f32); 6] = [
    (0.10, 0.60),
    (0.24, 0.50),
    (0.425, 0.39),
    (0.58, 0.50),
    (0.78, 0.50),
    (1.0, 0.36),
];

/// Catmull-Rom interpolation through `points`, `steps` samples per segment.
pub fn spline(points: &[Pos2], steps: usize) -> Vec<Pos2> {
    let mut out = Vec::new();
    for index in 0..points.len().saturating_sub(1) {
        let p0 = points[index.saturating_sub(1)];
        let p1 = points[index];
        let p2 = points[index + 1];
        let p3 = points[(index + 2).min(points.len() - 1)];
        for step in 0..steps {
            let t = step as f32 / steps as f32;
            let t2 = t * t;
            let t3 = t2 * t;
            let blend = |a: f32, b: f32, c: f32, d: f32| {
                0.5 * (2.0 * b
                    + (-a + c) * t
                    + (2.0 * a - 5.0 * b + 4.0 * c - d) * t2
                    + (-a + 3.0 * b - 3.0 * c + d) * t3)
            };
            out.push(pos2(
                blend(p0.x, p1.x, p2.x, p3.x),
                blend(p0.y, p1.y, p2.y, p3.y),
            ));
        }
    }
    out.extend(points.last());
    out
}

/// Draws the path inside `view`.
pub fn show(ui: &mut Ui, view: Rect, state: &MovieState) {
    let map = |(x, y): (f32, f32)| {
        pos2(
            view.left() + view.width() * x,
            view.top() + view.height() * y,
        )
    };
    let points: Vec<Pos2> = POINTS.iter().map(|point| map(*point)).collect();
    let painter = ui.painter().with_clip_rect(view);
    let path = spline(&points, 16);
    painter.extend(Shape::dashed_line(
        &path,
        Stroke::new(2.0, colors::with_alpha(colors::TERTIARY, 220)),
        8.0,
        6.0,
    ));
    for (index, point) in points.iter().enumerate() {
        if index == 2 {
            continue;
        }
        painter.circle_filled(*point, 5.0, colors::SURFACE_LOWEST);
        painter.circle_stroke(*point, 5.0, Stroke::new(2.0, colors::PRIMARY));
    }
    let active = points[2];
    let handle_a = active + vec2(-50.0, -74.0);
    let handle_b = active + vec2(50.0, 75.0);
    painter.line_segment(
        [handle_a, handle_b],
        Stroke::new(1.0, colors::with_alpha(colors::PRIMARY, 150)),
    );
    painter.circle_filled(handle_a, 3.0, colors::PRIMARY);
    painter.circle_filled(handle_b, 3.0, colors::PRIMARY);
    painter.circle_filled(active, 9.0, colors::SURFACE_LOWEST);
    painter.circle_stroke(active, 9.0, Stroke::new(3.0, colors::PRIMARY));
    text::paint_text(
        &painter,
        active + vec2(18.0, -14.0),
        eframe::egui::Align2::LEFT_CENTER,
        &format!("CAM_ACTIVE [{}]", timecode(state.time)),
        typography::mono_medium(11.0),
        colors::TERTIARY,
    );
    text::paint_text(
        &painter,
        active + vec2(18.0, 4.0),
        eframe::egui::Align2::LEFT_CENTER,
        tr("movie.path_fov"),
        typography::mono(10.0),
        colors::ON_SURFACE_VARIANT,
    );
    let first = points[0];
    text::paint_text(
        &painter,
        first + vec2(12.0, 0.0),
        eframe::egui::Align2::LEFT_CENTER,
        &format!("KF #01 [{}]", "00:00"),
        typography::mono_medium(11.0),
        colors::PRIMARY,
    );
    let mut badge = ui.new_child(
        UiBuilder::new()
            .max_rect(Rect::from_min_size(
                pos2(view.right() - 330.0, view.bottom() - 36.0),
                vec2(318.0, 24.0),
            ))
            .layout(eframe::egui::Layout::right_to_left(
                eframe::egui::Align::Center,
            )),
    );
    Tag::new(tr("movie.cam_pos"), colors::OUTLINE)
        .part(" X: 14.82m  Y: -22.40m  ", colors::ON_SURFACE)
        .part("Z: 3.42m", colors::PRIMARY)
        .fill(colors::with_alpha(colors::SURFACE_LOWEST, 204))
        .padding(8.0, 4.0)
        .show(&mut badge);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn spline_passes_through_the_control_points() {
        let points = [pos2(0.0, 0.0), pos2(10.0, 5.0), pos2(20.0, 0.0)];
        let curve = spline(&points, 8);
        assert_eq!(curve[0], points[0]);
        assert_eq!(curve[8], points[1]);
        assert_eq!(*curve.last().unwrap(), points[2]);
    }
}
