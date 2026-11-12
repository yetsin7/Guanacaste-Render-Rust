//! The output settings panel of the render studio.

use crate::components::{
    button::{Button, ButtonStyle},
    form, icons,
    layout::split_row,
    notice::{self, Level},
    slider::Slider,
    text,
};
use crate::config::theme::{colors, typography};
use crate::features::rendering::logic::resolution_label;
use crate::features::rendering::models::{RenderStudioState, RESOLUTIONS};
use crate::services::localization::tr;
use eframe::egui::{self, Margin, Rect, Ui, UiBuilder};

/// Width of the panel in points.
pub const WIDTH: f32 = 320.0;

/// Draws the panel into `area`.
pub fn show(ui: &mut Ui, area: Rect, state: &mut RenderStudioState) {
    ui.painter().rect_filled(area, 0.0, colors::SURFACE_LOW);
    let mut inner = ui.new_child(UiBuilder::new().max_rect(area));
    egui::Frame::new()
        .inner_margin(Margin::same(16))
        .show(&mut inner, |ui| {
            ui.set_width(WIDTH - 32.0);
            ui.spacing_mut().item_spacing.y = 12.0;
            text::label(
                ui,
                tr("render.output"),
                typography::headline_sm(),
                colors::ON_SURFACE,
            );
            let labels: Vec<String> = (0..RESOLUTIONS.len()).map(resolution_label).collect();
            let options: Vec<&str> = labels.iter().map(String::as_str).collect();
            form::caption(ui, tr("render.resolution"));
            form::dropdown(
                ui,
                "render-resolution",
                &mut state.resolution,
                &options,
                WIDTH - 40.0,
            );
            let samples = (16.0 + 4080.0 * state.samples).round() as u32;
            split_row(
                ui,
                18.0,
                |ui| form::caption(ui, tr("render.samples")),
                |ui| {
                    text::label(
                        ui,
                        samples.to_string(),
                        typography::label_md(),
                        colors::PRIMARY,
                    );
                },
            );
            Slider::precision().show(ui, &mut state.samples);
            form::switch_row(
                ui,
                tr("render.denoise"),
                tr("render.denoise_note"),
                &mut state.denoise,
            );
            form::caption(ui, tr("render.folder"));
            form::text_field(ui, &mut state.output, "", WIDTH - 40.0);
            notice::show(
                ui,
                Level::Info,
                tr("render.sample_title"),
                tr("render.no_engine"),
            );
            Button::new(tr("render.open_folder"), ButtonStyle::Container)
                .icon(icons::FOLDER_OPEN, 16.0)
                .padding(12.0, 6.0)
                .disabled(tr("dialog.no_engine"))
                .show(ui);
        });
}
