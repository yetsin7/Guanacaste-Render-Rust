//! The render engine card: ray tracing quality, frame generation and V-Sync rows.

use crate::components::{icons, layout::split_row, tag::Tag, text, toggle};
use crate::config::theme::{colors, typography};
use crate::features::settings::logic::matches_search;
use crate::features::settings::models::SettingsState;
use crate::services::localization::tr;
use eframe::egui::{self, pos2, vec2, Color32, CornerRadius, Margin, Rect, Sense, Ui, UiBuilder};

/// A panel card: title row with icon, accent title and right-aligned caption.
pub fn card(
    ui: &mut Ui,
    glyph: char,
    accent: Color32,
    title: &'static str,
    caption: &'static str,
    body: impl FnOnce(&mut Ui),
) {
    egui::Frame::new()
        .fill(colors::SURFACE_LOW)
        .corner_radius(CornerRadius::same(4))
        .inner_margin(Margin::same(12))
        .show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.spacing_mut().item_spacing.y = 8.0;
            split_row(
                ui,
                24.0,
                |ui| {
                    ui.spacing_mut().item_spacing.x = 4.0;
                    text::icon(ui, glyph, 18.0, accent);
                    text::label(ui, text::caps(tr(title)), typography::label_lg(), accent);
                },
                |ui| {
                    text::label(ui, tr(caption), typography::label_sm(), colors::OUTLINE);
                },
            );
            body(ui);
        });
}

/// One setting row: icon tile, title with badge, description and a trailing control drawn by `trailing`.
#[allow(clippy::too_many_arguments)]
pub fn setting_row(
    ui: &mut Ui,
    glyph: char,
    glyph_color: Color32,
    title: &str,
    badge: (&str, Color32, Color32),
    description: &str,
    trailing: impl FnOnce(&mut Ui),
) {
    let (rect, _) = ui.allocate_exact_size(vec2(ui.available_width(), 56.0), Sense::hover());
    ui.painter()
        .rect_filled(rect, CornerRadius::same(2), colors::SURFACE_CONTAINER);
    let tile = Rect::from_min_size(
        pos2(rect.left() + 8.0, rect.center().y - 16.0),
        vec2(32.0, 32.0),
    );
    ui.painter()
        .rect_filled(tile, CornerRadius::same(2), colors::SURFACE_LOWEST);
    text::paint_icon(ui.painter(), tile.center(), glyph, 20.0, glyph_color);
    let mut info = ui.new_child(UiBuilder::new().max_rect(Rect::from_min_max(
        pos2(tile.right() + 8.0, rect.top() + 8.0),
        pos2(rect.right() - 220.0, rect.bottom()),
    )));
    info.spacing_mut().item_spacing.y = 0.0;
    info.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 8.0;
        text::label(ui, title, typography::headline_sm(), colors::ON_SURFACE);
        Tag::new(badge.0, badge.2)
            .font(typography::mono_medium(11.0))
            .fill(badge.1)
            .padding(6.0, 2.0)
            .show(ui);
    });
    text::label(
        &mut info,
        description,
        typography::body_sm(),
        colors::ON_SURFACE_VARIANT,
    );
    let mut right = ui.new_child(
        UiBuilder::new()
            .max_rect(rect.shrink2(vec2(12.0, 0.0)))
            .layout(egui::Layout::right_to_left(egui::Align::Center)),
    );
    trailing(&mut right);
}

/// Draws the card.
pub fn show(ui: &mut Ui, state: &mut SettingsState) {
    card(
        ui,
        icons::VIEW_IN_AR,
        colors::PRIMARY,
        "settings.engine",
        "settings.engine_caption",
        |ui| {
            ui.spacing_mut().item_spacing.y = 4.0;
            let search = state.search.clone();
            if matches_search(&[tr("settings.rt"), tr("settings.rt_note")], &search) {
                let detail = format!(
                    "{} ({}spp)",
                    tr("settings.multi_bounce"),
                    state.quality.samples()
                );
                let mut cycle = false;
                setting_row(
                    ui,
                    icons::LIGHT,
                    colors::PRIMARY,
                    tr("settings.rt"),
                    (
                        tr(state.quality.label_key()),
                        colors::PRIMARY_CONTAINER,
                        colors::ON_PRIMARY_CONTAINER,
                    ),
                    tr("settings.rt_note"),
                    |ui| {
                        text::icon(ui, icons::CHEVRON_RIGHT, 20.0, colors::OUTLINE);
                        text::label(ui, detail, typography::label_md(), colors::OUTLINE);
                        cycle = ui
                            .interact(ui.max_rect(), ui.id().with("rt-cycle"), Sense::click())
                            .clicked();
                    },
                );
                if cycle {
                    state.quality = state.quality.next();
                    state.applied = false;
                }
            }
            if matches_search(
                &[tr("settings.dlss"), tr("settings.dlss_note"), "DLSS"],
                &search,
            ) {
                let mut cycle = false;
                setting_row(
                    ui,
                    icons::EFFECTS,
                    colors::TERTIARY,
                    tr("settings.dlss"),
                    (
                        tr(state.upscale.label_key()),
                        colors::SURFACE_HIGHEST,
                        colors::TERTIARY,
                    ),
                    tr("settings.dlss_note"),
                    |ui| {
                        text::icon(ui, icons::CHEVRON_RIGHT, 20.0, colors::OUTLINE);
                        text::label(ui, "DLSS 3.7+", typography::label_md(), colors::OUTLINE);
                        cycle = ui
                            .interact(ui.max_rect(), ui.id().with("dlss-cycle"), Sense::click())
                            .clicked();
                    },
                );
                if cycle {
                    state.upscale = state.upscale.next();
                    state.applied = false;
                }
            }
            if matches_search(&[tr("settings.vsync"), tr("settings.vsync_note")], &search) {
                let label = if state.vsync {
                    tr("settings.enabled")
                } else {
                    tr("settings.disabled")
                };
                setting_row(
                    ui,
                    icons::SYNC,
                    colors::OUTLINE,
                    tr("settings.vsync"),
                    (label, colors::SURFACE_LOWEST, colors::OUTLINE),
                    tr("settings.vsync_note"),
                    |ui| {
                        toggle::show_sized(ui, &mut state.vsync, vec2(40.0, 20.0));
                    },
                );
            }
        },
    );
}
