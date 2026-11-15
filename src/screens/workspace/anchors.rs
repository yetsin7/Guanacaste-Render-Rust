//! Where floating cards over the 3D view may sit without covering the tool palette, which owns the top-left
//! corner of the free area in every mode.

use eframe::egui::{pos2, Pos2, Rect};

/// Margin between the palette corner and the edge of the free area, and between stacked floating items.
pub const GAP: f32 = 12.0;

/// Height reserved for the palette on the first frame, before its real size is known.
const PALETTE_GUESS_HEIGHT: f32 = 104.0;

/// Top-left corner for a card that must stay clear of the palette: under it, aligned with its left edge.
/// `palette` is the palette's rectangle from the previous frame (`None` on the first frame).
pub fn below_palette(free: Rect, palette: Option<Rect>) -> Pos2 {
    let left = palette.map_or(free.left() + GAP, |r| r.left());
    let top = palette.map_or(free.top() + GAP + PALETTE_GUESS_HEIGHT, |r| r.bottom()) + GAP * 0.5;
    pos2(left, top.min(free.bottom() - GAP))
}

#[cfg(test)]
mod tests {
    use super::*;
    use eframe::egui::vec2;

    fn free() -> Rect {
        Rect::from_min_size(pos2(0.0, 100.0), vec2(900.0, 600.0))
    }

    fn clear_of(origin: Pos2, palette: Rect) -> bool {
        !Rect::from_min_size(origin, vec2(320.0, 250.0)).intersects(palette)
    }

    #[test]
    fn a_card_below_the_palette_never_overlaps_it() {
        let palette = Rect::from_min_size(pos2(12.0, 110.0), vec2(290.0, 110.0));
        let origin = below_palette(free(), Some(palette));
        assert!(clear_of(origin, palette));
        assert_eq!(origin.x, 12.0);
    }

    #[test]
    fn before_the_palette_is_measured_the_estimate_is_used() {
        // The palette sits 12 pt from the left and 10 pt from the top and is about 288 x 104 pt.
        let palette = Rect::from_min_size(pos2(12.0, 110.0), vec2(288.0, 104.0));
        assert!(clear_of(below_palette(free(), None), palette));
    }

    #[test]
    fn the_card_stays_inside_a_short_view() {
        let short = Rect::from_min_size(pos2(0.0, 0.0), vec2(900.0, 80.0));
        assert!(below_palette(short, None).y <= short.bottom());
    }
}
