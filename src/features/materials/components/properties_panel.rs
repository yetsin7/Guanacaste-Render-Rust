//! The 390 px properties panel: header, scrolling sections and the apply footer.

use super::{channel_modifiers, environment_section, shader_section, texture_section};
use crate::components::{
    button::{Button, ButtonStyle},
    icons,
    layout::split_row,
    text,
};
use crate::config::theme::{colors, typography};
use crate::features::materials::{logic, models::MaterialState};
use crate::services::localization::tr;
use eframe::egui::{
    self, pos2, vec2, Align, CornerRadius, Layout, Margin, Rect, ScrollArea, Ui, UiBuilder,
};

/// Width of the panel in points.
pub const WIDTH: f32 = 390.0;

/// A grey card holding one group of parameters.
pub fn section(ui: &mut Ui, add: impl FnOnce(&mut Ui)) {
    egui::Frame::new()
        .fill(colors::SURFACE_CONTAINER)
        .corner_radius(CornerRadius::same(2))
        .inner_margin(Margin::same(12))
        .show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.spacing_mut().item_spacing.y = 4.0;
            add(ui);
        });
}

/// Draws the panel into `area`.
pub fn show(ui: &mut Ui, area: Rect, state: &mut MaterialState) {
    ui.painter().rect_filled(area, 0.0, colors::SURFACE_LOW);
    let header = Rect::from_min_size(area.min, vec2(area.width(), 36.0));
    ui.painter().rect_filled(header, 0.0, colors::SURFACE_HIGH);
    let mut head = ui.new_child(
        UiBuilder::new()
            .max_rect(header.shrink2(vec2(12.0, 0.0)))
            .layout(Layout::top_down(Align::Min)),
    );
    split_row(
        &mut head,
        36.0,
        |ui| {
            ui.spacing_mut().item_spacing.x = 4.0;
            text::icon(ui, icons::TUNE, 18.0, colors::PRIMARY);
            text::label(
                ui,
                tr("material.properties"),
                typography::headline_sm(),
                colors::ON_SURFACE,
            );
        },
        |ui| {
            ui.spacing_mut().item_spacing.x = 4.0;
            text::icon(ui, icons::BOOKMARK, 14.0, colors::ON_SURFACE_VARIANT);
            if Button::new(tr("material.reset"), ButtonStyle::Quiet)
                .font(typography::label_sm())
                .padding(8.0, 2.0)
                .show(ui)
                .clicked()
            {
                logic::reset(state);
            }
        },
    );

    let footer_height = 76.0;
    let footer = Rect::from_min_max(pos2(area.left(), area.bottom() - footer_height), area.max);
    ui.painter().rect_filled(footer, 0.0, colors::SURFACE_HIGH);
    let body = Rect::from_min_max(
        pos2(area.left(), header.bottom()),
        pos2(area.right(), footer.top()),
    );
    let mut scroll_ui = ui.new_child(UiBuilder::new().max_rect(body));
    ScrollArea::vertical()
        .auto_shrink([false, false])
        .show(&mut scroll_ui, |ui| {
            egui::Frame::new()
                .inner_margin(Margin::same(12))
                .show(ui, |ui| {
                    ui.spacing_mut().item_spacing.y = 12.0;
                    shader_section::show(ui, state);
                    texture_section::show(ui);
                    environment_section::show(ui, state);
                    channel_modifiers::show(ui, state);
                });
        });

    let mut foot = ui.new_child(
        UiBuilder::new()
            .max_rect(footer.shrink(12.0))
            .layout(Layout::top_down(Align::Min)),
    );
    foot.spacing_mut().item_spacing.y = 4.0;
    let label = if state.applied {
        tr("material.applied")
    } else {
        tr("material.apply")
    };
    if Button::new(label, ButtonStyle::Pale)
        .icon(icons::CHECK_CIRCLE, 16.0)
        .colors(colors::PRIMARY_CONTAINER, colors::ON_PRIMARY_CONTAINER)
        .fill_width()
        .height(32.0)
        .show(&mut foot)
        .clicked()
    {
        state.applied = true;
    }
    split_row(
        &mut foot,
        18.0,
        |ui| {
            text::label(
                ui,
                tr("material.cache"),
                typography::mono(10.0),
                colors::ON_SURFACE_VARIANT,
            );
        },
        |ui| {
            text::label(ui, "Δt = 0.8ms", typography::mono(10.0), colors::TERTIARY);
        },
    );
}
