//! Text and icon primitives. Everything is non-selectable and single-line unless stated otherwise.

use crate::config::theme::typography;
use eframe::egui::{
    self, Align2, Color32, FontId, Painter, Pos2, Rect, Response, RichText, Sense, TextWrapMode,
    Ui, Vec2,
};

/// A single-line, non-selectable label.
pub fn label(ui: &mut Ui, text: impl Into<String>, font: FontId, color: Color32) -> Response {
    ui.add(
        egui::Label::new(RichText::new(text).font(font).color(color))
            .selectable(false)
            .wrap_mode(TextWrapMode::Extend),
    )
}

/// A wrapping paragraph with an explicit line height.
pub fn paragraph(
    ui: &mut Ui,
    text: impl Into<String>,
    font: FontId,
    color: Color32,
    line_height: f32,
) -> Response {
    ui.add(
        egui::Label::new(
            RichText::new(text)
                .font(font)
                .color(color)
                .line_height(Some(line_height)),
        )
        .selectable(false)
        .wrap_mode(TextWrapMode::Wrap),
    )
}

/// Paints one icon glyph at its natural size and returns its response.
pub fn icon(ui: &mut Ui, glyph: char, size: f32, color: Color32) -> Response {
    let (rect, response) = ui.allocate_exact_size(Vec2::splat(size), Sense::hover());
    paint_icon(ui.painter(), rect.center(), glyph, size, color);
    response
}

/// Paints an icon glyph centered on `center`.
pub fn paint_icon(painter: &Painter, center: Pos2, glyph: char, size: f32, color: Color32) {
    painter.text(
        center,
        Align2::CENTER_CENTER,
        glyph,
        typography::icon(size),
        color,
    );
}

/// Paints text anchored at `pos` and returns the rectangle it covers.
pub fn paint_text(
    painter: &Painter,
    pos: Pos2,
    anchor: Align2,
    text: &str,
    font: FontId,
    color: Color32,
) -> Rect {
    painter.text(pos, anchor, text, font, color)
}

/// Width in points of `text` laid out on one line.
pub fn text_width(ui: &Ui, text: &str, font: &FontId) -> f32 {
    ui.painter()
        .layout_no_wrap(text.to_owned(), font.clone(), Color32::WHITE)
        .size()
        .x
}

/// Uppercases a localized string for technical labels, which the design sets in capitals.
pub fn caps(text: &str) -> String {
    text.to_uppercase()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn caps_handles_accents() {
        assert_eq!(caps("Construcción"), "CONSTRUCCIÓN");
    }
}
