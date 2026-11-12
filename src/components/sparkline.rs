//! Line charts for telemetry series: solid or dashed lines with an optional gradient fill under them.

use eframe::egui::{epaint::Mesh, Color32, Pos2, Rect, Shape, Stroke, Ui};

/// How a series is drawn.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SeriesStyle {
    /// Line color.
    pub color: Color32,
    /// Fills the area under the line with a fading copy of the color.
    pub fill: bool,
    /// Draws the line dashed.
    pub dashed: bool,
    /// Line thickness.
    pub width: f32,
}

impl SeriesStyle {
    /// A solid 1.5 px line without fill.
    pub fn line(color: Color32) -> Self {
        Self {
            color,
            fill: false,
            dashed: false,
            width: 1.5,
        }
    }
}

/// `(min, max)` of a series; a flat series gets a symmetric unit range.
pub fn range_of(values: &[f32]) -> (f32, f32) {
    let low = values.iter().copied().fold(f32::INFINITY, f32::min);
    let high = values.iter().copied().fold(f32::NEG_INFINITY, f32::max);
    if (high - low).abs() < f32::EPSILON {
        (low - 1.0, high + 1.0)
    } else {
        (low, high)
    }
}

/// Maps `values` into points inside `rect`, with `range.0` at the bottom edge and `range.1` at the top.
pub fn points_in(values: &[f32], rect: Rect, range: (f32, f32)) -> Vec<Pos2> {
    if values.len() < 2 {
        return Vec::new();
    }
    let span = (range.1 - range.0).max(f32::EPSILON);
    let step = rect.width() / (values.len() - 1) as f32;
    values
        .iter()
        .enumerate()
        .map(|(index, value)| {
            let t = ((value - range.0) / span).clamp(0.0, 1.0);
            Pos2::new(
                rect.left() + step * index as f32,
                rect.bottom() - t * rect.height(),
            )
        })
        .collect()
}

/// Maps `values` into `rect` scaled between the series' own minimum and maximum.
pub fn points(values: &[f32], rect: Rect) -> Vec<Pos2> {
    points_in(values, rect, range_of(values))
}

/// Draws one series of `values` in `rect`, scaled to `range` (or to its own extremes when `None`).
pub fn show_series(
    ui: &mut Ui,
    rect: Rect,
    values: &[f32],
    range: Option<(f32, f32)>,
    style: SeriesStyle,
) {
    let points = points_in(values, rect, range.unwrap_or_else(|| range_of(values)));
    if points.is_empty() {
        return;
    }
    let painter = ui.painter().with_clip_rect(rect.expand(2.0));
    if style.fill {
        let mut mesh = Mesh::default();
        let top =
            Color32::from_rgba_unmultiplied(style.color.r(), style.color.g(), style.color.b(), 60);
        let bottom =
            Color32::from_rgba_unmultiplied(style.color.r(), style.color.g(), style.color.b(), 0);
        for point in &points {
            mesh.colored_vertex(*point, top);
            mesh.colored_vertex(Pos2::new(point.x, rect.bottom()), bottom);
        }
        for index in 0..points.len() as u32 - 1 {
            let base = index * 2;
            mesh.add_triangle(base, base + 1, base + 2);
            mesh.add_triangle(base + 1, base + 3, base + 2);
        }
        painter.add(Shape::mesh(mesh));
    }
    let stroke = Stroke::new(style.width, style.color);
    if style.dashed {
        painter.extend(Shape::dashed_line(&points, stroke, 4.0, 3.0));
    } else {
        painter.add(Shape::line(points, stroke));
    }
}

/// Draws a plain solid line scaled to its own extremes.
pub fn show(ui: &mut Ui, rect: Rect, values: &[f32], color: Color32, fill: bool) {
    show_series(
        ui,
        rect,
        values,
        None,
        SeriesStyle {
            fill,
            ..SeriesStyle::line(color)
        },
    );
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

    #[test]
    fn shared_range_keeps_series_comparable() {
        let rect = Rect::from_min_size(Pos2::ZERO, vec2(10.0, 10.0));
        let low = points_in(&[0.0, 1.0], rect, (0.0, 4.0));
        assert_eq!(low[1].y, 7.5);
    }
}
