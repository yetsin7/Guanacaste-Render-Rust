//! The three big dimension cards under the preview.

use crate::components::{icons, text};
use crate::config::theme::{colors, typography};
use crate::features::library::models::AssetDetails;
use crate::services::localization::tr;
use eframe::egui::{pos2, vec2, Align, Color32, CornerRadius, Layout, Rect, Sense, Ui, UiBuilder};

/// Draws the three cards sharing the available width.
pub fn show(ui: &mut Ui, details: &AssetDetails) {
    let cards = [
        (
            "inspector.height",
            details.height,
            "m",
            "inspector.height_note",
            icons::SWAP_VERT,
            colors::PRIMARY,
            colors::ON_SURFACE_VARIANT,
        ),
        (
            "inspector.spread",
            details.spread,
            "m",
            "inspector.spread_note",
            icons::SWAP_HORIZ,
            colors::SECONDARY,
            colors::ON_SURFACE_VARIANT,
        ),
        (
            "inspector.mesh",
            details.polygons,
            "Polígonos",
            "inspector.mesh_note",
            icons::TIMELINE,
            colors::TERTIARY,
            colors::hex(0x5cdbc0),
        ),
    ];
    let gap = 8.0;
    let width = ((ui.available_width() - gap * 2.0) / 3.0).floor();
    ui.horizontal_top(|ui| {
        ui.spacing_mut().item_spacing.x = gap;
        for (caption, value, unit, note, glyph, accent, note_color) in cards {
            card(
                ui,
                width,
                tr(caption),
                value,
                unit,
                tr(note),
                glyph,
                accent,
                note_color,
            );
        }
    });
}

#[allow(clippy::too_many_arguments)]
fn card(
    ui: &mut Ui,
    width: f32,
    caption: &str,
    value: &str,
    unit: &str,
    note: &str,
    glyph: char,
    accent: Color32,
    note_color: Color32,
) {
    let (rect, _) = ui.allocate_exact_size(vec2(width, 100.0), Sense::hover());
    ui.painter()
        .rect_filled(rect, CornerRadius::same(8), colors::SURFACE_LOW);
    let text_area = Rect::from_min_max(
        rect.min + vec2(12.0, 12.0),
        pos2(rect.right() - 12.0 - 56.0, rect.bottom() - 12.0),
    );
    let mut inner = ui.new_child(
        UiBuilder::new()
            .max_rect(text_area)
            .layout(Layout::top_down(Align::Min)),
    );
    inner.spacing_mut().item_spacing.y = 0.0;
    text::label(
        &mut inner,
        text::caps(caption),
        typography::label_sm(),
        colors::OUTLINE,
    );
    inner.add_space(2.0);
    inner.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 4.0;
        text::label(ui, value, typography::heading(24.0), colors::ON_SURFACE);
        text::label(ui, unit, typography::label_md(), accent);
    });
    inner.add_space(4.0);
    crate::components::text_layout::clamped(
        &mut inner,
        note,
        typography::body_sm(),
        note_color,
        text_area.width(),
        2,
        16.0,
    );
    let tile = Rect::from_center_size(
        pos2(rect.right() - 12.0 - 24.0, rect.center().y),
        vec2(48.0, 48.0),
    );
    ui.painter()
        .rect_filled(tile, CornerRadius::same(4), colors::SURFACE_HIGHEST);
    text::paint_icon(ui.painter(), tile.center(), glyph, 24.0, accent);
}
