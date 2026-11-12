//! A catalog card: picture, badges, name, rating, technical readouts and the select/place button.

use super::tree_art;
use crate::components::{
    button::{Button, ButtonStyle},
    icons, paint,
    tag::Tag,
    text, text_layout,
};
use crate::config::theme::{colors, typography};
use crate::features::library::models::{AssetArt, AssetBadge, LibraryAsset};
use crate::services::localization::tr;
use crate::services::textures::{Placeholder, Textures};
use eframe::egui::{
    self, pos2, vec2, Align, CornerRadius, Layout, Rect, Sense, Ui, UiBuilder, Vec2,
};

/// Height of a card of the given width.
pub fn height_for(width: f32) -> f32 {
    width + 124.0
}

/// What the user did on a card.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CardAction {
    /// Open the asset inspector.
    Inspect,
    /// Make this the selected asset.
    Select,
    /// Place the selected asset in the scene.
    Place,
}

/// Draws the card; returns the action requested, if any.
pub fn show(
    ui: &mut Ui,
    width: f32,
    asset: &LibraryAsset,
    active: bool,
    textures: &mut Textures,
) -> Option<CardAction> {
    let (rect, response) = ui.allocate_exact_size(vec2(width, height_for(width)), Sense::click());
    let painter = ui.painter().clone();
    painter.rect_filled(rect, CornerRadius::same(2), colors::SURFACE_LOW);
    if active {
        paint::vertical_gradient(
            &painter.with_clip_rect(rect),
            rect,
            colors::with_alpha(colors::SURFACE_HIGH, 102),
            colors::SURFACE_LOW,
        );
        painter.rect_filled(
            Rect::from_min_size(rect.min, vec2(width, 2.0)),
            CornerRadius::same(2),
            colors::PRIMARY_CONTAINER,
        );
    }
    let image = Rect::from_min_size(rect.min + vec2(4.0, 4.0), Vec2::splat(width - 8.0));
    match asset.art {
        AssetArt::ElmRender => {
            let texture = textures.get(ui.ctx(), Placeholder::ElmSpecimen).clone();
            paint::image_cover(
                &painter.with_clip_rect(image),
                image,
                &texture,
                vec2(0.5, 0.5),
                egui::Color32::WHITE,
            );
        }
        AssetArt::Tree(style) => tree_art::paint_tree(&painter, image, &style, asset.height_m),
    }
    overlays(ui, image, asset, active);

    let mut body = ui.new_child(
        UiBuilder::new()
            .max_rect(Rect::from_min_max(
                pos2(rect.left() + 4.0, image.bottom() + 4.0),
                pos2(rect.right() - 4.0, rect.bottom() - 4.0),
            ))
            .layout(Layout::top_down(Align::Min)),
    );
    body.spacing_mut().item_spacing.y = 2.0;
    crate::components::layout::split_row(
        &mut body,
        22.0,
        |ui| {
            text::label(
                ui,
                asset.name,
                typography::headline_sm(),
                colors::ON_SURFACE,
            );
        },
        |ui| {
            Tag::new(format!("{:.1}", asset.rating), colors::SECONDARY)
                .lead_icon(icons::STAR, 13.0, colors::SECONDARY)
                .gap(2.0)
                .padding(0.0, 0.0)
                .show(ui);
        },
    );
    text_layout::clamped(
        &mut body,
        asset.species,
        typography::body_sm(),
        colors::OUTLINE,
        width - 8.0,
        1,
        16.0,
    );
    body.add_space(2.0);
    stats(&mut body, asset);

    let button_rect = Rect::from_min_size(
        pos2(rect.left() + 4.0, rect.bottom() - 4.0 - 28.0),
        vec2(width - 8.0, 28.0),
    );
    let mut button_ui = ui.new_child(UiBuilder::new().max_rect(button_rect));
    let (label, icon, style) = if active {
        (tr("library.place"), icons::TOUCH, ButtonStyle::Solid)
    } else {
        (tr("library.select"), icons::ADD_CIRCLE, ButtonStyle::Raised)
    };
    let mut button = Button::new(label, style)
        .icon(icon, 16.0)
        .width(width - 8.0)
        .height(28.0);
    if active {
        button = button.colors(colors::PRIMARY_CONTAINER, colors::ON_PRIMARY_CONTAINER);
    }
    if button.show(&mut button_ui).clicked() {
        return Some(if active {
            CardAction::Place
        } else {
            CardAction::Select
        });
    }
    response.clicked().then_some(CardAction::Inspect)
}

/// Badges (top-right) and the scale label (bottom-left) over the picture.
fn overlays(ui: &mut Ui, image: Rect, asset: &LibraryAsset, active: bool) {
    let mut top = ui.new_child(
        UiBuilder::new()
            .max_rect(image.shrink(4.0))
            .layout(Layout::right_to_left(Align::Min)),
    );
    top.spacing_mut().item_spacing.x = 4.0;
    Tag::new("L26.0", colors::ON_SURFACE)
        .fill(colors::SURFACE_HIGHEST)
        .padding(4.0, 2.0)
        .show(&mut top);
    let (label, fill, ink) = if active {
        (
            "library.badge_active",
            colors::PRIMARY_CONTAINER,
            colors::ON_PRIMARY_CONTAINER,
        )
    } else if asset.badge == AssetBadge::New {
        (
            "library.badge_new",
            colors::hex(0x2fb89f),
            colors::hex(0x004338),
        )
    } else {
        ("library.badge_hd", colors::SURFACE_HIGH, colors::OUTLINE)
    };
    Tag::new(text::caps(tr(label)), ink)
        .fill(fill)
        .padding(6.0, 2.0)
        .show(&mut top);

    let mut bottom = ui.new_child(
        UiBuilder::new()
            .max_rect(image.shrink(4.0))
            .layout(Layout::bottom_up(Align::Min)),
    );
    let ink = if active {
        colors::SECONDARY
    } else {
        colors::ON_SURFACE_VARIANT
    };
    Tag::new(asset.scale, ink)
        .lead_icon(icons::ACCESSIBILITY, 12.0, colors::OUTLINE)
        .gap(4.0)
        .fill(colors::with_alpha(colors::SURFACE_LOWEST, 204))
        .padding(4.0, 2.0)
        .show(&mut bottom);
}

/// The two-column readout under the name.
fn stats(ui: &mut Ui, asset: &LibraryAsset) {
    let (rect, _) = ui.allocate_exact_size(vec2(ui.available_width(), 38.0), Sense::hover());
    ui.painter()
        .rect_filled(rect, CornerRadius::same(2), colors::SURFACE_CONTAINER);
    let inner = rect.shrink(4.0);
    let cell = vec2((inner.width() - 4.0) / 2.0, 30.0);
    let height = format!("{:.1} m", asset.height_m);
    let cells = [
        (tr("library.height"), height.as_str(), colors::ON_SURFACE),
        (asset.stat_label, asset.stat_value, colors::TERTIARY),
    ];
    for (index, (caption, value, color)) in cells.into_iter().enumerate() {
        let origin = inner.min + vec2(index as f32 * (cell.x + 4.0), 0.0);
        let mut child = ui.new_child(UiBuilder::new().max_rect(Rect::from_min_size(origin, cell)));
        crate::components::label_value::show(&mut child, caption, colors::OUTLINE, value, color);
    }
}
