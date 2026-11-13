//! The live picture of the render studio: paints the tiles of the running job into a texture as they arrive.

use crate::components::{progress_bar, tag::Tag, text};
use crate::config::theme::{colors, typography};
use crate::features::rendering::logic::time_text;
use crate::features::rendering::models::{JobInfo, JobStatus};
use crate::features::rendering::services::RenderQueue;
use crate::services::localization::{tr, trf};
use eframe::egui::{
    self, pos2, vec2, Color32, ColorImage, Context, Rect, TextureHandle, TextureOptions, Ui,
    UiBuilder,
};

/// Height of the progress strip at the bottom of the view.
const STRIP: f32 = 64.0;

/// Texture of the job being previewed and how much of its tile log was already painted.
#[derive(Default)]
pub struct PreviewView {
    texture: Option<TextureHandle>,
    job: Option<u64>,
    cursor: usize,
}

impl PreviewView {
    /// Paints the tiles that arrived since the last frame.
    pub fn update(&mut self, ctx: &Context, queue: &RenderQueue) {
        let Some(update) = queue.preview_since(self.job, self.cursor) else {
            return;
        };
        let (width, height) = (update.size.0 as usize, update.size.1 as usize);
        if update.reset || self.texture.is_none() {
            let blank = ColorImage::filled([width, height], Color32::from_gray(18));
            self.texture = Some(ctx.load_texture("render-preview", blank, TextureOptions::LINEAR));
        }
        if let Some(texture) = &mut self.texture {
            for tile in &update.tiles {
                let image = ColorImage::from_rgba_unmultiplied(
                    [tile.width as usize, tile.height as usize],
                    &tile.rgba,
                );
                texture.set_partial(
                    [tile.x as usize, tile.y as usize],
                    image,
                    TextureOptions::LINEAR,
                );
            }
        }
        self.job = Some(update.job);
        self.cursor = update.cursor;
    }

    /// Draws the picture fitted into `area`, with the progress of the job it belongs to under it.
    pub fn show(&self, ui: &mut Ui, area: Rect, jobs: &[JobInfo]) {
        ui.painter().rect_filled(area, 0.0, colors::SURFACE_LOWEST);
        let job = self.job.and_then(|id| jobs.iter().find(|j| j.id == id));
        let (Some(texture), Some(job)) = (&self.texture, job) else {
            let mut empty = ui.new_child(UiBuilder::new().max_rect(area.shrink(32.0)));
            empty.add_space(area.height() / 2.0 - 40.0);
            empty.vertical_centered(|ui| {
                text::icon(ui, crate::components::icons::IMAGE, 32.0, colors::OUTLINE);
                text::label(
                    ui,
                    tr("render.preview_empty"),
                    typography::body_md(),
                    colors::ON_SURFACE_VARIANT,
                );
            });
            return;
        };
        let strip = Rect::from_min_max(pos2(area.left(), area.bottom() - STRIP), area.max);
        let canvas = Rect::from_min_max(area.min, pos2(area.right(), strip.top())).shrink(16.0);
        let size = texture.size_vec2();
        let scale = (canvas.width() / size.x).min(canvas.height() / size.y);
        let fitted = Rect::from_center_size(canvas.center(), size * scale);
        ui.painter().image(
            texture.id(),
            fitted,
            Rect::from_min_max(pos2(0.0, 0.0), pos2(1.0, 1.0)),
            Color32::WHITE,
        );
        progress_strip(ui, strip, job);
    }
}

fn progress_strip(ui: &mut Ui, strip: Rect, job: &JobInfo) {
    let mut inner = ui.new_child(UiBuilder::new().max_rect(strip.shrink2(vec2(16.0, 8.0))));
    inner.spacing_mut().item_spacing.y = 4.0;
    let ink = match job.status {
        JobStatus::Done => colors::TERTIARY,
        JobStatus::Failed => colors::ERROR,
        JobStatus::Cancelled => colors::OUTLINE,
        _ => colors::PRIMARY,
    };
    inner.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 12.0;
        Tag::new(tr(job.status.label_key()), ink)
            .padding(8.0, 3.0)
            .show(ui);
        text::label(ui, &job.name, typography::label_md(), colors::ON_SURFACE);
        text::label(
            ui,
            trf(
                "render.progress",
                &[&format!("{:.0}", job.progress * 100.0)],
            ),
            typography::mono(11.0),
            colors::PRIMARY,
        );
        text::label(ui, time_text(job), typography::mono(11.0), colors::OUTLINE);
    });
    egui::Frame::NONE.show(&mut inner, |ui| {
        progress_bar::show(ui, job.progress, ink, 6.0);
    });
}
