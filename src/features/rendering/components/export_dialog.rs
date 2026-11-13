//! The export-image dialog. Until the render engine exists the image is the plan + isometric line preview.

use crate::components::{
    button::{Button, ButtonStyle},
    form, icons,
    layout::split_row,
    modal,
    notice::{self, Level},
    slider::Slider,
    text,
};
use crate::config::theme::{colors, typography};
use crate::features::rendering::logic::{estimate_megabytes, resolution_label};
use crate::features::rendering::models::{ExportState, ImageFormat, RESOLUTIONS};
use crate::models::dialog::DialogOutcome;
use crate::services::localization::tr;
use eframe::egui::Context;

/// Draws the dialog.
pub fn show(ctx: &Context, state: &mut ExportState) -> DialogOutcome {
    let mut outcome = DialogOutcome::Keep;
    let formats: Vec<&str> = ImageFormat::ALL
        .iter()
        .map(|format| format.name())
        .collect();
    let labels: Vec<String> = (0..RESOLUTIONS.len()).map(resolution_label).collect();
    let resolutions: Vec<&str> = labels.iter().map(String::as_str).collect();
    let closed = modal::show(
        ctx,
        "export-dialog",
        tr("export.title"),
        icons::IMAGE,
        520.0,
        |ui| {
            modal::content(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing.x = 16.0;
                    ui.vertical(|ui| {
                        form::caption(ui, tr("export.format"));
                        form::dropdown(ui, "export-format", &mut state.format, &formats, 232.0);
                    });
                    ui.vertical(|ui| {
                        form::caption(ui, tr("export.resolution"));
                        form::dropdown(
                            ui,
                            "export-resolution",
                            &mut state.resolution,
                            &resolutions,
                            232.0,
                        );
                    });
                });
                let format = ImageFormat::ALL[state.format];
                if format == ImageFormat::Jpeg {
                    split_row(
                        ui,
                        18.0,
                        |ui| form::caption(ui, tr("export.quality")),
                        |ui| {
                            text::label(
                                ui,
                                format!("{:.0}%", state.quality * 100.0),
                                typography::label_md(),
                                colors::PRIMARY,
                            );
                        },
                    );
                    Slider::precision().show(ui, &mut state.quality);
                }
                form::caption(ui, tr("export.path"));
                form::text_field(ui, &mut state.path, "", 488.0);
                if format.supports_alpha() {
                    form::switch_row(
                        ui,
                        tr("export.transparent"),
                        tr("export.transparent_note"),
                        &mut state.transparent,
                    );
                } else {
                    state.transparent = false;
                }
                let estimate = estimate_megabytes(format, state.resolution, state.quality);
                split_row(
                    ui,
                    18.0,
                    |ui| {
                        text::label(
                            ui,
                            tr("export.estimate"),
                            typography::body_sm(),
                            colors::OUTLINE,
                        );
                    },
                    |ui| {
                        text::label(
                            ui,
                            format!("≈ {estimate:.1} MB"),
                            typography::label_md(),
                            colors::ON_SURFACE,
                        );
                    },
                );
                notice::show(
                    ui,
                    Level::Warning,
                    tr("export.note_title"),
                    tr("export.note_body"),
                );
            });
            modal::footer(ui, |ui| {
                if Button::new(tr("export.export"), ButtonStyle::Pale)
                    .icon(icons::DOWNLOAD, 16.0)
                    .padding(16.0, 6.0)
                    .show(ui)
                    .clicked()
                {
                    state.request = true;
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
