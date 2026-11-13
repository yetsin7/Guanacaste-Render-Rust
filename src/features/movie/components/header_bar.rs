//! The studio header: close, title, scene chip and the export button.

use crate::components::{
    button::{Button, ButtonStyle},
    icon_button::IconButton,
    icons,
    layout::split_row,
    tag::Tag,
    text,
};
use crate::config::theme::{colors, typography};
use crate::services::localization::tr;
use eframe::egui::{self, Margin, Sense, Ui, Vec2};

/// What the user asked for in the header.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HeaderAction {
    /// Leave the studio.
    Close,
    /// Open the video export panel.
    Export,
}

/// Draws the header; `scene` is the name of the open project, if any.
pub fn show(ui: &mut Ui, scene: Option<&str>) -> Option<HeaderAction> {
    let (mut close, mut export) = (false, false);
    egui::Frame::new()
        .fill(colors::SURFACE_LOW)
        .inner_margin(Margin::symmetric(8, 2))
        .show(ui, |ui| {
            split_row(
                ui,
                32.0,
                |ui| {
                    ui.spacing_mut().item_spacing.x = 8.0;
                    if IconButton::new(icons::CLOSE)
                        .sizes(15.0, 24.0)
                        .tooltip(tr("movie.close"))
                        .show(ui)
                        .clicked()
                    {
                        close = true;
                    }
                    let (bar, _) = ui.allocate_exact_size(Vec2::new(1.0, 12.0), Sense::hover());
                    ui.painter().rect_filled(bar, 0.0, colors::OUTLINE_VARIANT);
                    text::icon(ui, icons::MOVIE, 17.0, colors::PRIMARY);
                    text::label(
                        ui,
                        tr("movie.title"),
                        typography::headline_sm(),
                        colors::ON_SURFACE,
                    );
                    Tag::new(text::caps(tr("movie.sequence")), colors::OUTLINE)
                        .fill(colors::SURFACE_HIGH)
                        .padding(6.0, 1.0)
                        .show(ui);
                    ui.add_space(8.0);
                    Tag::new(tr("movie.scene"), colors::OUTLINE)
                        .part(
                            format!(" {}", scene.unwrap_or(tr("movie.no_project"))),
                            colors::ON_SURFACE,
                        )
                        .fill(colors::SURFACE_LOWEST)
                        .padding(8.0, 2.0)
                        .show(ui);
                },
                |ui| {
                    if Button::new(tr("movie.export"), ButtonStyle::Pale)
                        .icon(icons::MOVIE, 14.0)
                        .font(typography::label_sm())
                        .padding(12.0, 4.0)
                        .show(ui)
                        .clicked()
                    {
                        export = true;
                    }
                },
            );
        });
    if close {
        Some(HeaderAction::Close)
    } else if export {
        Some(HeaderAction::Export)
    } else {
        None
    }
}
