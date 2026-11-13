//! Start screen: new, open and import actions, the open project (to continue) and the recent projects.

use crate::commands;
use crate::commands::sample_model as sample;
use crate::components::button::{Button, ButtonStyle};
use crate::components::{empty_state, icons, key_cap::KeyCap, paint, text};
use crate::config::theme::{colors, typography};
use crate::features::projects::components::open_dialog::elide_middle;
use crate::features::projects::logic::describe::{age_label, size_label};
use crate::features::projects::services::recent::RecentInfo;
use crate::features::projects::services::util;
use crate::models::dialog::Dialog;
use crate::models::navigation::Screen;
use crate::models::AppState;
use crate::services::localization::{tr, trf};
use crate::services::textures::Placeholder;
use eframe::egui::{
    pos2, vec2, Align2, Color32, CornerRadius, CursorIcon, Margin, Rect, ScrollArea, Sense, Ui,
};

const MAX_WIDTH: f32 = 860.0;
const ROW_HEIGHT: f32 = 56.0;

/// Draws the start screen.
pub fn show(ui: &mut Ui, app: &mut AppState) {
    ScrollArea::vertical()
        .auto_shrink([false, false])
        .show(ui, |ui| {
            let total = ui.available_width();
            let width = (total - 48.0).min(MAX_WIDTH);
            ui.horizontal_top(|ui| {
                ui.add_space((total - width) / 2.0);
                ui.vertical(|ui| {
                    ui.set_width(width);
                    ui.spacing_mut().item_spacing.y = 14.0;
                    ui.add_space(40.0);
                    title(ui);
                    actions(ui);
                    sample_model(ui);
                    continue_card(ui, app);
                    recent(ui, app, width);
                    ui.add_space(16.0);
                    showcase(ui, app, width);
                    ui.add_space(24.0);
                });
            });
        });
}

fn title(ui: &mut Ui) {
    text::label(
        ui,
        tr("home.title"),
        typography::headline_lg(),
        colors::ON_SURFACE,
    );
    text::label(
        ui,
        tr("home.subtitle"),
        typography::body_md(),
        colors::ON_SURFACE_VARIANT,
    );
    ui.add_space(6.0);
}

fn actions(ui: &mut Ui) {
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 10.0;
        let items = [
            (
                icons::ADD_BOX,
                "home.new",
                "Ctrl+N",
                Dialog::NewScene,
                ButtonStyle::Pale,
            ),
            (
                icons::FOLDER_OPEN,
                "home.open",
                "Ctrl+O",
                Dialog::OpenProject,
                ButtonStyle::Raised,
            ),
            (
                icons::UPLOAD,
                "home.import",
                "Ctrl+I",
                Dialog::Import,
                ButtonStyle::Raised,
            ),
        ];
        for (glyph, key, shortcut, dialog, style) in items {
            let clicked = Button::new(tr(key), style)
                .icon(glyph, 18.0)
                .font(typography::label_lg())
                .padding(16.0, 10.0)
                .show(ui)
                .clicked();
            KeyCap::new(shortcut).small().show(ui);
            ui.add_space(8.0);
            if clicked {
                commands::request_dialog(ui.ctx(), dialog);
            }
        }
    });
}

fn continue_card(ui: &mut Ui, app: &mut AppState) {
    let Some(name) = app.workspace.name().map(str::to_owned) else {
        return;
    };
    eframe::egui::Frame::new()
        .fill(colors::SURFACE_LOW)
        .corner_radius(CornerRadius::same(4))
        .inner_margin(Margin::same(14))
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.vertical(|ui| {
                    ui.spacing_mut().item_spacing.y = 2.0;
                    text::label(
                        ui,
                        text::caps(tr("home.current")),
                        typography::label_sm(),
                        colors::OUTLINE,
                    );
                    text::label(ui, name, typography::headline_sm(), colors::ON_SURFACE);
                });
                ui.with_layout(
                    eframe::egui::Layout::right_to_left(eframe::egui::Align::Center),
                    |ui| {
                        if Button::new(tr("home.continue"), ButtonStyle::Solid)
                            .icon(icons::ARROW_FORWARD, 16.0)
                            .padding(14.0, 8.0)
                            .show(ui)
                            .clicked()
                        {
                            app.navigate(Screen::Workspace(app.mode));
                        }
                    },
                );
            });
        });
}

fn recent(ui: &mut Ui, app: &mut AppState, width: f32) {
    text::label(
        ui,
        text::caps(tr("home.recent")),
        typography::label_md(),
        colors::ON_SURFACE_VARIANT,
    );
    if app.workspace.recent.is_empty() {
        empty_state::show(
            ui,
            icons::FOLDER_OPEN,
            tr("home.empty_title"),
            tr("home.empty_body"),
            None,
        );
        return;
    }
    let now = util::now_unix();
    let mut open = None;
    ui.spacing_mut().item_spacing.y = 4.0;
    for info in &app.workspace.recent {
        if recent_row(ui, info, width, now) {
            open = Some(info.entry.path.clone());
        }
    }
    if let Some(path) = open {
        commands::request_open(ui.ctx(), path);
    }
}

