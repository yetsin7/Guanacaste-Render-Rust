//! The notifications panel anchored under the title bar.

use crate::components::{
    button::{Button, ButtonStyle},
    empty_state, frames, icons,
    layout::split_row,
    notice::{self, Level},
    text,
};
use crate::config::theme::{colors, typography};
use crate::models::dialog::DialogOutcome;
use crate::services::localization::tr;
use eframe::egui::{self, Align2, Context, Id, Order, Pos2};

/// A notification entry: level plus localization keys.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Notice {
    /// Severity.
    pub level: Level,
    /// Title key.
    pub title: &'static str,
    /// Body key.
    pub body: &'static str,
}

/// The notices the application starts with.
pub fn initial_notices() -> Vec<Notice> {
    vec![
        Notice {
            level: Level::Info,
            title: "notices.info_title",
            body: "notices.info_body",
        },
        Notice {
            level: Level::Warning,
            title: "notices.warn_title",
            body: "notices.warn_body",
        },
        Notice {
            level: Level::Error,
            title: "notices.error_title",
            body: "notices.error_body",
        },
    ]
}

/// Draws the panel with its top-right corner at `anchor`; clicking outside closes it.
pub fn show(ctx: &Context, anchor: Pos2, notices: &mut Vec<Notice>) -> DialogOutcome {
    let response = egui::Area::new(Id::new("notifications-panel"))
        .order(Order::Foreground)
        .pivot(Align2::RIGHT_TOP)
        .fixed_pos(anchor)
        .show(ctx, |ui| {
            frames::floating(12).show(ui, |ui| {
                ui.set_width(360.0);
                ui.spacing_mut().item_spacing.y = 8.0;
                let mut clear = false;
                split_row(
                    ui,
                    24.0,
                    |ui| {
                        ui.spacing_mut().item_spacing.x = 6.0;
                        text::icon(ui, icons::NOTIFICATIONS, 18.0, colors::PRIMARY);
                        text::label(
                            ui,
                            tr("notices.title"),
                            typography::headline_sm(),
                            colors::ON_SURFACE,
                        );
                    },
                    |ui| {
                        clear = Button::new(tr("notices.clear"), ButtonStyle::Quiet)
                            .font(typography::label_sm())
                            .padding(8.0, 2.0)
                            .show(ui)
                            .clicked();
                    },
                );
                if clear {
                    notices.clear();
                }
                if notices.is_empty() {
                    empty_state::show(
                        ui,
                        icons::NOTIFICATIONS,
                        tr("notices.empty_title"),
                        tr("notices.empty_body"),
                        None,
                    );
                }
                for notice_entry in notices.iter() {
                    notice::show(
                        ui,
                        notice_entry.level,
                        tr(notice_entry.title),
                        tr(notice_entry.body),
                    );
                }
            });
        })
        .response;
    let outside = ctx.input(|input| input.pointer.any_click()) && !response.contains_pointer();
    if outside || ctx.input(|input| input.key_pressed(egui::Key::Escape)) {
        DialogOutcome::Close
    } else {
        DialogOutcome::Keep
    }
}
