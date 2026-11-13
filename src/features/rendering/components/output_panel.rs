//! The output settings panel of the render studio.

use crate::components::{
    button::{Button, ButtonStyle},
    form, icons,
    layout::split_row,
    notice::{self, Level},
    text,
};
use crate::config::theme::{colors, typography};
use crate::features::rendering::engine::ViewPreset;
use crate::features::rendering::logic::estimate_megabytes;
use crate::features::rendering::models::{
    output_size, AspectRatio, ImageFormat, RenderStudioState, QUALITIES, RESOLUTIONS,
};
use crate::services::localization::tr;
use eframe::egui::{self, Margin, Rect, Ui, UiBuilder};

/// Width of the panel in points.
pub const WIDTH: f32 = 320.0;

/// What the user asked of the panel.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PanelAction {
    /// Choose the output folder with the system dialog.
    Browse,
    /// Show the output folder in the file manager.
    OpenFolder,
}

/// Draws the panel into `area`.
pub fn show(ui: &mut Ui, area: Rect, state: &mut RenderStudioState) -> Option<PanelAction> {
    let mut action = None;
    ui.painter().rect_filled(area, 0.0, colors::SURFACE_LOW);
    let mut inner = ui.new_child(UiBuilder::new().max_rect(area));
    egui::ScrollArea::vertical().show(&mut inner, |ui| {
        egui::Frame::new()
            .inner_margin(Margin::same(16))
            .show(ui, |ui| {
                ui.set_width(WIDTH - 32.0);
                ui.spacing_mut().item_spacing.y = 10.0;
                text::label(
                    ui,
                    tr("render.output"),
                    typography::headline_sm(),
                    colors::ON_SURFACE,
                );
                settings(ui, state);
                action = destination(ui, state);
            });
    });
    action
}

fn settings(ui: &mut Ui, state: &mut RenderStudioState) {
    let half = (WIDTH - 40.0 - 8.0) / 2.0;
    let widths: Vec<String> = RESOLUTIONS.iter().map(|(w, _)| format!("{w} px")).collect();
    let widths: Vec<&str> = widths.iter().map(String::as_str).collect();
    let ratios: Vec<&str> = AspectRatio::ALL.iter().map(|a| a.name()).collect();
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 8.0;
        ui.vertical(|ui| {
            form::caption(ui, tr("render.resolution"));
            form::dropdown(
                ui,
                "render-resolution",
                &mut state.resolution,
                &widths,
                half,
            );
        });
        ui.vertical(|ui| {
            form::caption(ui, tr("render.aspect"));
            form::dropdown(ui, "render-aspect", &mut state.aspect, &ratios, half);
        });
    });
    let size = output_size(state.resolution, state.aspect);
    text::label(
        ui,
        format!("{}×{} px", size.0, size.1),
        typography::mono(11.0),
        colors::PRIMARY,
    );
    let qualities: Vec<&str> = QUALITIES.iter().map(|q| tr(q.label_key())).collect();
    form::caption(ui, tr("render.quality"));
    form::dropdown(
        ui,
        "render-quality",
        &mut state.quality,
        &qualities,
        WIDTH - 40.0,
    );
    let views: Vec<&str> = ViewPreset::ALL.iter().map(|v| tr(v.label_key())).collect();
    let mut view = ViewPreset::ALL
        .iter()
        .position(|v| *v == state.view)
        .unwrap_or(0);
    form::caption(ui, tr("render.view"));
    form::dropdown(ui, "render-view", &mut view, &views, WIDTH - 40.0);
    state.view = ViewPreset::ALL[view];
}

fn destination(ui: &mut Ui, state: &mut RenderStudioState) -> Option<PanelAction> {
    let mut action = None;
    let formats: Vec<&str> = ImageFormat::ALL.iter().map(|f| f.name()).collect();
    form::caption(ui, tr("render.format"));
    form::dropdown(
        ui,
        "render-format",
        &mut state.format,
        &formats,
        WIDTH - 40.0,
    );
    let format = ImageFormat::ALL[state.format.min(ImageFormat::ALL.len() - 1)];
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
    form::caption(ui, tr("render.folder"));
    form::text_field(ui, &mut state.output, "", WIDTH - 40.0);
    form::caption(ui, tr("render.file_name"));
    form::text_field(ui, &mut state.file_name, "", WIDTH - 40.0);
    let estimate = estimate_megabytes(format, output_size(state.resolution, state.aspect), 0.9);
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
    if let Some((message, error)) = &state.notice {
        let (level, title) = if *error {
            (Level::Error, tr("render.notice_error"))
        } else {
            (Level::Info, tr("render.notice_ok"))
        };
        notice::show(ui, level, title, message);
    }
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 8.0;
        let browse = Button::new(tr("render.browse"), ButtonStyle::Container)
            .icon(icons::FOLDER_OPEN, 16.0)
            .padding(12.0, 6.0);
        if browse.show(ui).clicked() {
            action = Some(PanelAction::Browse);
        }
        let open = Button::new(tr("render.open_folder"), ButtonStyle::Container)
            .icon(icons::IMAGE, 16.0)
            .padding(12.0, 6.0);
        if open.show(ui).clicked() {
            action = Some(PanelAction::OpenFolder);
        }
    });
    action
}
