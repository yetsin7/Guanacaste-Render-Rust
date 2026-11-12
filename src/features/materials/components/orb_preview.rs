//! The shader orb preview card: a procedurally shaded concrete sphere.

use crate::components::{layout::split_row, paint, tag::Tag, text};
use crate::config::theme::{colors, typography};
use crate::services::localization::tr;
use eframe::egui::{vec2, Align, CornerRadius, Layout, Pos2, Rect, Sense, Ui, UiBuilder};

/// Side of the card in points.
pub const SIDE: f32 = 176.0;

/// Draws the card with its top-left corner at `origin`.
pub fn show(ui: &mut Ui, origin: Pos2) {
    let rect = Rect::from_min_size(origin, vec2(SIDE, SIDE));
    ui.painter().rect_filled(
        rect,
        CornerRadius::same(4),
        colors::with_alpha(colors::SURFACE_LOW, 242),
    );
    let mut inner = ui.new_child(
        UiBuilder::new()
            .max_rect(rect.shrink(8.0))
            .layout(Layout::top_down(Align::Min)),
    );
    inner.spacing_mut().item_spacing.y = 4.0;
    split_row(
        &mut inner,
        28.0,
        |ui| {
            text::label(
                ui,
                text::caps(tr("material.orb")),
                typography::mono(10.0),
                colors::ON_SURFACE_VARIANT,
            );
        },
        |ui| {
            text::label(ui, "PBR v3.4", typography::mono(10.0), colors::PRIMARY);
        },
    );
    let (stage, _) = inner.allocate_exact_size(
        vec2(inner.available_width(), inner.available_height()),
        Sense::hover(),
    );
    ui.painter()
        .rect_filled(stage, CornerRadius::same(2), colors::SURFACE_LOWEST);
    paint_sphere(ui, stage.center(), 56.0);
    let mut badge = ui.new_child(
        UiBuilder::new()
            .max_rect(stage.shrink(4.0))
            .layout(Layout::bottom_up(Align::Max)),
    );
    Tag::new(tr("material.scale"), colors::TERTIARY)
        .font(typography::mono(10.0))
        .padding(4.0, 0.0)
        .fill(colors::with_alpha(colors::SURFACE_LOWEST, 204))
        .show(&mut badge);
}

/// A speckled grey sphere lit from the upper left.
fn paint_sphere(ui: &Ui, center: Pos2, radius: f32) {
    let painter = ui.painter();
    let base = colors::hex(0xb9b5ae);
    painter.circle_filled(center, radius, base);
    for step in 0..14 {
        let t = step as f32 / 14.0;
        let offset = vec2(radius * 0.35 * t, radius * 0.35 * t);
        let shade = paint::mix(colors::hex(0xe6e2da), colors::hex(0x2a2b2e), t);
        painter.circle_filled(
            center - vec2(radius * 0.3, radius * 0.3) + offset,
            radius * (1.0 - t * 0.65),
            colors::with_alpha(shade, 26),
        );
    }
    let mut seed = 7_u32;
    for _ in 0..90 {
        seed = seed.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
        let angle = (seed >> 8) as f32 / 16_777_216.0 * std::f32::consts::TAU;
        seed = seed.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
        let distance = ((seed >> 8) as f32 / 16_777_216.0).sqrt() * radius * 0.92;
        let point = center + vec2(angle.cos(), angle.sin()) * distance;
        painter.circle_filled(point, 0.9, colors::with_alpha(colors::hex(0x4b4a48), 150));
    }
    painter.circle_stroke(
        center,
        radius,
        eframe::egui::Stroke::new(1.0, colors::with_alpha(colors::PRIMARY, 40)),
    );
}
