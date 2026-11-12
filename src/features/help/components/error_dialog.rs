//! The error dialog: what failed, why, and the technical code.

use crate::components::{
    button::{Button, ButtonStyle},
    icons, modal,
    notice::{self, Level},
    text,
};
use crate::config::theme::{colors, typography};
use crate::models::dialog::{DialogOutcome, ErrorInfo};
use crate::services::localization::{tr, trf};
use eframe::egui::Context;

/// Draws the dialog for `error`.
pub fn show(ctx: &Context, error: &ErrorInfo) -> DialogOutcome {
    let mut outcome = DialogOutcome::Keep;
    let closed = modal::show(
        ctx,
        "error-dialog",
        tr("error.title"),
        icons::ERROR,
        480.0,
        |ui| {
            modal::content(ui, |ui| {
                notice::show(ui, Level::Error, tr(error.title), tr(error.message));
                text::label(
                    ui,
                    format!("{} {}", tr("error.code"), error.code),
                    typography::label_sm(),
                    colors::OUTLINE,
                );
            });
            modal::footer(ui, |ui| {
                if Button::new(tr("dialog.accept"), ButtonStyle::Pale)
                    .padding(16.0, 6.0)
                    .show(ui)
                    .clicked()
                {
                    outcome = DialogOutcome::Close;
                }
                if Button::new(tr("error.copy"), ButtonStyle::Container)
                    .icon(icons::COPY, 14.0)
                    .padding(12.0, 6.0)
                    .show(ui)
                    .clicked()
                {
                    let details = trf(
                        "error.details",
                        &[tr(error.title), tr(error.message), error.code],
                    );
                    ui.ctx().copy_text(details);
                }
            });
        },
    );
    if closed {
        DialogOutcome::Close
    } else {
        outcome
    }
}
