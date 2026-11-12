//! Edge and corner resize zones for the frameless window.

use eframe::egui::{
    self, Context, CursorIcon, Id, Pos2, Rect, ResizeDirection, Sense, ViewportCommand,
};

const EDGE: f32 = 5.0;

/// The eight resize zones of a window of the given outer rectangle.
pub fn zones(screen: Rect) -> [(Rect, ResizeDirection, CursorIcon); 8] {
    let (min, max) = (screen.min, screen.max);
    let corner = |x: f32, y: f32| Rect::from_min_size(Pos2::new(x, y), egui::vec2(EDGE, EDGE));
    [
        (
            corner(min.x, min.y),
            ResizeDirection::NorthWest,
            CursorIcon::ResizeNorthWest,
        ),
        (
            corner(max.x - EDGE, min.y),
            ResizeDirection::NorthEast,
            CursorIcon::ResizeNorthEast,
        ),
        (
            corner(min.x, max.y - EDGE),
            ResizeDirection::SouthWest,
            CursorIcon::ResizeSouthWest,
        ),
        (
            corner(max.x - EDGE, max.y - EDGE),
            ResizeDirection::SouthEast,
            CursorIcon::ResizeSouthEast,
        ),
        (
            Rect::from_min_max(min, Pos2::new(max.x, min.y + EDGE)),
            ResizeDirection::North,
            CursorIcon::ResizeVertical,
        ),
        (
            Rect::from_min_max(Pos2::new(min.x, max.y - EDGE), max),
            ResizeDirection::South,
            CursorIcon::ResizeVertical,
        ),
        (
            Rect::from_min_max(min, Pos2::new(min.x + EDGE, max.y)),
            ResizeDirection::West,
            CursorIcon::ResizeHorizontal,
        ),
        (
            Rect::from_min_max(Pos2::new(max.x - EDGE, min.y), max),
            ResizeDirection::East,
            CursorIcon::ResizeHorizontal,
        ),
    ]
}

/// Installs the resize zones unless the window is maximized.
pub fn resize_edges(context: &Context) {
    if context.input(|input| input.viewport().maximized.unwrap_or(false)) {
        return;
    }
    let screen = context.content_rect();
    egui::Area::new(Id::new("resize-edges"))
        .order(egui::Order::Foreground)
        .fixed_pos(screen.min)
        .interactable(true)
        .show(context, |ui| {
            for (index, (rect, direction, cursor)) in zones(screen).into_iter().enumerate() {
                let response = ui
                    .interact(rect, Id::new(("resize-edge", index)), Sense::drag())
                    .on_hover_cursor(cursor);
                if response.drag_started() {
                    context.send_viewport_cmd(ViewportCommand::BeginResize(direction));
                }
            }
        });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn eight_zones_stay_inside_the_window() {
        let screen = Rect::from_min_size(Pos2::ZERO, egui::vec2(800.0, 600.0));
        let zones = zones(screen);
        assert_eq!(zones.len(), 8);
        for (rect, _, _) in zones {
            assert!(screen.contains_rect(rect));
        }
    }
}
