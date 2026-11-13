//! The video export dialog: output settings, the queue with progress and cancel, and the result of each job.

use crate::components::{
    button::{Button, ButtonStyle},
    checkbox, form, icons, modal, progress_bar, text,
};
use crate::config::theme::{colors, typography};
use crate::features::import_export::services::picker::FilePicker;
use crate::features::movie::logic::export_text::describe;
use crate::features::movie::models::Animation;
use crate::features::rendering::models::RESOLUTIONS;
use crate::features::rendering::services::ffmpeg;
use crate::features::rendering::services::frame_source::ColorFrameSource;
use crate::features::rendering::services::job::{ExportQueue, ExportRequest};
use crate::features::rendering::services::sequence::{clean_prefix, ExportSettings};
use crate::services::localization::{tr, trf};
use eframe::egui::{Context, Ui};
use std::path::PathBuf;

/// State of the export dialog and the queue behind it.
pub struct ExportPanel {
    /// Whether the dialog is shown.
    pub open: bool,
    /// Selected index into [`RESOLUTIONS`].
    pub resolution: usize,
    /// Name of the files and of the video.
    pub name: String,
    /// Chosen output folder.
    pub folder: Option<PathBuf>,
    /// Whether to encode an MP4 when ffmpeg is available.
    pub encode_video: bool,
    ffmpeg: Option<PathBuf>,
    picker: FilePicker,
    queue: ExportQueue,
}

impl Default for ExportPanel {
    fn default() -> Self {
        Self {
            open: false,
            resolution: 0,
            name: "recorrido".into(),
            folder: None,
            encode_video: true,
            ffmpeg: None,
            picker: FilePicker::default(),
            queue: ExportQueue::default(),
        }
    }
}

impl ExportPanel {
    /// Opens the dialog and looks for ffmpeg again.
    pub fn show_dialog(&mut self) {
        self.open = true;
        self.ffmpeg = ffmpeg::find_ffmpeg();
    }

    /// Whether the queue has work, so the screen keeps repainting.
    pub fn is_busy(&self) -> bool {
        self.queue.is_busy() || self.picker.is_open()
    }

    /// Advances the queue and the folder picker; call once per frame.
    pub fn update(&mut self) {
        if let Some(folder) = self.picker.poll() {
            self.folder = Some(folder);
        }
        self.queue.poll();
    }

    fn enqueue(&mut self, animation: &Animation) {
        let Some(folder) = self.folder.clone() else {
            return;
        };
        let (width, height) = RESOLUTIONS[self.resolution.min(RESOLUTIONS.len() - 1)];
        let settings = ExportSettings {
            folder,
            prefix: clean_prefix(&self.name),
            width,
            height,
            encode_video: self.encode_video,
        };
        // The photo renderer will replace this source when it exists.
        self.queue.enqueue(ExportRequest {
            animation: animation.clone(),
            source: Box::new(ColorFrameSource),
            settings,
            ffmpeg: self.ffmpeg.clone(),
        });
    }

    /// Draws the dialog when it is open.
    pub fn show(&mut self, ctx: &Context, animation: &Animation) {
        if !self.open {
            return;
        }
        let closed = modal::show(
            ctx,
            "movie-export",
            tr("movie.export_title"),
            icons::MOVIE,
            460.0,
            |ui| {
                modal::content(ui, |ui| self.settings(ui, animation));
                modal::divider(ui);
                modal::content(ui, |ui| self.progress(ui));
            },
        );
        if closed {
            self.open = false;
        }
    }

