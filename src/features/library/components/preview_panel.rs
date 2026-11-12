//! The photometric preview with its overlays, and the title strip under it.

use crate::components::{
    button::{Button, ButtonStyle},
    icon_button::IconButton,
    icons,
    layout::split_row,
    paint,
    tag::Tag,
    text,
};
use crate::config::theme::{colors, typography};
use crate::features::library::models::{AssetDetails, InspectorState};
use crate::services::localization::tr;
use crate::services::textures::{Placeholder, Textures};
use eframe::egui::{
    self, pos2, vec2, Align, CornerRadius, Layout, Rect, Sense, Ui, UiBuilder, Vec2,
};

/// Height of the preview area in points.
pub const PREVIEW_HEIGHT: f32 = 524.0;

/// What the user asked for in the panel.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PreviewAction {
    /// Place the asset in the scene.
    Place,
}

/// Draws the panel; returns the action requested, if any.
pub fn show(
    ui: &mut Ui,
    details: &AssetDetails,
    state: &mut InspectorState,
    textures: &mut Textures,
) -> Option<PreviewAction> {
    let width = ui.available_width();
    let (rect, _) = ui.allocate_exact_size(vec2(width, PREVIEW_HEIGHT), Sense::hover());
    let painter = ui.painter().clone();
    let preview = Rect::from_min_size(rect.min, vec2(width, PREVIEW_HEIGHT));
    painter.rect_filled(
        preview,
        CornerRadius {
            nw: 8,
            ne: 8,
            sw: 0,
            se: 0,
        },
        colors::SURFACE_LOWEST,
    );
    paint::vertical_gradient(
        &painter.with_clip_rect(preview),
        preview,
        colors::hex(0x131517),
        colors::hex(0x0a0b0d),
    );
    dot_grid(&painter.with_clip_rect(preview), preview);
    let side = (PREVIEW_HEIGHT * 0.8).min(640.0).min(width - 32.0);
    let image = Rect::from_center_size(preview.center(), Vec2::splat(side));
    let texture = textures.get(ui.ctx(), Placeholder::ElmSpecimen).clone();
    painter.image(
        texture.id(),
        image,
        Rect::from_min_max(pos2(0.0, 0.0), pos2(1.0, 1.0)),
        egui::Color32::WHITE,
    );
    overlays(ui, preview, state);

    let strip_rect = {
        let (strip, _) = ui.allocate_exact_size(vec2(width, 64.0), Sense::hover());
        strip
    };
    painter.rect_filled(
        strip_rect,
        CornerRadius {
            nw: 0,
            ne: 0,
            sw: 8,
            se: 8,
        },
        colors::SURFACE_LOW,
    );
    let mut action = None;
    let mut inner = ui.new_child(UiBuilder::new().max_rect(strip_rect.shrink2(vec2(16.0, 8.0))));
    split_row(
        &mut inner,
        48.0,
        |ui| {
            ui.vertical(|ui| {
                ui.spacing_mut().item_spacing.y = 2.0;
                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing.x = 8.0;
                    text::label(
                        ui,
                        details.name,
                        typography::headline_md(),
                        colors::ON_SURFACE,
                    );
                    Tag::new(details.grade, colors::PRIMARY)
                        .font(typography::mono_medium(11.0))
                        .fill(colors::with_alpha(colors::PRIMARY_CONTAINER, 51))
                        .padding(8.0, 2.0)
                        .show(ui);
                });
                text::label(
                    ui,
                    details.lineage,
                    typography::body_sm(),
                    colors::ON_SURFACE_VARIANT,
                );
            });
        },
        |ui| {
            ui.spacing_mut().item_spacing.x = 8.0;
            if Button::new(tr("inspector.place"), ButtonStyle::Pale)
                .icon(icons::ADD_CIRCLE, 16.0)
                .font(typography::body_md())
                .padding(12.0, 6.0)
                .show(ui)
                .clicked()
            {
                action = Some(PreviewAction::Place);
            }
            Button::new(tr("inspector.edit"), ButtonStyle::Raised)
                .icon(icons::EDIT_NOTE, 16.0)
                .font(typography::body_sm())
                .padding(8.0, 6.0)
                .show(ui);
        },
    );
    action
}

