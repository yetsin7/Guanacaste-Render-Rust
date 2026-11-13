//! Offers the recovery copies written by autosave when the application did not close normally.

use crate::components::{
    button::{Button, ButtonStyle},
    icons, modal, text,
};
use crate::config::theme::{colors, typography};
use crate::features::projects::logic::describe::{age_label, size_label};
use crate::features::projects::services::{recovery::RecoveryEntry, util};
use crate::services::localization::tr;
use eframe::egui::Context;

/// The user's answer.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RecoveryChoice {
    /// Restore copy number `n`.
    Restore(usize),
    /// Delete every copy.
    DiscardAll,
    /// Keep the copies and decide later.
    Later,
}

/// Draws the list of `entries`; returns the answer once given.
pub fn show(ctx: &Context, entries: &[RecoveryEntry]) -> Option<RecoveryChoice> {
    let mut choice = None;
    let now = util::now_unix();
    let closed = modal::show(
        ctx,
        "recovery-dialog",
        tr("recovery.title"),
        icons::RESTORE,
        520.0,
        |ui| {
            modal::content(ui, |ui| {
                ui.set_width(488.0);
                text::paragraph(
                    ui,
                    tr("recovery.body"),
                    typography::body_md(),
                    colors::ON_SURFACE_VARIANT,
                    20.0,
                );
                ui.add_space(8.0);
                for (index, entry) in entries.iter().enumerate() {
                    ui.horizontal(|ui| {
                        text::label(ui, &entry.name, typography::label_md(), colors::ON_SURFACE);
                        text::label(
                            ui,
                            format!(
                                "  {} · {}",
                                age_label(now, entry.modified),
                                size_label(entry.size)
                            ),
                            typography::mono(10.0),
                            colors::OUTLINE,
                        );
                        ui.with_layout(
                            eframe::egui::Layout::right_to_left(eframe::egui::Align::Center),
                            |ui| {
                                if Button::new(tr("recovery.restore"), ButtonStyle::Pale)
                                    .padding(12.0, 4.0)
                                    .show(ui)
                                    .clicked()
                                {
                                    choice = Some(RecoveryChoice::Restore(index));
                                }
                            },
                        );
                    });
                }
            });
            modal::footer(ui, |ui| {
                if Button::new(tr("recovery.discard"), ButtonStyle::Container)
                    .icon(icons::DELETE, 16.0)
                    .padding(16.0, 6.0)
                    .show(ui)
                    .clicked()
                {
                    choice = Some(RecoveryChoice::DiscardAll);
                }
                if Button::new(tr("recovery.later"), ButtonStyle::Container)
                    .padding(16.0, 6.0)
                    .show(ui)
                    .clicked()
                {
                    choice = Some(RecoveryChoice::Later);
                }
            });
        },
    );
    if closed {
        Some(RecoveryChoice::Later)
    } else {
        choice
    }
}
