//! The new-scene dialog: creates an empty project that is saved as `.gr` on the first "Save".

use crate::components::{
    button::{Button, ButtonStyle},
    form, icons, modal,
    notice::{self, Level},
};
use crate::features::projects::models::NewSceneState;
use crate::models::dialog::DialogOutcome;
use crate::services::localization::tr;
use eframe::egui::Context;

/// Draws the dialog.
pub fn show(ctx: &Context, state: &mut NewSceneState) -> DialogOutcome {
    let mut outcome = DialogOutcome::Keep;
    let templates = [
        tr("new.template_empty"),
        tr("new.template_terrain"),
        tr("new.template_interior"),
    ];
    let closed = modal::show(
        ctx,
        "new-scene-dialog",
        tr("new.title"),
        icons::ADD_BOX,
        520.0,
        |ui| {
            modal::content(ui, |ui| {
                form::caption(ui, tr("new.name"));
                form::text_field(ui, &mut state.name, tr("new.name_hint"), 488.0);
                form::caption(ui, tr("new.template"));
                form::dropdown(
                    ui,
                    "new-scene-template",
                    &mut state.template,
                    &templates,
                    488.0,
                );
                form::caption(ui, tr("new.location"));
                form::text_field(ui, &mut state.folder, "C:/Proyectos/Guanacaste", 488.0);
                notice::show(ui, Level::Info, tr("new.note_title"), tr("new.note_body"));
            });
            modal::footer(ui, |ui| {
                if Button::new(tr("new.create"), ButtonStyle::Pale)
                    .padding(16.0, 6.0)
                    .show(ui)
                    .clicked()
                {
                    state.create = true;
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
