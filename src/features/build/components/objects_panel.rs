//! The objects panel the shell embeds: `panel(ui, &mut ObjectsState)`.

use super::{objects_browser, objects_selection, objects_tools};
use crate::components::{section_title, text};
use crate::config::theme::{colors, typography};
use crate::features::build::models::ObjectsState;
use crate::services::localization::{tr, trf};
use eframe::egui::{ScrollArea, Ui};

/// Draws the whole objects panel: library, active asset, mode and options, selection, edit and mass tools.
pub fn panel(ui: &mut Ui, state: &mut ObjectsState) {
    if let Some(path) = state.picker.poll() {
        state.message = state
            .import_model(&path)
            .err()
            .map(|_| "objects.import_failed");
    }
    ScrollArea::vertical()
        .auto_shrink([false, false])
        .show(ui, |ui| {
            let count = state.doc.instances.len().to_string();
            section_title::with_count(
                ui,
                tr("objects.title"),
                Some(&trf("objects.count", &[&count])),
            );
            ui.add_space(6.0);
            objects_browser::show(ui, state);
            ui.add_space(8.0);
            active_line(ui, state);
            objects_tools::mode_bar(ui, state);
            ui.add_space(6.0);
            objects_tools::options(ui, state);
            ui.add_space(8.0);
            objects_selection::show(ui, state);
            ui.add_space(6.0);
            objects_tools::edit_bar(ui, state);
            ui.add_space(6.0);
            objects_tools::selection_tools(ui, state);
            ui.add_space(8.0);
            objects_tools::mass_tools(ui, state);
            if let Some(key) = state.message {
                text::label(ui, tr(key), typography::body_sm(), colors::ERROR);
            }
        });
}

/// The active asset with its variation seed controls.
fn active_line(ui: &mut Ui, state: &mut ObjectsState) {
    let Some(active) = state.browser.active else {
        text::label(
            ui,
            tr("objects.no_active"),
            typography::body_sm(),
            colors::OUTLINE,
        );
        return;
    };
    let name = crate::features::library::logic::asset_search::entries(&state.doc.imported_assets)
        .into_iter()
        .find(|e| e.asset.with_seed(0) == active.with_seed(0))
        .map(|e| e.name)
        .unwrap_or_default();
    text::label(
        ui,
        trf("objects.active", &[&name]),
        typography::label_md(),
        colors::ON_SURFACE,
    );
    if matches!(active, crate::features::library::models::AssetRef::Procedural { kind, .. } if kind.varies())
    {
        ui.horizontal(|ui| {
            text::label(
                ui,
                tr("objects.seed"),
                typography::body_sm(),
                colors::ON_SURFACE_VARIANT,
            );
            let mut seed = state.browser.seed;
            ui.add(eframe::egui::DragValue::new(&mut seed).range(0..=999_999));
            if crate::components::button::Button::new(
                tr("objects.seed_next"),
                crate::components::button::ButtonStyle::Quiet,
            )
            .show(ui)
            .clicked()
            {
                seed = (seed + 1) % 1_000_000;
            }
            if seed != state.browser.seed {
                state.browser.seed = seed;
                state.browser.active = Some(active.with_seed(seed));
            }
        });
    }
}
