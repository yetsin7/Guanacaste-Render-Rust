//! Multi-part text layouts: clamped paragraphs and justified rows.

use eframe::egui::{
    text::{LayoutJob, TextFormat},
    Align2, Color32, FontId, Painter, Pos2, Rect, Response, Sense, Ui, Vec2,
};

/// A wrapping paragraph limited to `lines` lines (ellipsis when cut), `width` points wide.
pub fn clamped(
    ui: &mut Ui,
    text: &str,
    font: FontId,
    color: Color32,
    width: f32,
    lines: usize,
    line_height: f32,
) -> Response {
    let mut job = LayoutJob::single_section(
        text.to_owned(),
        TextFormat {
            font_id: font,
            color,
            line_height: Some(line_height),
            ..Default::default()
        },
    );
    job.wrap.max_width = width;
    job.wrap.max_rows = lines;
    job.wrap.overflow_character = Some('…');
    let galley = ui.painter().layout_job(job);
    let (rect, response) =
        ui.allocate_exact_size(Vec2::new(width, galley.size().y), Sense::hover());
    ui.painter().galley(rect.min, galley, color);
    response
}

/// Paints `parts` on one line inside `rect` with equal gaps between them (CSS `justify-content: space-between`).
pub fn paint_space_between(painter: &Painter, rect: Rect, parts: &[(&str, FontId, Color32)]) {
    let galleys: Vec<_> = parts
        .iter()
        .map(|(text, font, color)| painter.layout_no_wrap((*text).to_owned(), font.clone(), *color))
        .collect();
    let total: f32 = galleys.iter().map(|galley| galley.size().x).sum();
    let gap = if galleys.len() > 1 {
        ((rect.width() - total) / (galleys.len() - 1) as f32).max(0.0)
    } else {
        0.0
    };
    let mut x = rect.left();
    for galley in galleys {
        let width = galley.size().x;
        let pos = Pos2::new(x, rect.center().y - galley.size().y / 2.0);
        painter.galley(pos, galley, Color32::WHITE);
        x += width + gap;
    }
}

/// Paints a single run of text anchored left-center at `pos`.
pub fn paint_left(painter: &Painter, pos: Pos2, text: &str, font: FontId, color: Color32) -> Rect {
    painter.text(pos, Align2::LEFT_CENTER, text, font, color)
}
