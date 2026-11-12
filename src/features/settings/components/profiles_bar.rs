//! The render profile selector bar.

use crate::components::{
    button::{Button, ButtonStyle},
    icons,
    layout::split_row,
    text,
};
use crate::config::theme::{colors, typography};
use crate::features::settings::logic::apply_profile;
use crate::features::settings::models::{RenderProfile, SettingsState};
use crate::services::localization::tr;
use eframe::egui::{self, CornerRadius, Margin, Ui};

const LABELS: [&str; 3] = [
    "settings.profile_draft",
    "settings.profile_interactive",
    "settings.profile_production",
];

/// Draws the bar; clicking a profile applies its preset.
pub fn show(ui: &mut Ui, state: &mut SettingsState) {
    egui::Frame::new()
        .fill(colors::SURFACE_LOW)
        .corner_radius(CornerRadius::same(4))
        .inner_margin(Margin::same(12))
        .show(ui, |ui| {
            let mut chosen = None;
            split_row(
                ui,
                28.0,
                |ui| {
                    ui.spacing_mut().item_spacing.x = 4.0;
                    text::icon(ui, icons::BOLT, 16.0, colors::OUTLINE);
                    text::label(
                        ui,
                        text::caps(tr("settings.profiles")),
                        typography::label_md(),
                        colors::ON_SURFACE_VARIANT,
                    );
                },
                |ui| {
                    ui.spacing_mut().item_spacing.x = 4.0;
                    for (index, profile) in RenderProfile::ALL.into_iter().enumerate().rev() {
                        let selected = state.profile == profile;
                        let (fill, ink) = if selected {
                            (colors::PRIMARY, colors::ON_PRIMARY)
                        } else {
                            (colors::SURFACE_CONTAINER, colors::ON_SURFACE)
                        };
                        let font = if selected {
                            typography::mono_medium(11.0)
                        } else {
                            typography::label_sm()
                        };
                        if Button::new(tr(LABELS[index]), ButtonStyle::Container)
                            .font(font)
                            .padding(12.0, 4.0)
                            .colors(fill, ink)
                            .show(ui)
                            .clicked()
                        {
                            chosen = Some(profile);
                        }
                    }
                },
            );
            if let Some(profile) = chosen {
                apply_profile(state, profile);
            }
        });
}
