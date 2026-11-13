//! The asset browser of the objects panel: search, category chips and a grid of CPU-rendered thumbnails.

use crate::components::{
    button::{Button, ButtonStyle},
    chip::Chip,
    search_field::SearchField,
    text,
};
use crate::config::theme::{colors, typography};
use crate::features::build::models::ObjectsState;
use crate::features::library::logic::asset_search::{self, AssetEntry};
use crate::features::library::models::{AssetCategory, AssetRef};
use crate::services::localization::{tr, trf};
use eframe::egui::{
    self, ColorImage, CornerRadius, Sense, Stroke, StrokeKind, TextureOptions, Ui, Vec2,
};

const THUMB: usize = 72;
const CARD: Vec2 = Vec2::new(84.0, 100.0);

/// The texture of `asset`'s thumbnail, rendering and uploading it the first time.
fn texture(ui: &Ui, state: &mut ObjectsState, asset: AssetRef) -> Option<egui::TextureHandle> {
    let key = (asset, THUMB);
    if let Some(handle) = state.textures.get(&key) {
        return Some(handle.clone());
    }
    let image = state.cache.thumbnail(asset, THUMB)?;
    let color = ColorImage::from_rgba_unmultiplied([image.size, image.size], &image.rgba);
    let handle = ui
        .ctx()
        .load_texture(format!("object-{asset:?}"), color, TextureOptions::LINEAR);
    state.textures.insert(key, handle.clone());
    Some(handle)
}

fn card(ui: &mut Ui, state: &mut ObjectsState, entry: &AssetEntry) {
    let selected = state.browser.active.is_some_and(|active| {
        std::mem::discriminant(&active) == std::mem::discriminant(&entry.asset)
            && active.with_seed(0) == entry.asset.with_seed(0)
    });
    let (rect, response) = ui.allocate_exact_size(CARD, Sense::click());
    let fill = if selected {
        colors::SURFACE_HIGHEST
    } else if response.hovered() {
        colors::SURFACE_HIGH
    } else {
        colors::SURFACE_CONTAINER
    };
    ui.painter().rect_filled(rect, CornerRadius::same(2), fill);
    if selected {
        ui.painter().rect_stroke(
            rect,
            CornerRadius::same(2),
            Stroke::new(1.0, colors::PRIMARY),
            StrokeKind::Inside,
        );
    }
    let preview = egui::Rect::from_min_size(
        rect.min + Vec2::new((CARD.x - THUMB as f32) / 2.0, 4.0),
        Vec2::splat(THUMB as f32),
    );
    let seeded = entry.asset.with_seed(state.browser.seed);
    match texture(ui, state, seeded) {
        Some(handle) => {
            egui::Image::new(&handle).paint_at(ui, preview);
        }
        None => text::paint_icon(
            ui.painter(),
            preview.center(),
            crate::components::icons::CLOSE,
            16.0,
            colors::OUTLINE,
        ),
    }
    ui.painter().text(
        egui::pos2(rect.center().x, rect.bottom() - 11.0),
        egui::Align2::CENTER_CENTER,
        &entry.name,
        typography::label_sm(),
        colors::ON_SURFACE,
    );
    if response.clicked() {
        state.browser.active = Some(entry.asset.with_seed(state.browser.seed));
        state.mode = crate::features::build::models::GizmoMode::Place;
    }
}

/// Draws the search box, the category chips, the thumbnail grid and the import button.
pub fn show(ui: &mut Ui, state: &mut ObjectsState) {
    SearchField::new(tr("objects.search_hint"))
        .width(ui.available_width())
        .show(ui, &mut state.browser.search);
    ui.add_space(6.0);
    ui.horizontal_wrapped(|ui| {
        ui.spacing_mut().item_spacing = Vec2::new(4.0, 4.0);
        let all = state.browser.category.is_none();
        if Chip::new(tr("objects.cat_all"))
            .selected(all)
            .show(ui)
            .clicked()
        {
            state.browser.category = None;
        }
        for category in AssetCategory::ALL {
            let selected = state.browser.category == Some(category);
            if Chip::new(tr(category.key()))
                .selected(selected)
                .show(ui)
                .clicked()
            {
                state.browser.category = Some(category);
            }
        }
    });
    ui.add_space(6.0);
    let entries = asset_search::entries(&state.doc.imported_assets);
    let shown = asset_search::filter(&entries, &state.browser.search, state.browser.category);
    if shown.is_empty() {
        text::label(
            ui,
            tr("objects.no_results"),
            typography::body_sm(),
            colors::OUTLINE,
        );
    }
    ui.horizontal_wrapped(|ui| {
        ui.spacing_mut().item_spacing = Vec2::splat(6.0);
        for entry in &shown {
            card(ui, state, entry);
        }
    });
    ui.add_space(6.0);
    if Button::new(tr("objects.import_model"), ButtonStyle::Raised)
        .fill_width()
        .show(ui)
        .clicked()
    {
        state.picker.open();
    }
    if !state.missing.is_empty() {
        let names = state.missing.join(", ");
        text::label(
            ui,
            trf("objects.missing_models", &[&names]),
            typography::body_sm(),
            colors::WARNING,
        );
    }
}
