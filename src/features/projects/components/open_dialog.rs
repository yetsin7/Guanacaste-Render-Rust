//! The open-project dialog: real recent files (path, size, age, thumbnail), a typed path and the native picker.
//! The format is decided by the file signature, never by the extension.

use crate::components::{
    button::{Button, ButtonStyle},
    empty_state, form, icons, modal,
    notice::{self, Level},
    text,
};
use crate::config::theme::{colors, typography};
use crate::features::projects::logic::describe::{age_label, size_label};
use crate::features::projects::models::OpenState;
use crate::features::projects::services::{recent::RecentInfo, store, util};
use crate::models::dialog::{Dialog, DialogOutcome, ErrorInfo};
use crate::services::localization::tr;
use eframe::egui::{
    pos2, vec2, Align2, ColorImage, Context, CornerRadius, Rect, Sense, TextureOptions, Ui,
};
use std::path::{Path, PathBuf};

/// The error shown when a file cannot be opened; the exact cause appears in the status message.
pub const OPEN_ERROR: ErrorInfo = ErrorInfo {
    title: "error.open_title",
    message: "error.open_body",
    code: "GR-PROJECT-001",
};

fn thumbnail(ctx: &Context, state: &mut OpenState, path: &Path) -> Option<eframe::egui::TextureId> {
    let entry = state
        .thumbnails
        .entry(path.to_path_buf())
        .or_insert_with(|| {
            let png = store::peek(path).ok()?.thumbnail?;
            let rgba = image::load_from_memory(&png).ok()?.to_rgba8();
            let size = [rgba.width() as usize, rgba.height() as usize];
            let color = ColorImage::from_rgba_unmultiplied(size, rgba.as_raw());
            Some(ctx.load_texture(path.display().to_string(), color, TextureOptions::LINEAR))
        });
    entry.as_ref().map(|texture| texture.id())
}

/// Draws the dialog over the `recent` files.
pub fn show(ctx: &Context, state: &mut OpenState, recent: &[RecentInfo]) -> DialogOutcome {
    if let Some(path) = state.picker.poll() {
        state.request = Some(path);
    }
    if state.picker.is_open() {
        ctx.request_repaint_after(std::time::Duration::from_millis(100));
    }
    let mut outcome = DialogOutcome::Keep;
    let now = util::now_unix();
    let closed = modal::show(
        ctx,
        "open-dialog",
        tr("open.title"),
        icons::FOLDER_OPEN,
        600.0,
        |ui| {
            modal::content(ui, |ui| {
                form::caption(ui, tr("open.recent"));
                if recent.is_empty() {
                    empty_state::show(
                        ui,
                        icons::FOLDER_OPEN,
                        tr("open.empty_title"),
                        tr("open.empty_body"),
                        None,
                    );
                }
                for (index, file) in recent.iter().enumerate() {
                    let texture = thumbnail(ctx, state, &file.entry.path);
                    let response = row(ui, file, texture, now, state.selected == Some(index));
                    if response.double_clicked() {
                        state.request = Some(file.entry.path.clone());
                    } else if response.clicked() {
                        state.selected = Some(index);
                        state.path = file.entry.path.display().to_string();
                    }
                }
                form::caption(ui, tr("open.path"));
                ui.horizontal(|ui| {
                    form::text_field(ui, &mut state.path, "C:/Proyectos/escena.gr", 440.0);
                    if Button::new(tr("open.browse"), ButtonStyle::Container)
                        .padding(12.0, 6.0)
                        .show(ui)
                        .clicked()
                    {
                        state.picker.open_project();
                    }
                });
                notice::show(
                    ui,
                    Level::Info,
                    tr("open.signature_title"),
                    tr("open.signature_body"),
                );
            });
            modal::footer(ui, |ui| {
                if Button::new(tr("open.open"), ButtonStyle::Pale)
                    .padding(16.0, 6.0)
                    .show(ui)
                    .clicked()
                    && !state.path.trim().is_empty()
                {
                    state.request = Some(PathBuf::from(state.path.trim()));
                }
                if Button::new(tr("open.import"), ButtonStyle::Container)
                    .icon(icons::UPLOAD, 16.0)
                    .padding(16.0, 6.0)
                    .show(ui)
                    .clicked()
                {
                    outcome = DialogOutcome::Open(Dialog::Import);
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

fn row(
    ui: &mut Ui,
    file: &RecentInfo,
    texture: Option<eframe::egui::TextureId>,
    now: u64,
    selected: bool,
) -> eframe::egui::Response {
    let (rect, response) = ui.allocate_exact_size(vec2(ui.available_width(), 48.0), Sense::click());
    let fill = if selected {
        colors::with_alpha(colors::PRIMARY_CONTAINER, 38)
    } else if response.hovered() {
        colors::SURFACE_HIGH
    } else {
        colors::SURFACE_CONTAINER
    };
    let painter = ui.painter();
    painter.rect_filled(rect, CornerRadius::same(2), fill);
    if selected {
        crate::components::paint::hairline(painter, rect, 2, colors::PRIMARY);
    }
    let thumb = Rect::from_min_size(rect.min + vec2(6.0, 6.0), vec2(72.0, 36.0));
    match texture {
        Some(id) => {
            painter.image(
                id,
                thumb,
                Rect::from_min_max(pos2(0.0, 0.0), pos2(1.0, 1.0)),
                eframe::egui::Color32::WHITE,
            );
        }
        None => {
            painter.rect_filled(thumb, CornerRadius::same(2), colors::SURFACE_HIGHEST);
            text::paint_icon(
                painter,
                thumb.center(),
                icons::FOLDER_OPEN,
                18.0,
                colors::PRIMARY,
            );
        }
    }
    let left = rect.left_center() + vec2(90.0, 0.0);
    let folder = file
        .entry
        .path
        .parent()
        .map(|p| p.display().to_string())
        .unwrap_or_default();
    let name = file
        .entry
        .path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or_default();
    text::paint_text(
        painter,
        left + vec2(0.0, -9.0),
        Align2::LEFT_CENTER,
        name,
        typography::mono_medium(11.0),
        colors::ON_SURFACE,
    );
    text::paint_text(
        painter,
        left + vec2(0.0, 9.0),
        Align2::LEFT_CENTER,
        &folder,
        typography::mono(10.0),
        colors::OUTLINE,
    );
    let right = rect.right_center() + vec2(-12.0, 0.0);
    text::paint_text(
        painter,
        right + vec2(0.0, -9.0),
        Align2::RIGHT_CENTER,
        &size_label(file.size),
        typography::label_sm(),
        colors::ON_SURFACE_VARIANT,
    );
    text::paint_text(
        painter,
        right + vec2(0.0, 9.0),
        Align2::RIGHT_CENTER,
        &age_label(now, file.modified),
        typography::mono(10.0),
        colors::OUTLINE,
    );
    response.on_hover_text(file.entry.path.display().to_string())
}
