//! Collapsible panels docked over the 3D view: a right-hand column and a bottom drawer. Each has a slim header
//! with its title and a collapse button; collapsed, only the header (or a narrow strip) stays on screen. The
//! panel swallows pointer input over its whole area so drags never reach the 3D view underneath.

use super::{icon_button::IconButton, icons, text};
use crate::config::theme::{colors, typography};
use crate::services::localization::tr;
use eframe::egui::{
    pos2, vec2, Align, CornerRadius, Id, Layout, Rect, Sense, Stroke, StrokeKind, Ui, UiBuilder,
};

/// Height of a panel header in points.
pub const HEADER: f32 = 32.0;
/// Width of a collapsed right panel in points.
pub const COLLAPSED_WIDTH: f32 = 36.0;

/// Which edge of the view a panel is docked to.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Edge {
    /// A column along the right edge.
    Right,
    /// A column along the left edge.
    Left,
    /// A drawer along the bottom edge.
    Bottom,
}

/// A docked panel.
pub struct DockPanel<'a> {
    id: &'a str,
    title: &'a str,
    glyph: char,
    edge: Edge,
}

impl<'a> DockPanel<'a> {
    /// A panel with a unique `id`, a header `title` and icon, docked to `edge`.
    pub fn new(id: &'a str, title: &'a str, glyph: char, edge: Edge) -> Self {
        Self {
            id,
            title,
            glyph,
            edge,
        }
    }

    /// Space the panel takes along its docking axis: `size` when open, otherwise the collapsed size.
    pub fn extent(edge: Edge, open: bool, size: f32) -> f32 {
        match (edge, open) {
            (_, true) => size,
            (Edge::Right | Edge::Left, false) => COLLAPSED_WIDTH,
            (Edge::Bottom, false) => HEADER,
        }
    }

    /// Draws the panel into `area`; `body` fills the space under the header while `open`.
    pub fn show(self, ui: &mut Ui, area: Rect, open: &mut bool, body: impl FnOnce(&mut Ui)) {
        ui.interact(area, Id::new(("dock", self.id)), Sense::click_and_drag());
        let painter = ui.painter();
        painter.rect_filled(
            area,
            CornerRadius::ZERO,
            colors::with_alpha(colors::SURFACE_LOW, 238),
        );
        painter.rect_stroke(
            area,
            CornerRadius::ZERO,
            Stroke::new(1.0, colors::SURFACE_HIGHEST),
            StrokeKind::Inside,
        );
        if self.edge != Edge::Bottom && !*open {
            self.strip(ui, area, open);
            return;
        }
        let header = Rect::from_min_size(area.min, vec2(area.width(), HEADER));
        self.header(ui, header, open);
        if !*open {
            return;
        }
        let body_rect = Rect::from_min_max(pos2(area.left(), header.bottom()), area.max);
        let mut child = ui.new_child(
            UiBuilder::new()
                .id_salt(("dock-body", self.id))
                .max_rect(body_rect)
                .layout(Layout::top_down(Align::Min)),
        );
        child.set_clip_rect(body_rect.intersect(ui.clip_rect()));
        body(&mut child);
    }

    fn header(&self, ui: &mut Ui, header: Rect, open: &mut bool) {
        ui.painter()
            .rect_filled(header, CornerRadius::ZERO, colors::SURFACE_CONTAINER);
        let mut row = ui.new_child(
            UiBuilder::new()
                .id_salt(("dock-header", self.id))
                .max_rect(header.shrink2(vec2(8.0, 0.0)))
                .layout(Layout::left_to_right(Align::Center)),
        );
        row.spacing_mut().item_spacing.x = 6.0;
        text::icon(&mut row, self.glyph, 15.0, colors::PRIMARY);
        text::label(
            &mut row,
            text::caps(self.title),
            typography::label_md(),
            colors::ON_SURFACE,
        );
        row.with_layout(Layout::right_to_left(Align::Center), |row| {
            row.spacing_mut().item_spacing.x = 4.0;
            let (glyph, tooltip) = match (self.edge, *open) {
                (Edge::Right, _) => (icons::CHEVRON_RIGHT, "dock.collapse"),
                (Edge::Left, _) => (icons::CHEVRON_LEFT, "dock.collapse"),
                (Edge::Bottom, true) => (icons::EXPAND_MORE, "dock.collapse"),
                (Edge::Bottom, false) => (icons::EXPAND_LESS, "dock.expand"),
            };
            if IconButton::new(glyph)
                .sizes(16.0, 24.0)
                .fill(None)
                .tooltip(tr(tooltip))
                .show(row)
                .clicked()
            {
                *open = !*open;
            }
        });
    }

    /// The collapsed right panel: an expand button and the panel icon.
    fn strip(&self, ui: &mut Ui, area: Rect, open: &mut bool) {
        let mut column = ui.new_child(
            UiBuilder::new()
                .id_salt(("dock-strip", self.id))
                .max_rect(area.shrink2(vec2(0.0, 4.0)))
                .layout(Layout::top_down(Align::Center)),
        );
        column.spacing_mut().item_spacing.y = 8.0;
        let expand = if self.edge == Edge::Left {
            icons::CHEVRON_RIGHT
        } else {
            icons::CHEVRON_LEFT
        };
        if IconButton::new(expand)
            .sizes(16.0, 28.0)
            .fill(None)
            .tooltip(tr("dock.expand"))
            .show(&mut column)
            .clicked()
        {
            *open = true;
        }
        text::icon(&mut column, self.glyph, 16.0, colors::OUTLINE).on_hover_text(self.title);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn collapsed_panels_keep_a_handle() {
        assert_eq!(DockPanel::extent(Edge::Right, true, 300.0), 300.0);
        assert_eq!(
            DockPanel::extent(Edge::Right, false, 300.0),
            COLLAPSED_WIDTH
        );
        assert_eq!(DockPanel::extent(Edge::Bottom, false, 200.0), HEADER);
    }
}
