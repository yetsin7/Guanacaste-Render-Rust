//! The material list with its actions: new, preset, duplicate, rename and delete.

use crate::components::{
    button::{Button, ButtonStyle},
    form, icons, text,
};
use crate::config::theme::{colors, typography};
use crate::features::materials::logic::{editor, presets};
use crate::features::materials::models::MaterialState;
use crate::services::localization::{tr, trf};
use eframe::egui::{
    self, pos2, vec2, Color32, CornerRadius, CursorIcon, Margin, Rect, ScrollArea, Sense, Ui,
    UiBuilder,
};

/// Width of the list panel in points.
pub const WIDTH: f32 = 280.0;
const ROW: f32 = 34.0;

fn swatch(color: [f32; 4]) -> Color32 {
    let byte = |c: f32| (c.clamp(0.0, 1.0) * 255.0 + 0.5) as u8;
    Color32::from_rgb(byte(color[0]), byte(color[1]), byte(color[2]))
}

/// Draws the panel into `area`.
pub fn show(ui: &mut Ui, area: Rect, state: &mut MaterialState) {
    ui.painter().rect_filled(area, 0.0, colors::SURFACE_LOW);
    let footer_height = 112.0;
    let footer = Rect::from_min_max(pos2(area.left(), area.bottom() - footer_height), area.max);
    let body = Rect::from_min_max(area.min, pos2(area.right(), footer.top()));
    let mut list_ui = ui.new_child(UiBuilder::new().max_rect(body));
    ScrollArea::vertical()
        .auto_shrink([false, false])
        .show(&mut list_ui, |ui| {
            egui::Frame::new()
                .inner_margin(Margin::same(8))
                .show(ui, |ui| rows(ui, state));
        });
    ui.painter().rect_filled(footer, 0.0, colors::SURFACE_HIGH);
    let mut foot = ui.new_child(UiBuilder::new().max_rect(footer.shrink(8.0)));
    actions(&mut foot, state);
}

fn rows(ui: &mut Ui, state: &mut MaterialState) {
    ui.spacing_mut().item_spacing.y = 2.0;
    let shown = editor::visible(state);
    if state.library.is_empty() {
        text::paragraph(
            ui,
            tr("material.list_empty"),
            typography::body_sm(),
            colors::ON_SURFACE_VARIANT,
            18.0,
        );
    } else if shown.is_empty() {
        text::paragraph(
            ui,
            tr("material.list_no_match"),
            typography::body_sm(),
            colors::ON_SURFACE_VARIANT,
            18.0,
        );
    }
    let mut clicked = None;
    for index in shown {
        let Some(material) = state.library.get(index) else {
            continue;
        };
        let (rect, response) =
            ui.allocate_exact_size(vec2(ui.available_width(), ROW), Sense::click());
        let selected = index == state.selected;
        let fill = if selected {
            colors::SURFACE_HIGHEST
        } else if response.hovered() {
            colors::SURFACE_CONTAINER
        } else {
            Color32::TRANSPARENT
        };
        ui.painter().rect_filled(rect, CornerRadius::same(2), fill);
        ui.painter().circle_filled(
            pos2(rect.left() + 18.0, rect.center().y),
            9.0,
            swatch(material.base_color),
        );
        ui.painter().circle_stroke(
            pos2(rect.left() + 18.0, rect.center().y),
            9.0,
            egui::Stroke::new(1.0, colors::OUTLINE_VARIANT),
        );
        let editing = selected && state.renaming.is_some();
        if !editing {
            text::paint_text(
                ui.painter(),
                pos2(rect.left() + 36.0, rect.center().y),
                egui::Align2::LEFT_CENTER,
                &material.name,
                typography::body_md(),
                if selected {
                    colors::ON_SURFACE
                } else {
                    colors::ON_SURFACE_VARIANT
                },
            );
            let mut x = rect.right() - 12.0;
            for glyph in badges(material) {
                text::paint_icon(
                    ui.painter(),
                    pos2(x, rect.center().y),
                    glyph,
                    13.0,
                    colors::OUTLINE,
                );
                x -= 18.0;
            }
        }
        if response.on_hover_cursor(CursorIcon::PointingHand).clicked() {
            clicked = Some(index);
        }
        if editing {
            rename_field(ui, rect, state);
        }
    }
    if let Some(index) = clicked {
        editor::select(state, index);
    }
}

