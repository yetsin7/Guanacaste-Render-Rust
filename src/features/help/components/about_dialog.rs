//! The about box: version, current stage and font licenses.

use crate::components::{
    button::{Button, ButtonStyle},
    icons, modal, text,
};
use crate::config::theme::{colors, typography};
use crate::core::Application;
use crate::models::dialog::DialogOutcome;
use crate::services::localization::tr;
use eframe::egui::Context;

/// Draws the dialog.
pub fn show(ctx: &Context) -> DialogOutcome {
    let mut outcome = DialogOutcome::Keep;
    let application = Application::new();
    let closed = modal::show(
        ctx,
        "about-dialog",
        tr("about.title"),
        icons::LOGO,
        480.0,
        |ui| {
            modal::content(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing.x = 12.0;
                    text::icon(ui, icons::LOGO, 40.0, colors::PRIMARY);
                    ui.vertical(|ui| {
                        ui.spacing_mut().item_spacing.y = 2.0;
                        text::label(
                            ui,
                            text::caps(application.name()),
                            typography::headline_md(),
                            colors::ON_SURFACE,
                        );
                        text::label(
                            ui,
                            tr("about.tagline"),
                            typography::body_sm(),
                            colors::ON_SURFACE_VARIANT,
                        );
                        text::label(
                            ui,
                            format!("{} {}", tr("about.version"), application.version()),
                            typography::label_sm(),
                            colors::TERTIARY,
                        );
                    });
                });
                text::paragraph(
                    ui,
                    tr("about.stage"),
                    typography::body_sm(),
                    colors::ON_SURFACE_VARIANT,
                    18.0,
                );
                text::label(
                    ui,
                    text::caps(tr("about.licenses")),
                    typography::mono_medium(11.0),
                    colors::OUTLINE,
                );
                for key in [
                    "about.license_inter",
                    "about.license_mono",
                    "about.license_icons",
                ] {
                    text::label(ui, tr(key), typography::label_sm(), colors::ON_SURFACE);
                }
                text::paragraph(
                    ui,
                    tr("about.placeholders"),
                    typography::body_sm(),
                    colors::OUTLINE,
                    18.0,
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
            });
        },
    );
    if closed {
        DialogOutcome::Close
    } else {
        outcome
    }
}
