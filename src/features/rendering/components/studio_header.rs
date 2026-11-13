//! The render studio header: title, tabs and the main actions.

use crate::components::{
    button::{Button, ButtonStyle},
    icons,
    layout::split_row,
    tab_bar, text,
};
use crate::config::theme::{colors, typography};
use crate::features::rendering::models::RenderStudioState;
use crate::models::dialog::Dialog;
use crate::services::localization::{tr, trf};
use eframe::egui::{self, Margin, Ui};

/// What the user asked of the studio.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HeaderAction {
    /// Render the selected view.
    Render,
    /// Render every preset view.
    RenderAllViews,
}

/// Draws the header; `ready` tells whether there is a scene to render.
pub fn show(ui: &mut Ui, state: &mut RenderStudioState, ready: bool) -> Option<HeaderAction> {
    let mut action = None;
    egui::Frame::new()
        .fill(colors::SURFACE_LOW)
        .inner_margin(Margin::symmetric(16, 0))
        .show(ui, |ui| {
            let queue_label = trf("render.tab_queue", &[&state.jobs.len().to_string()]);
            let mut tab = state.tab;
            split_row(
                ui,
                40.0,
                |ui| {
                    ui.spacing_mut().item_spacing.x = 8.0;
                    text::icon(ui, icons::CAMERA, 18.0, colors::PRIMARY);
                    text::label(
                        ui,
                        tr("render.title"),
                        typography::headline_sm(),
                        colors::ON_SURFACE,
                    );
                    ui.add_space(16.0);
                    tab_bar::show(
                        ui,
                        &[tr("render.tab_preview"), queue_label.as_str()],
                        &mut tab,
                    );
                },
                |ui| {
                    ui.spacing_mut().item_spacing.x = 8.0;
                    let mut start = Button::new(tr("render.start"), ButtonStyle::Solid)
                        .icon(icons::PLAY_ARROW, 16.0)
                        .font(typography::label_md())
                        .padding(14.0, 6.0);
                    let mut all = Button::new(tr("render.start_all"), ButtonStyle::Container)
                        .icon(icons::QUEUE, 16.0)
                        .font(typography::label_md())
                        .padding(12.0, 6.0);
                    if !ready {
                        start = start.disabled(tr("render.no_scene"));
                        all = all.disabled(tr("render.no_scene"));
                    }
                    if start.show(ui).clicked() {
                        action = Some(HeaderAction::Render);
                    }
                    if all.show(ui).clicked() {
                        action = Some(HeaderAction::RenderAllViews);
                    }
                    if Button::new(tr("render.export"), ButtonStyle::Raised)
                        .icon(icons::IMAGE, 16.0)
                        .font(typography::label_md())
                        .padding(12.0, 6.0)
                        .show(ui)
                        .clicked()
                    {
                        crate::commands::request_dialog(ui.ctx(), Dialog::ExportImage);
                    }
                },
            );
            state.tab = tab;
        });
    action
}
