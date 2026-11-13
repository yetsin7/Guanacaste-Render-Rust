//! The "save as" dialog: file name, folder (typed or picked) and thumbnail. When "Save" had to become
//! "Save as" (a Guanacaste 3D original, a read-only project) one line explains why.

use crate::components::{
    button::{Button, ButtonStyle},
    form, icons, modal,
    notice::{self, Level},
};
use crate::features::projects::models::SaveState;
use crate::models::dialog::DialogOutcome;
use crate::services::localization::tr;
use eframe::egui::Context;

/// Draws the dialog.
pub fn show(ctx: &Context, state: &mut SaveState) -> DialogOutcome {
    if let Some(path) = state.picker.poll() {
        state.request = Some(path);
    }
    if state.picker.is_open() {
        ctx.request_repaint_after(std::time::Duration::from_millis(100));
    }
    let mut outcome = DialogOutcome::Keep;
    let closed = modal::show(
        ctx,
        "save-dialog",
        tr("save.title"),
        icons::SAVE,
        520.0,
        |ui| {
            modal::content(ui, |ui| {
                if let Some(reason) = state.reason_key {
                    notice::show(ui, Level::Warning, tr("save.reason_title"), tr(reason));
                }
                form::caption(ui, tr("save.name"));
                form::text_field(ui, &mut state.name, "escena.gr", 488.0);
                form::caption(ui, tr("save.folder"));
                ui.horizontal(|ui| {
                    form::text_field(ui, &mut state.folder, "C:/Proyectos", 380.0);
                    if Button::new(tr("open.browse"), ButtonStyle::Container)
                        .padding(12.0, 6.0)
                        .show(ui)
                        .clicked()
                    {
                        state.picker.save_project(&state.target());
                    }
                });
                form::switch_row(
                    ui,
                    tr("save.thumbnail"),
                    tr("save.thumbnail_note"),
                    &mut state.thumbnail,
                );
                notice::show(
                    ui,
                    Level::Info,
                    tr("save.format_title"),
                    tr("save.format_body"),
                );
            });
            modal::footer(ui, |ui| {
                let ready = !state.name.trim().is_empty() && !state.folder.trim().is_empty();
                if Button::new(tr("save.save"), ButtonStyle::Pale)
                    .padding(16.0, 6.0)
                    .show(ui)
                    .clicked()
                    && ready
                {
                    state.request = Some(state.target());
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
