//! The save-project dialog.

use crate::components::{
    button::{Button, ButtonStyle},
    form, icons, modal,
    notice::{self, Level},
};
use crate::features::projects::models::SaveState;
use crate::models::dialog::{Dialog, DialogOutcome, ErrorInfo};
use crate::services::localization::tr;
use eframe::egui::Context;

/// The error raised when saving is attempted.
pub const SAVE_ERROR: ErrorInfo = ErrorInfo {
    title: "error.save_title",
    message: "error.save_body",
    code: "GR-PROJECT-002",
};

/// Draws the dialog.
pub fn show(ctx: &Context, state: &mut SaveState) -> DialogOutcome {
    let mut outcome = DialogOutcome::Keep;
    let closed = modal::show(
        ctx,
        "save-dialog",
        tr("save.title"),
        icons::SAVE,
        520.0,
        |ui| {
            modal::content(ui, |ui| {
                form::caption(ui, tr("save.name"));
                form::text_field(ui, &mut state.name, "", 488.0);
                form::caption(ui, tr("save.folder"));
                form::text_field(ui, &mut state.folder, "", 488.0);
                form::switch_row(
                    ui,
                    tr("save.compress"),
                    tr("save.compress_note"),
                    &mut state.compress,
                );
                form::switch_row(
                    ui,
                    tr("save.thumbnail"),
                    tr("save.thumbnail_note"),
                    &mut state.thumbnail,
                );
                notice::show(
                    ui,
                    Level::Warning,
                    tr("notices.error_title"),
                    tr("notices.error_body"),
                );
            });
            modal::footer(ui, |ui| {
                if Button::new(tr("save.save"), ButtonStyle::Pale)
                    .padding(16.0, 6.0)
                    .show(ui)
                    .clicked()
                {
                    outcome = DialogOutcome::Open(Dialog::Error(SAVE_ERROR));
                }
                if Button::new(tr("dialog.cancel"), ButtonStyle::Container)
                    .padding(16.0, 6.0)
                    .show(ui)
                    .clicked()
                {
                    outcome = DialogOutcome::Close;
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
