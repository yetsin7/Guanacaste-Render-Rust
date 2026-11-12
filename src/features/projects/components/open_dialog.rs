//! The open-project dialog: recent files, typed path and the honest "format not available" notice.

use crate::components::{
    button::{Button, ButtonStyle},
    empty_state, form, icons, modal,
    notice::{self, Level},
    text,
};
use crate::config::theme::{colors, typography};
use crate::features::projects::models::{OpenState, RecentFile};
use crate::models::dialog::{Dialog, DialogOutcome, ErrorInfo};
use crate::services::localization::tr;
use eframe::egui::{vec2, Context, CornerRadius, Sense, Ui};

/// The error raised when opening is attempted.
pub const OPEN_ERROR: ErrorInfo = ErrorInfo {
    title: "error.open_title",
    message: "error.open_body",
    code: "GR-PROJECT-001",
};

/// Draws the dialog over `recent` files.
pub fn show(ctx: &Context, state: &mut OpenState, recent: &[RecentFile]) -> DialogOutcome {
    let mut outcome = DialogOutcome::Keep;
    let closed = modal::show(
        ctx,
        "open-dialog",
        tr("open.title"),
        icons::FOLDER_OPEN,
        560.0,
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
                    if row(ui, file, state.selected == Some(index)) {
                        state.selected = Some(index);
                    }
                }
                form::caption(ui, tr("open.path"));
                ui.horizontal(|ui| {
                    form::text_field(ui, &mut state.path, "C:/Proyectos/escena.grproj", 400.0);
                    Button::new(tr("open.browse"), ButtonStyle::Container)
                        .padding(12.0, 6.0)
                        .disabled(tr("dialog.no_engine"))
                        .show(ui);
                });
                notice::show(
                    ui,
                    Level::Info,
                    tr("notices.error_title"),
                    tr("notices.error_body"),
                );
            });
            modal::footer(ui, |ui| {
                if Button::new(tr("open.open"), ButtonStyle::Pale)
                    .padding(16.0, 6.0)
                    .show(ui)
                    .clicked()
                {
                    outcome = DialogOutcome::Open(Dialog::Error(OPEN_ERROR));
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

fn row(ui: &mut Ui, file: &RecentFile, selected: bool) -> bool {
    let (rect, response) = ui.allocate_exact_size(vec2(ui.available_width(), 44.0), Sense::click());
    let fill = if selected {
        colors::with_alpha(colors::PRIMARY_CONTAINER, 38)
    } else if response.hovered() {
        colors::SURFACE_HIGH
    } else {
        colors::SURFACE_CONTAINER
    };
    ui.painter().rect_filled(rect, CornerRadius::same(2), fill);
    if selected {
        crate::components::paint::hairline(ui.painter(), rect, 2, colors::PRIMARY);
    }
    text::paint_icon(
        ui.painter(),
        rect.left_center() + vec2(20.0, 0.0),
        icons::FOLDER_OPEN,
        18.0,
        colors::PRIMARY,
    );
    text::paint_text(
        ui.painter(),
        rect.left_center() + vec2(40.0, -9.0),
        eframe::egui::Align2::LEFT_CENTER,
        file.name,
        typography::mono_medium(11.0),
        colors::ON_SURFACE,
    );
    text::paint_text(
        ui.painter(),
        rect.left_center() + vec2(40.0, 9.0),
        eframe::egui::Align2::LEFT_CENTER,
        file.folder,
        typography::mono(10.0),
        colors::OUTLINE,
    );
    text::paint_text(
        ui.painter(),
        rect.right_center() + vec2(-12.0, -9.0),
        eframe::egui::Align2::RIGHT_CENTER,
        file.size,
        typography::label_sm(),
        colors::ON_SURFACE_VARIANT,
    );
    text::paint_text(
        ui.painter(),
        rect.right_center() + vec2(-12.0, 9.0),
        eframe::egui::Align2::RIGHT_CENTER,
        file.modified,
        typography::mono(10.0),
        colors::OUTLINE,
    );
    response.clicked()
}
