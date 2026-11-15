//! The pieces of photo mode around the 3D view: the rail of sections, the action buttons of the section panel and
//! the bottom bar with the numbered camera slots.

use super::export;
use super::photo::SECTIONS;
use crate::commands::camera_link;
use crate::components::button::{Button, ButtonStyle};
use crate::components::{checkbox, text};
use crate::config::theme::{colors, typography};
use crate::models::navigation::PhotoSection;
use crate::models::AppState;
use crate::services::localization::{tr, trf};
use eframe::egui::{
    pos2, vec2, Align2, Color32, CornerRadius, CursorIcon, Rect, ScrollArea, Sense, Stroke,
    StrokeKind, Ui,
};

/// Slots the bar always shows (saved cameras first, then the save slot, then free ones).
const SLOTS: usize = 5;
const CARD: (f32, f32) = (164.0, 100.0);

/// The vertical rail of sections.
pub fn rail(ui: &mut Ui, app: &mut AppState) {
    let area = ui.max_rect();
    for (index, section) in PhotoSection::ALL.into_iter().enumerate() {
        let (glyph, key) = SECTIONS[index];
        let cell = Rect::from_min_size(
            pos2(area.left() + 4.0, area.top() + 8.0 + index as f32 * 70.0),
            vec2(area.width() - 8.0, 64.0),
        );
        let response = ui
            .interact(cell, ui.id().with(("rail", index)), Sense::click())
            .on_hover_cursor(CursorIcon::PointingHand);
        let active = app.photo_section == section;
        let painter = ui.painter();
        if active {
            painter.rect_filled(cell, CornerRadius::same(3), colors::SURFACE_HIGH);
            painter.rect_filled(
                Rect::from_min_size(cell.min, vec2(2.0, cell.height())),
                0.0,
                colors::PRIMARY_CONTAINER,
            );
        } else if response.hovered() {
            painter.rect_filled(cell, CornerRadius::same(3), colors::SURFACE_CONTAINER);
        }
        let ink = if active {
            colors::PRIMARY
        } else {
            colors::ON_SURFACE_VARIANT
        };
        text::paint_icon(
            painter,
            pos2(cell.center().x, cell.top() + 22.0),
            glyph,
            22.0,
            ink,
        );
        text::paint_text(
            painter,
            pos2(cell.center().x, cell.bottom() - 14.0),
            Align2::CENTER_CENTER,
            tr(key),
            typography::label_sm(),
            ink,
        );
        if response.clicked() {
            app.photo_section = section;
            app.panels.side_open = true;
        }
    }
}

/// The buttons above a section panel: preview, image (render), save, and pause (which a CPU render cannot do).
pub fn actions(ui: &mut Ui, app: &mut AppState) {
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 6.0;
        Button::new(tr("photo.pause"), ButtonStyle::Container)
            .disabled(tr("photo.pause_reason"))
            .show(ui);
        if Button::new(tr("photo.preview"), ButtonStyle::Container)
            .show(ui)
            .clicked()
        {
            export::preview_photo(app);
        }
        if Button::new(tr("photo.image_button"), ButtonStyle::Container)
            .show(ui)
            .clicked()
        {
            export::render_photos(app);
        }
        if Button::new(tr("photo.save"), ButtonStyle::Container)
            .show(ui)
            .clicked()
        {
            camera_link::save_current(app);
        }
    });
}

/// The bottom bar.
pub fn show(ui: &mut Ui, app: &mut AppState) {
    let area = ui.max_rect().shrink2(vec2(12.0, 8.0));
    let mut bar = ui.new_child(eframe::egui::UiBuilder::new().max_rect(area).layout(
        eframe::egui::Layout::left_to_right(eframe::egui::Align::Min),
    ));
    bar.spacing_mut().item_spacing.x = 12.0;
    bar.allocate_ui(vec2(150.0, area.height()), |ui| {
        ui.vertical(|ui| {
            ui.spacing_mut().item_spacing.y = 10.0;
            ui.add_space(14.0);
            if Button::new(tr("bar.new_photo"), ButtonStyle::Raised)
                .padding(14.0, 8.0)
                .show(ui)
                .clicked()
            {
                camera_link::save_current(app);
            }
            ui.horizontal(|ui| {
                checkbox::show(ui, &mut app.live_style);
                text::label(
                    ui,
                    tr("bar.live_style"),
                    typography::label_md(),
                    colors::ON_SURFACE,
                );
            });
        });
    });
    let render_width = 170.0;
    let slots_width = (area.width() - render_width - 150.0).max(200.0);
    let (mut recall, mut remove, mut save) = (None, None, false);
    bar.allocate_ui(vec2(slots_width, area.height()), |ui| {
        ScrollArea::horizontal()
            .auto_shrink([false, false])
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing.x = 10.0;
                    let total = SLOTS.max(app.cameras.len() + 1);
                    for index in 0..total {
                        if index < app.cameras.len() {
                            let name = app.cameras[index].name.clone();
                            let texture = app.thumbs.items.get(index).and_then(Option::as_ref);
                            match card(ui, index, &name, texture) {
                                Action::Recall => recall = Some(index),
                                Action::Remove => remove = Some(index),
                                Action::None => {}
                            }
                        } else if index == app.cameras.len() {
                            save |= empty(ui, index, tr("bar.save_view"), true);
                        } else {
                            empty(ui, index, tr("bar.free"), false);
                        }
                    }
                });
            });
    });
    bar.vertical(|ui| {
        ui.add_space(20.0);
        let ready = app
            .workspace
            .project
            .as_ref()
            .is_some_and(|project| !project.meshes.is_empty());
        let mut button = Button::new(tr("bar.render"), ButtonStyle::Solid)
            .colors(colors::TERTIARY, colors::ON_TERTIARY)
            .font(typography::label_lg())
            .padding(16.0, 12.0)
            .width(render_width);
        if !ready {
            button = button.disabled(tr("render.no_scene"));
        }
        if button.show(ui).clicked() {
            export::render_photos(app);
        }
        text::label(
            ui,
            trf("bar.photos", &[&app.cameras.len().max(1).to_string()]),
            typography::label_sm(),
            colors::OUTLINE,
        );
    });
    if save {
        camera_link::save_current(app);
    }
    if let Some(index) = recall {
        camera_link::recall(app, index);
    }
    if let Some(index) = remove {
        app.cameras.remove(index);
        app.thumbs.remove(index);
    }
}

