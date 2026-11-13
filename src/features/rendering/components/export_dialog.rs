//! The export-image dialog: renders the project from the corner view and writes the picture to a folder.

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
use crate::features::rendering::logic::estimate_megabytes;
use crate::features::rendering::models::{
    output_size, AspectRatio, ExportState, ImageFormat, QUALITIES, RESOLUTIONS,
};
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
    let labels: Vec<String> = RESOLUTIONS.iter().map(|(w, _)| format!("{w} px")).collect();
    let resolutions: Vec<&str> = labels.iter().map(String::as_str).collect();
    let ratios: Vec<&str> = AspectRatio::ALL.iter().map(|a| a.name()).collect();
    let qualities: Vec<&str> = QUALITIES.iter().map(|q| tr(q.label_key())).collect();
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
                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing.x = 16.0;
                    ui.vertical(|ui| {
                        form::caption(ui, tr("render.aspect"));
                        form::dropdown(ui, "export-aspect", &mut state.aspect, &ratios, 232.0);
                    });
                    ui.vertical(|ui| {
                        form::caption(ui, tr("render.quality"));
                        form::dropdown(ui, "export-quality", &mut state.quality, &qualities, 232.0);
                    });
                });
                let format = ImageFormat::ALL[state.format.min(ImageFormat::ALL.len() - 1)];
                if format == ImageFormat::Jpeg {
                    split_row(
                        ui,
                        18.0,
                        |ui| form::caption(ui, tr("export.quality")),
                        |ui| {
                            text::label(
                                ui,
                                format!("{:.0}%", state.jpeg_quality * 100.0),
                                typography::label_md(),
                                colors::PRIMARY,
                            );
                        },
                    );
                    Slider::precision().show(ui, &mut state.jpeg_quality);
                }
                form::caption(ui, tr("render.folder"));
                form::text_field(ui, &mut state.folder, "", 488.0);
                form::caption(ui, tr("render.file_name"));
                form::text_field(ui, &mut state.file_name, "", 488.0);
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
                let size = output_size(state.resolution, state.aspect);
                let estimate = estimate_megabytes(format, size, state.jpeg_quality);
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
                            format!("{}×{} px · ≈ {estimate:.1} MB", size.0, size.1),
                            typography::label_md(),
                            colors::ON_SURFACE,
                        );
                    },
                );
                notice::show(
                    ui,
                    Level::Info,
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