/// Faint dots on a 24 px grid.
fn dot_grid(painter: &egui::Painter, rect: Rect) {
    let color = colors::with_alpha(colors::PRIMARY_CONTAINER, 38);
    let mut y = rect.top() + 12.0;
    while y < rect.bottom() {
        let mut x = rect.left() + 12.0;
        while x < rect.right() {
            painter.circle_filled(pos2(x, y), 0.6, color);
            x += 24.0;
        }
        y += 24.0;
    }
}

fn overlays(ui: &mut Ui, preview: Rect, state: &mut InspectorState) {
    let inner = preview.shrink(16.0);
    let panel = colors::with_alpha(colors::SURFACE_LOW, 230);
    let mut top_left = ui.new_child(UiBuilder::new().max_rect(inner));
    Tag::new(text::caps(tr("inspector.preview")), colors::SECONDARY)
        .part(format!("   {}", tr("inspector.fov")), colors::OUTLINE)
        .dot(colors::SECONDARY)
        .gap(8.0)
        .fill(panel)
        .padding(12.0, 4.0)
        .show(&mut top_left);

    let group = Rect::from_min_size(pos2(inner.right() - 184.0, inner.top()), vec2(184.0, 40.0));
    painter_frame(ui, group, panel);
    let mut top_right = ui.new_child(
        UiBuilder::new()
            .max_rect(group.shrink(4.0))
            .layout(Layout::left_to_right(Align::Center)),
    );
    top_right.spacing_mut().item_spacing.x = 4.0;
    let button = |glyph, active| IconButton::new(glyph).sizes(18.0, 32.0).active(active);
    if button(icons::FAVORITE, state.favorite)
        .show(&mut top_right)
        .clicked()
    {
        state.favorite = !state.favorite;
    }
    button(icons::SHARE, false).show(&mut top_right);
    if button(icons::LOCK, state.locked)
        .show(&mut top_right)
        .clicked()
    {
        state.locked = !state.locked;
    }
    button(icons::COPY, false).show(&mut top_right);
    IconButton::new(icons::TARGET)
        .sizes(18.0, 32.0)
        .fill(Some(colors::SURFACE_HIGHEST))
        .show(&mut top_right);

    let scale = Rect::from_min_size(pos2(inner.left(), inner.bottom() - 70.0), vec2(270.0, 70.0));
    painter_frame(ui, scale, panel);
    let mut bottom_left = ui.new_child(
        UiBuilder::new()
            .max_rect(scale.shrink(12.0))
            .layout(Layout::top_down(Align::Min)),
    );
    bottom_left.spacing_mut().item_spacing.y = 4.0;
    {
        let ui = &mut bottom_left;
        split_row(
            ui,
            14.0,
            |ui| {
                text::label(
                    ui,
                    text::caps(tr("inspector.scale_ref")),
                    typography::label_sm(),
                    colors::OUTLINE,
                );
            },
            |ui| {
                text::label(
                    ui,
                    tr("inspector.biometric"),
                    typography::label_sm(),
                    colors::TERTIARY,
                );
            },
        );
        ui.horizontal(|ui| {
            let (bar, _) = ui.allocate_exact_size(vec2(60.0, 4.0), Sense::hover());
            ui.painter()
                .rect_filled(bar, CornerRadius::same(2), colors::SURFACE_HIGHEST);
            ui.painter().rect_filled(
                Rect::from_min_size(bar.min, vec2(bar.width() * 0.72, 4.0)),
                CornerRadius::same(2),
                colors::PRIMARY_CONTAINER,
            );
            text::label(
                ui,
                tr("inspector.human"),
                typography::label_sm(),
                colors::ON_SURFACE,
            );
        });
        text::label(
            ui,
            tr("inspector.grid_step"),
            typography::mono(10.0),
            colors::OUTLINE,
        );
    }

    let mut bottom_right = ui.new_child(
        UiBuilder::new()
            .max_rect(inner)
            .layout(Layout::bottom_up(Align::Max)),
    );
    Tag::new(tr("inspector.sun"), colors::ON_SURFACE)
        .part("  |  ", colors::OUTLINE)
        .part(tr("inspector.sky"), colors::SECONDARY)
        .lead_icon(icons::SUN, 16.0, colors::hex(0x7bf8dc))
        .gap(8.0)
        .fill(panel)
        .padding(12.0, 6.0)
        .show(&mut bottom_right);
}

fn painter_frame(ui: &Ui, rect: Rect, fill: egui::Color32) {
    ui.painter().rect_filled(rect, CornerRadius::same(2), fill);
}
