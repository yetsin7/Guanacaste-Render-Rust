//! The 390 px properties panel: header, scrolling sections and the texture-cache footer.

use super::{session::Session, surface_section, texture_section, uv_section};
use crate::components::{
    button::{Button, ButtonStyle},
    icons,
    layout::split_row,
    text,
};
use crate::config::theme::{colors, typography};
use crate::features::materials::logic::editor;
use crate::features::materials::models::MaterialState;
use crate::services::localization::{tr, trf};
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
pub fn show(ui: &mut Ui, area: Rect, state: &mut MaterialState, session: &mut Session) {
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
            if state.current().is_some()
                && Button::new(tr("material.reset"), ButtonStyle::Quiet)
                    .font(typography::label_sm())
                    .padding(8.0, 2.0)
                    .show(ui)
                    .clicked()
            {
                editor::reset_selected(state);
            }
        },
    );

    let footer = Rect::from_min_max(pos2(area.left(), area.bottom() - 32.0), area.max);
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
                    if state.current().is_none() {
                        text::paragraph(
                            ui,
                            tr("material.none_selected"),
                            typography::body_sm(),
                            colors::ON_SURFACE_VARIANT,
                            18.0,
                        );
                        return;
                    }
                    surface_section::show(ui, state);
                    texture_section::show(ui, state, session);
                    uv_section::show(ui, state);
                });
        });

    let mut foot = ui.new_child(
        UiBuilder::new()
            .max_rect(footer.shrink2(vec2(12.0, 0.0)))
            .layout(Layout::top_down(Align::Min)),
    );
    let mib = |bytes: usize| format!("{:.1}", bytes as f64 / (1024.0 * 1024.0));
    let stats = trf(
        "material.cache_stats",
        &[
            &session.cache.len().to_string(),
            &mib(session.cache.used_bytes()),
            &mib(session.cache.budget()),
        ],
    );
    split_row(
        &mut foot,
        32.0,
        |ui| {
            text::label(
                ui,
                stats,
                typography::mono(10.0),
                colors::ON_SURFACE_VARIANT,
            );
        },
        |_| {},
    );
}
