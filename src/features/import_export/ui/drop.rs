//! Drag and drop of model files onto the window: the hover overlay and the dropped paths.

use crate::config::theme::{colors, typography};
use crate::services::localization::tr;
use eframe::egui::{Align2, Color32, Context, Id, LayerId, Order};
use std::path::PathBuf;

/// Paths of the files dropped on the window this frame.
pub fn dropped_paths(ctx: &Context) -> Vec<PathBuf> {
    ctx.input(|input| {
        input
            .raw
            .dropped_files
            .iter()
            .map(|file| file.path().to_path_buf())
            .collect()
    })
}

/// Paints a hint over the window while files are dragged above it.
pub fn overlay(ctx: &Context) {
    if ctx.input(|input| input.raw.hovered_files.is_empty()) {
        return;
    }
    let painter = ctx.layer_painter(LayerId::new(Order::Foreground, Id::new("drop-overlay")));
    let rect = ctx.content_rect();
    painter.rect_filled(rect, 0.0, Color32::from_black_alpha(170));
    painter.text(
        rect.center(),
        Align2::CENTER_CENTER,
        tr("import.drop_title"),
        typography::headline_sm(),
        colors::PRIMARY,
    );
}
