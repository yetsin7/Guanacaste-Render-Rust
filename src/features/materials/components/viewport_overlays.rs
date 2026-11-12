//! Overlays of the material editor's 3D view: status, tools and mesh readouts.

use super::orb_preview;
use crate::components::{icon_button::IconButton, icons, tag::Tag, text};
use crate::config::theme::{colors, typography};
use crate::features::materials::models::MaterialState;
use crate::services::localization::tr;
use eframe::egui::{pos2, vec2, Align, CornerRadius, Layout, Pos2, Rect, Ui, UiBuilder, Vec2};

const TOOLS: [(char, &str); 4] = [
    (icons::ROTATE, "material.tool_orbit"),
    (icons::PAN_TOOL, "material.tool_pan"),
    (icons::COLORIZE, "material.tool_pick"),
    (icons::FOCUS, "material.tool_focus"),
];

/// Draws every overlay inside `area`.
pub fn show(ui: &mut Ui, area: Rect, state: &mut MaterialState) {
    let panel = colors::with_alpha(colors::SURFACE_LOW, 230);
    let inner = area.shrink(12.0);
    let mut status = ui.new_child(UiBuilder::new().max_rect(inner));
    Tag::new(tr("material.status"), colors::ON_SURFACE)
        .font(typography::mono_medium(11.0))
        .dot(colors::TERTIARY)
        .gap(8.0)
        .fill(panel)
        .padding(12.0, 4.0)
        .show(&mut status);

    let tools = Rect::from_min_size(
        pos2(inner.right() - 4.0 * 28.0 - 3.0 * 4.0 - 8.0, inner.top()),
        vec2(4.0 * 28.0 + 3.0 * 4.0 + 8.0, 36.0),
    );
    ui.painter()
        .rect_filled(tools, CornerRadius::same(2), panel);
    let mut row = ui.new_child(
        UiBuilder::new()
            .max_rect(tools.shrink(4.0))
            .layout(Layout::left_to_right(Align::Center)),
    );
    row.spacing_mut().item_spacing.x = 4.0;
    for (index, (glyph, tip)) in TOOLS.into_iter().enumerate() {
        if IconButton::new(glyph)
            .sizes(16.0, 28.0)
            .fill(None)
            .active(state.tool == index)
            .tooltip(tr(tip))
            .show(&mut row)
            .clicked()
        {
            state.tool = index;
        }
    }

    let origin: Pos2 = pos2(inner.left(), inner.bottom() - orb_preview::SIDE);
    orb_preview::show(ui, origin);
    mesh_info(
        ui,
        pos2(origin.x + orb_preview::SIDE + 12.0, inner.bottom() - 52.0),
        panel,
    );
}

fn mesh_info(ui: &mut Ui, origin: Pos2, fill: eframe::egui::Color32) {
    let rect = Rect::from_min_size(origin, vec2(470.0, 52.0));
    ui.painter().rect_filled(rect, CornerRadius::same(2), fill);
    let cells = [
        (
            "material.mesh",
            "Facade_Panel_North_A_04",
            colors::ON_SURFACE,
            typography::label_md(),
        ),
        (
            "material.uv",
            "material.uv_value",
            colors::ON_SURFACE,
            typography::label_md(),
        ),
        (
            "material.samples",
            "144 / 256",
            colors::TERTIARY,
            typography::label_md(),
        ),
    ];
    let mut x = rect.left() + 12.0;
    for (index, (caption, value, color, font)) in cells.into_iter().enumerate() {
        let value = if value.starts_with("material.") {
            tr(value)
        } else {
            value
        };
        let caption_rect = text::paint_text(
            ui.painter(),
            pos2(x, rect.top() + 14.0),
            eframe::egui::Align2::LEFT_CENTER,
            &text::caps(tr(caption)),
            typography::mono(10.0),
            colors::OUTLINE,
        );
        let value_rect = text::paint_text(
            ui.painter(),
            pos2(x, rect.top() + 32.0),
            eframe::egui::Align2::LEFT_CENTER,
            value,
            font,
            color,
        );
        x += caption_rect.width().max(value_rect.width()) + 16.0;
        if index < 2 {
            ui.painter().rect_filled(
                Rect::from_min_size(pos2(x - 8.0, rect.top() + 14.0), Vec2::new(1.0, 24.0)),
                0.0,
                colors::SURFACE_HIGHEST,
            );
            x += 8.0;
        }
    }
}