fn badges(material: &crate::features::materials::models::Material) -> Vec<char> {
    let mut glyphs = Vec::new();
    if material.maps.albedo.is_some() || material.maps.normal.is_some() {
        glyphs.push(icons::TEXTURE);
    }
    if material.is_transparent() {
        glyphs.push(icons::VISIBILITY);
    }
    if material.is_emissive() {
        glyphs.push(icons::LIGHT);
    }
    glyphs
}

fn rename_field(ui: &mut Ui, rect: Rect, state: &mut MaterialState) {
    let mut buffer = state.renaming.take().unwrap_or_default();
    let mut field = ui.new_child(UiBuilder::new().max_rect(Rect::from_min_max(
        pos2(rect.left() + 32.0, rect.top() + 3.0),
        pos2(rect.right() - 4.0, rect.bottom() - 3.0),
    )));
    let response = form::text_field(
        &mut field,
        &mut buffer,
        tr("material.name_hint"),
        rect.width() - 40.0,
    );
    state.renaming = Some(buffer);
    let enter = ui.input(|i| i.key_pressed(egui::Key::Enter));
    let escape = ui.input(|i| i.key_pressed(egui::Key::Escape));
    if escape {
        state.renaming = None;
    } else if enter || response.lost_focus() {
        editor::commit_rename(state);
    }
}

fn actions(ui: &mut Ui, state: &mut MaterialState) {
    ui.spacing_mut().item_spacing = vec2(6.0, 6.0);
    let has = state.current().is_some();
    ui.horizontal(|ui| {
        if Button::new(tr("material.new"), ButtonStyle::Pale)
            .icon(icons::ADD, 14.0)
            .font(typography::label_sm())
            .padding(10.0, 4.0)
            .show(ui)
            .clicked()
        {
            editor::add_new(state, tr("material.new_name"));
        }
        ui.menu_button(tr("material.add_preset"), |ui| {
            ScrollArea::vertical().max_height(320.0).show(ui, |ui| {
                for (index, preset) in presets::all().iter().enumerate() {
                    if ui.button(&preset.name).clicked() {
                        editor::add_preset(state, index);
                        ui.close();
                    }
                }
            });
        });
    });
    ui.horizontal(|ui| {
        let small = |text: &str, glyph: char| {
            Button::new(text, ButtonStyle::Raised)
                .icon(glyph, 13.0)
                .font(typography::label_sm())
                .padding(8.0, 3.0)
        };
        let reason = tr("material.none_selected");
        let duplicate = small(tr("material.duplicate"), icons::COPY);
        let rename = small(tr("material.rename"), icons::EDIT_NOTE);
        if (if has {
            duplicate
        } else {
            duplicate.disabled(reason)
        })
        .show(ui)
        .clicked()
        {
            editor::duplicate_selected(state);
        }
        if (if has { rename } else { rename.disabled(reason) })
            .show(ui)
            .clicked()
        {
            editor::begin_rename(state);
        }
    });
    let label = if state.confirm_delete {
        trf(
            "material.delete_confirm",
            &[state.current().map_or("", |m| m.name.as_str())],
        )
    } else {
        tr("material.delete").to_owned()
    };
    let delete = Button::new(label, ButtonStyle::Raised)
        .icon(icons::DELETE, 13.0)
        .font(typography::label_sm())
        .padding(8.0, 3.0)
        .colors(
            if state.confirm_delete {
                colors::ERROR_CONTAINER
            } else {
                colors::SURFACE_HIGHEST
            },
            colors::ON_SURFACE,
        );
    let delete = if has {
        delete
    } else {
        delete.disabled(tr("material.none_selected"))
    };
    if delete.show(ui).clicked() {
        if state.confirm_delete {
            editor::delete_selected(state);
        } else {
            state.confirm_delete = true;
        }
    }
}
