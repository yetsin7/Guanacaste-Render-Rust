//! The studio header: close, title, scene chip, search, storyboards and export queue.

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
use crate::features::movie::models::MovieState;
use crate::services::localization::tr;
use eframe::egui::{self, Margin, Sense, Ui, Vec2};

/// What the user asked for in the header.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HeaderAction {
    /// Leave the studio.
    Close,
    /// Open the export queue.
    ExportQueue,
}

/// Draws the header and returns the requested action, if any.
pub fn show(ui: &mut Ui, state: &mut MovieState) -> Option<HeaderAction> {
    let mut close = false;
    let mut queue = false;
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
                        .part(format!(" {} ", tr("movie.scene_name")), colors::ON_SURFACE)
                        .part(tr("movie.live"), colors::TERTIARY)
                        .fill(colors::SURFACE_LOWEST)
                        .padding(8.0, 2.0)
                        .show(ui);
                },
                |ui| {
                    ui.spacing_mut().item_spacing.x = 8.0;
                    if Button::new(tr("movie.export"), ButtonStyle::Pale)
                        .icon(icons::MOVIE, 14.0)
                        .font(typography::label_sm())
                        .padding(12.0, 4.0)
                        .show(ui)
                        .clicked()
                    {
                        queue = true;
                    }
                    Button::new(tr("movie.storyboards"), ButtonStyle::Raised)
                        .icon(icons::BOOK, 14.0)
                        .font(typography::label_sm())
                        .padding(12.0, 4.0)
                        .show(ui);
                    IconButton::bell()
                        .sizes(16.0, 28.0)
                        .badge(true)
                        .tooltip(tr("library.notifications"))
                        .show(ui);
                    SearchField::new(tr("movie.search_hint"))
                        .width(208.0)
                        .height(24.0)
                        .font(typography::body_sm(), 14.0)
                        .show(ui, &mut state.search);
                },
            );
        });
    if close {
        Some(HeaderAction::Close)
    } else if queue {
        Some(HeaderAction::ExportQueue)
    } else {
        None
    }
}