fn recent_row(ui: &mut Ui, info: &RecentInfo, width: f32, now: u64) -> bool {
    let (rect, response) = ui.allocate_exact_size(vec2(width, ROW_HEIGHT), Sense::click());
    let response = response.on_hover_cursor(CursorIcon::PointingHand);
    let fill = if response.hovered() {
        colors::SURFACE_HIGH
    } else {
        colors::SURFACE_LOW
    };
    let painter = ui.painter();
    painter.rect_filled(rect, CornerRadius::same(3), fill);
    text::paint_icon(
        painter,
        pos2(rect.left() + 28.0, rect.center().y),
        icons::VIEW_IN_AR,
        22.0,
        colors::PRIMARY,
    );
    text::paint_text(
        painter,
        pos2(rect.left() + 56.0, rect.center().y - 9.0),
        Align2::LEFT_CENTER,
        &info.entry.name,
        typography::label_lg(),
        colors::ON_SURFACE,
    );
    let folder = info
        .entry
        .path
        .parent()
        .map(|parent| parent.display().to_string())
        .unwrap_or_default();
    let folder = elide_middle(&folder, ((width - 300.0) / 6.2).max(12.0) as usize);
    text::paint_text(
        painter,
        pos2(rect.left() + 56.0, rect.center().y + 10.0),
        Align2::LEFT_CENTER,
        &folder,
        typography::mono(10.0),
        colors::OUTLINE,
    );
    text::paint_text(
        painter,
        pos2(rect.right() - 14.0, rect.center().y - 8.0),
        Align2::RIGHT_CENTER,
        &age_label(now, info.modified),
        typography::label_md(),
        colors::ON_SURFACE_VARIANT,
    );
    text::paint_text(
        painter,
        pos2(rect.right() - 14.0, rect.center().y + 10.0),
        Align2::RIGHT_CENTER,
        &trf("home.size", &[&size_label(info.size)]),
        typography::mono(10.0),
        colors::OUTLINE,
    );
    response
        .on_hover_text(info.entry.path.display().to_string())
        .clicked()
}

fn note_id() -> eframe::egui::Id {
    eframe::egui::Id::new("home-sample-note")
}

/// The button that opens the Casa del Cobogó example, with a note when the file cannot be found.
fn sample_model(ui: &mut Ui) {
    ui.horizontal(|ui| {
        if Button::new(tr("home.sample"), ButtonStyle::Container)
            .icon(icons::VIEW_IN_AR, 16.0)
            .padding(12.0, 6.0)
            .show(ui)
            .on_hover_text(tr("home.sample_hint"))
            .clicked()
        {
            let note = match sample::locate() {
                Some(path) => {
                    commands::request_open(ui.ctx(), path);
                    String::new()
                }
                None => tr("home.sample_missing").to_owned(),
            };
            ui.data_mut(|data| data.insert_temp(note_id(), note));
        }
    });
    let note: String = ui.data(|data| data.get_temp(note_id())).unwrap_or_default();
    if !note.is_empty() {
        text::label(ui, note, typography::body_sm(), colors::WARNING);
    }
}

/// An illustration of what the software can do, labelled as a sample. It is never a project or a scene.
fn showcase(ui: &mut Ui, app: &mut AppState, width: f32) {
    text::label(
        ui,
        text::caps(tr("home.showcase")),
        typography::label_md(),
        colors::ON_SURFACE_VARIANT,
    );
    let height = (width * 0.34).clamp(120.0, 300.0);
    let (rect, _) = ui.allocate_exact_size(vec2(width, height), Sense::hover());
    let texture = app.textures.get(ui.ctx(), Placeholder::VillaRender).clone();
    let painter = ui.painter().with_clip_rect(rect);
    painter.rect_filled(rect, CornerRadius::same(4), colors::SURFACE_LOWEST);
    paint::image_cover(&painter, rect, &texture, vec2(0.5, 0.6), Color32::WHITE);
    let badge = Rect::from_min_size(rect.left_top() + vec2(10.0, 10.0), vec2(190.0, 22.0));
    painter.rect_filled(
        badge,
        CornerRadius::same(3),
        colors::with_alpha(colors::SURFACE_LOWEST, 225),
    );
    text::paint_text(
        &painter,
        badge.center(),
        Align2::CENTER_CENTER,
        tr("home.showcase_badge"),
        typography::label_sm(),
        colors::WARNING,
    );
    text::label(
        ui,
        tr("home.showcase_note"),
        typography::body_sm(),
        colors::OUTLINE,
    );
}