    fn settings(&mut self, ui: &mut Ui, animation: &Animation) {
        text::label(
            ui,
            trf(
                "movie.export_summary",
                &[
                    &animation.frame_count().to_string(),
                    &animation.fps.to_string(),
                    &format!("{:.1}", animation.video_duration()),
                ],
            ),
            typography::body_md(),
            colors::ON_SURFACE,
        );
        text::paragraph(
            ui,
            tr("movie.export_placeholder"),
            typography::body_sm(),
            colors::TERTIARY,
            16.0,
        );
        let sizes: Vec<String> = RESOLUTIONS
            .iter()
            .map(|(w, h)| format!("{w}×{h}"))
            .collect();
        let labels: Vec<&str> = sizes.iter().map(String::as_str).collect();
        form::caption(ui, tr("movie.export_resolution"));
        form::dropdown(
            ui,
            "movie-export-resolution",
            &mut self.resolution,
            &labels,
            200.0,
        );
        form::caption(ui, tr("movie.export_name"));
        form::text_field(ui, &mut self.name, tr("movie.export_name"), 260.0);
        form::caption(ui, tr("movie.export_folder"));
        ui.horizontal(|ui| {
            let shown = self.folder.as_ref().map_or_else(
                || tr("movie.export_no_folder").to_owned(),
                |f| f.display().to_string(),
            );
            text::label(
                ui,
                shown,
                typography::mono(11.0),
                colors::ON_SURFACE_VARIANT,
            );
            if Button::new(tr("movie.export_choose"), ButtonStyle::Raised)
                .font(typography::label_sm())
                .padding(10.0, 4.0)
                .show(ui)
                .clicked()
            {
                self.picker.pick_folder();
            }
        });
        self.encoding_option(ui);
        let ready = self.folder.is_some() && !animation.is_empty();
        let reason = if animation.is_empty() {
            tr("movie.export_nothing")
        } else {
            tr("movie.export_need_folder")
        };
        let mut button = Button::new(tr("movie.export_start"), ButtonStyle::Solid)
            .icon(icons::MOVIE, 15.0)
            .font(typography::label_md());
        if !ready {
            button = button.disabled(reason);
        }
        if button.show(ui).clicked() && ready {
            self.enqueue(animation);
        }
    }

    fn encoding_option(&mut self, ui: &mut Ui) {
        match &self.ffmpeg {
            Some(path) => {
                ui.horizontal(|ui| {
                    checkbox::show(ui, &mut self.encode_video);
                    text::label(
                        ui,
                        tr("movie.export_mp4"),
                        typography::body_md(),
                        colors::ON_SURFACE,
                    );
                });
                text::label(
                    ui,
                    trf("movie.ffmpeg_found", &[&path.display().to_string()]),
                    typography::mono(10.0),
                    colors::OUTLINE,
                );
            }
            None => {
                text::paragraph(
                    ui,
                    tr("movie.ffmpeg_missing"),
                    typography::body_sm(),
                    colors::ON_SURFACE_VARIANT,
                    16.0,
                );
            }
        }
    }

    fn progress(&mut self, ui: &mut Ui) {
        if let Some(job) = self.queue.current() {
            let (done, total) = job.progress();
            let fraction = if total == 0 {
                0.0
            } else {
                done as f32 / total as f32
            };
            let line = if job.is_encoding() {
                tr("movie.export_encoding").to_owned()
            } else {
                trf(
                    "movie.export_progress",
                    &[&done.to_string(), &total.to_string(), job.name()],
                )
            };
            text::label(ui, line, typography::body_sm(), colors::ON_SURFACE);
            progress_bar::show(ui, fraction, colors::PRIMARY, 6.0);
            ui.horizontal(|ui| {
                if Button::new(tr("movie.export_cancel"), ButtonStyle::Raised)
                    .font(typography::label_sm())
                    .padding(10.0, 4.0)
                    .show(ui)
                    .clicked()
                {
                    self.queue.cancel_all();
                }
                if self.queue.waiting() > 0 {
                    text::label(
                        ui,
                        trf("movie.export_waiting", &[&self.queue.waiting().to_string()]),
                        typography::label_sm(),
                        colors::OUTLINE,
                    );
                }
            });
        }
        for (name, end) in self.queue.history.iter().rev().take(4) {
            text::paragraph(
                ui,
                format!("{name}: {}", describe(end)),
                typography::body_sm(),
                colors::ON_SURFACE_VARIANT,
                16.0,
            );
        }
    }
}
