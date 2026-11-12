//! A small line chart for telemetry series.

use crate::config::theme::colors;
use eframe::egui::{epaint::PathShape, Color32, Pos2, Rect, Shape, Stroke, Ui};

/// Maps `values` into points inside `rect`, scaled between the series minimum and maximum (with a flat series
/// drawn in the middle).
pub fn points(values: &[f32], rect: Rect) -> Vec<Pos2> {
    if values.len() < 2 {
        return Vec::new();
    }
    let low = values.iter().copied().fold(f32::INFINITY, f32::min);
    let high = values.iter().copied().fold(f32::NEG_INFINITY, f32::max);
    let span = (high - low).max(f32::EPSILON);
    let step = rect.width() / (values.len() - 1) as f32;
    values
        .iter()
        .enumerate()
        .map(|(index, value)| {
            let t = if (high - low).abs() < f32::EPSILON {
                0.5
            } else {
                (value - low) / span
            };
            Pos2::new(
                rect.left() + step * index as f32,
                rect.bottom() - t * rect.height(),
            )
        })
        .collect()
}

/// Draws the series as a line, optionally filled to the bottom edge with a translucent copy of the color.
pub fn show(ui: &mut Ui, rect: Rect, values: &[f32], color: Color32, fill: bool) {
    let points = points(values, rect);
    if points.is_empty() {
        return;
    }
    let painter = ui.painter().with_clip_rect(rect.expand(2.0));
    if fill {
        let mut area = points.clone();
        area.push(rect.right_bottom());
        area.push(rect.left_bottom());
        painter.add(Shape::Path(PathShape::convex_polygon(
            area,
            colors::with_alpha(color, 36),
            Stroke::NONE,
        )));
    }
    painter.add(Shape::line(points, Stroke::new(1.5, color)));
}

#[cfg(test)]
mod tests {
    use super::*;
    use eframe::egui::vec2;

    #[test]
    fn maps_extremes_to_edges() {
        let rect = Rect::from_min_size(Pos2::ZERO, vec2(100.0, 20.0));
        let pts = points(&[1.0, 3.0, 2.0], rect);
        assert_eq!(pts.len(), 3);
        assert_eq!(pts[0].y, 20.0);
        assert_eq!(pts[1].y, 0.0);
        assert_eq!(pts[2].x, 100.0);
    }

    #[test]
    fn short_series_yield_no_points() {
        assert!(points(&[1.0], Rect::ZERO).is_empty());
    }
}
