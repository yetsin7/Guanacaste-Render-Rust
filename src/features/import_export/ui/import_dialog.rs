//! The import dialog: file picker, format explanation, progress with cancel, content preview and import report.

use super::state::{ImportDialogState, UNIT_CHOICES};
use crate::components::{
    button::{Button, ButtonStyle},
    form, icons, modal,
    notice::{self, Level},
    progress_bar, text,
};
use crate::config::theme::{colors, typography};
use crate::features::import_export::components::{
    proprietary_notice, report_list, summary_panel, wireframe,
};
use crate::features::import_export::logic::formats::Detected;
use crate::features::import_export::models::{ImportError, SourceFormat};
use crate::models::dialog::DialogOutcome;
use crate::services::localization::{tr, trf};
use eframe::egui::{vec2, Context, Ui};
use std::time::Duration;

fn error_notice(ui: &mut Ui, error: &ImportError) {
    let detail = if error.detail().is_empty() {
        String::new()
    } else {
        format!("\n{}", error.detail())
    };
    notice::show(
        ui,
        if *error == ImportError::Cancelled {
            Level::Info
        } else {
            Level::Error
        },
        tr(error.key()),
        &detail,
    );
}

fn options(ui: &mut Ui, state: &mut ImportDialogState) {
    let labels: Vec<&str> = UNIT_CHOICES.iter().map(|(name, _)| *name).collect();
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 12.0;
        text::label(
            ui,
            tr("import.unit_scale"),
            typography::body_sm(),
            colors::ON_SURFACE_VARIANT,
        );
        form::dropdown(ui, "import-unit", &mut state.unit_index, &labels, 80.0);
        ui.checkbox(&mut state.options.y_up, tr("import.y_up"));
        if Button::new(tr("import.reimport"), ButtonStyle::Container)
            .padding(10.0, 4.0)
            .show(ui)
            .clicked()
        {
            state.start();
        }
    });
}

fn body(ui: &mut Ui, state: &mut ImportDialogState) {
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 8.0;
        if Button::new(tr("import.choose"), ButtonStyle::Pale)
            .icon(icons::FOLDER_OPEN, 16.0)
            .padding(14.0, 6.0)
            .show(ui)
            .clicked()
        {
            state.picker.open();
        }
        let name = state.path.as_ref().map_or_else(
            || tr("import.no_file").to_owned(),
            |p| p.display().to_string(),
        );
        text::label(ui, name, typography::label_sm(), colors::ON_SURFACE_VARIANT);
    });
    text::paragraph(
        ui,
        tr("import.supported"),
        typography::label_sm(),
        colors::OUTLINE,
        15.0,
    );
    match state.detected {
        Some(Detected::Proprietary(kind)) => proprietary_notice::show(ui, kind),
        Some(Detected::Unknown) => notice::show(
            ui,
            Level::Warning,
            tr("import.unknown_title"),
            tr("import.unknown_body"),
        ),
        Some(Detected::Supported(SourceFormat::Obj)) => options(ui, state),
        Some(Detected::Supported(_)) | None => {}
    }
    if let Some(job) = &state.job {
        let fraction = job.progress.get();
        text::label(
            ui,
            trf("import.analyzing", &[&format!("{:.0}", fraction * 100.0)]),
            typography::label_md(),
            colors::PRIMARY,
        );
        progress_bar::show(ui, fraction, colors::PRIMARY_CONTAINER, 6.0);
        if Button::new(tr("import.cancel"), ButtonStyle::Container)
            .padding(12.0, 5.0)
            .show(ui)
            .clicked()
        {
            state.cancel();
        }
    }
    let (yaw, pitch) = (&mut state.yaw, &mut state.pitch);
    match state.result.as_deref() {
        Some(Ok(outcome)) => {
            summary_panel::show(ui, &outcome.scene, &outcome.report);
            ui.horizontal_top(|ui| {
                ui.spacing_mut().item_spacing.x = 12.0;
                wireframe::show(ui, &outcome.scene, yaw, pitch, vec2(300.0, 210.0));
                ui.vertical(|ui| {
                    text::label(
                        ui,
                        text::caps(&trf(
                            "import.report",
                            &[&outcome.report.warning_count().to_string()],
                        )),
                        typography::mono_medium(11.0),
                        colors::OUTLINE,
                    );
                    report_list::show(ui, &outcome.report, 190.0);
                });
            });
            text::paragraph(
                ui,
                tr("import.not_loaded"),
                typography::label_sm(),
                colors::OUTLINE,
                15.0,
            );
        }
        Some(Err(error)) => error_notice(ui, error),
        None => {}
    }
}

/// Draws the dialog.
pub fn show(ctx: &Context, state: &mut ImportDialogState) -> DialogOutcome {
    state.tick();
    if state.busy() {
        ctx.request_repaint_after(Duration::from_millis(80));
    }
    let closed = modal::show(
        ctx,
        "import-dialog",
        tr("import.title"),
        icons::UPLOAD,
        760.0,
        |ui| {
            modal::content(ui, |ui| body(ui, state));
            modal::footer(ui, |ui| {
                if Button::new(tr("import.close"), ButtonStyle::Container)
                    .padding(16.0, 6.0)
                    .show(ui)
                    .clicked()
                {
                    state.cancel();
                }
            });
        },
    );
    if closed {
        state.cancel();
        DialogOutcome::Close
    } else {
        DialogOutcome::Keep
    }
}