enum Action {
    Recall,
    Remove,
    None,
}

fn number_badge(ui: &Ui, rect: Rect, number: usize) {
    let badge = Rect::from_min_size(rect.min + vec2(6.0, 6.0), vec2(20.0, 20.0));
    ui.painter().rect_filled(
        badge,
        CornerRadius::same(10),
        colors::with_alpha(colors::SURFACE_LOWEST, 220),
    );
    text::paint_text(
        ui.painter(),
        badge.center(),
        Align2::CENTER_CENTER,
        &(number + 1).to_string(),
        typography::label_md(),
        colors::PRIMARY,
    );
}

fn card(
    ui: &mut Ui,
    index: usize,
    name: &str,
    texture: Option<&eframe::egui::TextureHandle>,
) -> Action {
    let (rect, response) = ui.allocate_exact_size(vec2(CARD.0, CARD.1), Sense::click());
    let response = response.on_hover_cursor(CursorIcon::PointingHand);
    let picture = Rect::from_min_size(rect.min, vec2(rect.width(), rect.height() - 24.0));
    let painter = ui.painter();
    painter.rect_filled(rect, CornerRadius::same(3), colors::SURFACE_CONTAINER);
    match texture {
        Some(texture) => {
            painter.image(
                texture.id(),
                picture,
                Rect::from_min_max(pos2(0.0, 0.0), pos2(1.0, 1.0)),
                Color32::WHITE,
            );
        }
        None => {
            painter.rect_filled(picture, CornerRadius::same(3), colors::SURFACE_LOWEST);
        }
    }
    painter.rect_stroke(
        rect,
        CornerRadius::same(3),
        Stroke::new(
            1.0,
            if response.hovered() {
                colors::PRIMARY
            } else {
                colors::SURFACE_HIGHEST
            },
        ),
        StrokeKind::Inside,
    );
    text::paint_text(
        painter,
        pos2(rect.left() + 8.0, rect.bottom() - 12.0),
        Align2::LEFT_CENTER,
        name,
        typography::label_md(),
        colors::ON_SURFACE,
    );
    number_badge(ui, rect, index);
    let close = Rect::from_center_size(
        pos2(rect.right() - 14.0, rect.top() + 14.0),
        vec2(20.0, 20.0),
    );
    if response.hovered() {
        let remove = ui
            .interact(
                close,
                ui.id().with(("remove-camera", index)),
                Sense::click(),
            )
            .on_hover_text(tr("strip.remove"));
        ui.painter().circle_filled(
            close.center(),
            10.0,
            colors::with_alpha(colors::SURFACE_LOWEST, 220),
        );
        text::paint_icon(
            ui.painter(),
            close.center(),
            crate::components::icons::CLOSE,
            13.0,
            colors::ON_SURFACE,
        );
        if remove.clicked() {
            return Action::Remove;
        }
    }
    if response.clicked() {
        Action::Recall
    } else {
        Action::None
    }
}

fn empty(ui: &mut Ui, index: usize, label: &str, active: bool) -> bool {
    let (rect, response) = ui.allocate_exact_size(vec2(CARD.0, CARD.1), Sense::click());
    let painter = ui.painter();
    let fill = if active && response.hovered() {
        colors::SURFACE_HIGH
    } else {
        colors::SURFACE_LOW
    };
    painter.rect_filled(rect, CornerRadius::same(3), fill);
    painter.rect_stroke(
        rect,
        CornerRadius::same(3),
        Stroke::new(1.0, colors::SURFACE_HIGHEST),
        StrokeKind::Inside,
    );
    number_badge(ui, rect, index);
    let ink = if active {
        colors::PRIMARY
    } else {
        colors::OUTLINE
    };
    text::paint_text(
        ui.painter(),
        rect.center() + vec2(0.0, 6.0),
        Align2::CENTER_CENTER,
        label,
        typography::label_md(),
        ink,
    );
    if active {
        text::paint_icon(
            ui.painter(),
            rect.center() - vec2(0.0, 16.0),
            crate::components::icons::ADD_CIRCLE,
            24.0,
            ink,
        );
        return response
            .on_hover_cursor(CursorIcon::PointingHand)
            .on_hover_text(tr("strip.save_hint"))
            .clicked();
    }
    false
}
