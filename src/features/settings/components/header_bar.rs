//! The settings header: title, project chip, search, defaults and apply.

use crate::components::{
    button::{Button, ButtonStyle},
    icon_button::IconButton,
    icons,
    layout::split_row,
    search_field::SearchField,
    tag::Tag,
    text,
};
use crate::config::theme::{colors, typography};
use crate::features::settings::models::SettingsState;
use crate::services::localization::tr;
use eframe::egui::{self, Margin, Ui};

/// What the user asked for in the header.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HeaderAction {
    /// Restore defaults.
    Defaults,
    /// Apply the configuration.
    Apply,
}

/// Draws the header and returns the requested action, if any.
pub fn show(ui: &mut Ui, state: &mut SettingsState) -> Option<HeaderAction> {
    let mut defaults = false;
    let mut apply = false;
    egui::Frame::new()
        .fill(colors::SURFACE_LOW)
        .inner_margin(Margin::symmetric(16, 8))
        .show(ui, |ui| {
            split_row(
                ui,
                32.0,
                |ui| {
                    ui.spacing_mut().item_spacing.x = 8.0;
                    text::icon(ui, icons::TUNE, 20.0, colors::PRIMARY);
                    text::label(
                        ui,
                        text::caps(tr("settings.title")),
                        typography::headline_sm(),
                        colors::ON_SURFACE,
                    );
                    text::label(ui, "/", typography::label_sm(), colors::OUTLINE);
                    Tag::new(tr("settings.project"), colors::ON_SURFACE_VARIANT)
                        .part(format!(" {}", tr("settings.target")), colors::OUTLINE)
                        .dot(colors::TERTIARY)
                        .gap(8.0)
                        .fill(colors::SURFACE_CONTAINER)
                        .padding(8.0, 2.0)
                        .show(ui);
                },
                |ui| {
                    ui.spacing_mut().item_spacing.x = 8.0;
                    let label = if state.applied {
                        tr("settings.applied")
                    } else {
                        tr("settings.apply")
                    };
                    apply = Button::new(label, ButtonStyle::Pale)
                        .icon(icons::SAVE, 16.0)
                        .padding(12.0, 6.0)
                        .show(ui)
                        .on_hover_text(tr("settings.engine_note"))
                        .clicked();
                    defaults = Button::new(tr("settings.defaults"), ButtonStyle::Container)
                        .icon(icons::RESTORE, 16.0)
                        .padding(12.0, 6.0)
                        .show(ui)
                        .clicked();
                    IconButton::bell()
                        .sizes(18.0, 28.0)
                        .badge(true)
                        .tooltip(tr("library.notifications"))
                        .show(ui);
                    SearchField::new(tr("settings.search_hint"))
                        .width(256.0)
                        .height(28.0)
                        .fill(colors::SURFACE_LOWEST)
                        .font(typography::label_md(), 16.0)
                        .show(ui, &mut state.search);
                },
            );
        });
    if apply {
        Some(HeaderAction::Apply)
    } else if defaults {
        Some(HeaderAction::Defaults)
    } else {
        None
    }
}
