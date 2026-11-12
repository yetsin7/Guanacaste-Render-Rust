//! The full keyboard shortcut reference with search.

use crate::components::{
    form, icons, key_cap::KeyCap, layout::split_row, modal, search_field::SearchField, text,
};
use crate::config::theme::{colors, typography};
use crate::features::help::logic::filter_shortcuts;
use crate::features::help::models::Shortcut;
use crate::models::dialog::DialogOutcome;
use crate::services::localization::tr;
use crate::services::sample_data::help as sample;
use eframe::egui::{Context, ScrollArea, Ui};

fn group(ui: &mut Ui, title: &'static str, shortcuts: &[Shortcut], search: &str) {
    let rows = filter_shortcuts(shortcuts, search);
    if rows.is_empty() {
        return;
    }
    form::caption(ui, tr(title));
    for shortcut in rows {
        split_row(
            ui,
            26.0,
            |ui| {
                text::label(
                    ui,
                    shortcut.action,
                    typography::body_md(),
                    colors::ON_SURFACE,
                );
            },
            |ui| {
                KeyCap::new(shortcut.keys)
                    .colors(colors::SURFACE_HIGHEST, colors::PRIMARY)
                    .font(typography::mono_medium(11.0))
                    .show(ui);
            },
        );
    }
}

/// Draws the dialog; `search` is its filter text.
pub fn show(ctx: &Context, search: &mut String) -> DialogOutcome {
    let app = sample::app_shortcuts();
    let view: Vec<Shortcut> = sample::placement_shortcuts()
        .into_iter()
        .chain(sample::transform_shortcuts())
        .chain(sample::navigation_shortcuts())
        .collect();
    let closed = modal::show(
        ctx,
        "shortcuts-dialog",
        tr("shortcuts.title"),
        icons::KEYBOARD,
        560.0,
        |ui| {
            modal::content(ui, |ui| {
                SearchField::new(tr("shortcuts.search"))
                    .width(528.0)
                    .height(28.0)
                    .font(typography::body_sm(), 14.0)
                    .show(ui, search);
                let none = filter_shortcuts(&app, search).is_empty()
                    && filter_shortcuts(&view, search).is_empty();
                if none {
                    text::label(
                        ui,
                        tr("shortcuts.empty"),
                        typography::body_sm(),
                        colors::OUTLINE,
                    );
                }
                ScrollArea::vertical()
                    .max_height(380.0)
                    .auto_shrink([false, true])
                    .show(ui, |ui| {
                        ui.spacing_mut().item_spacing.y = 6.0;
                        group(ui, "shortcuts.app", &app, search);
                        group(ui, "shortcuts.view", &view, search);
                    });
            });
        },
    );
    if closed {
        DialogOutcome::Close
    } else {
        DialogOutcome::Keep
    }
}
