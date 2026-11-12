//! Form controls: captions, text fields, dropdowns and switch rows.

use super::{layout::split_row, paint, text, toggle};
use crate::config::theme::{colors, typography};
use eframe::egui::{self, Margin, Response, RichText, TextEdit, Ui, Vec2};

/// A small uppercase caption above a field.
pub fn caption(ui: &mut Ui, label: &str) {
    text::label(
        ui,
        text::caps(label),
        typography::mono(10.0),
        colors::OUTLINE,
    );
}

/// A framed single-line text field `width` points wide.
pub fn text_field(ui: &mut Ui, value: &mut String, hint: &str, width: f32) -> Response {
    let (rect, _) = ui.allocate_exact_size(Vec2::new(width, 28.0), egui::Sense::hover());
    paint::framed(
        ui.painter(),
        rect,
        2,
        colors::SURFACE_LOWEST,
        colors::SURFACE_HIGHEST,
    );
    let mut inner = ui.new_child(
        egui::UiBuilder::new()
            .max_rect(rect.shrink2(Vec2::new(8.0, 0.0)))
            .layout(egui::Layout::left_to_right(egui::Align::Center)),
    );
    inner.add(
        TextEdit::singleline(value)
            .frame(egui::Frame::NONE)
            .font(typography::body_md())
            .text_color(colors::ON_SURFACE)
            .hint_text(RichText::new(hint).color(colors::OUTLINE))
            .margin(Margin::ZERO)
            .desired_width(width - 16.0),
    )
}

/// A dropdown over `options` showing the selected one; returns true when the selection changed.
pub fn dropdown(ui: &mut Ui, id: &str, selected: &mut usize, options: &[&str], width: f32) -> bool {
    let before = *selected;
    ui.scope(|ui| {
        let visuals = ui.visuals_mut();
        visuals.widgets.inactive.weak_bg_fill = colors::SURFACE_LOWEST;
        visuals.widgets.inactive.bg_stroke = egui::Stroke::NONE;
        visuals.widgets.hovered.weak_bg_fill = colors::SURFACE_CONTAINER;
        visuals.widgets.hovered.bg_stroke = egui::Stroke::NONE;
        egui::ComboBox::from_id_salt(id)
            .selected_text(RichText::new(options[*selected]).font(typography::body_md()))
            .width(width)
            .show_ui(ui, |ui| {
                for (index, option) in options.iter().enumerate() {
                    ui.selectable_value(
                        selected,
                        index,
                        RichText::new(*option).font(typography::body_md()),
                    );
                }
            });
    });
    *selected != before
}

/// A row with a title, an explanation and a switch on the right.
pub fn switch_row(ui: &mut Ui, title: &str, note: &str, on: &mut bool) {
    split_row(
        ui,
        36.0,
        |ui| {
            ui.vertical(|ui| {
                ui.spacing_mut().item_spacing.y = 0.0;
                text::label(ui, title, typography::body_md(), colors::ON_SURFACE);
                text::label(ui, note, typography::label_sm(), colors::OUTLINE);
            });
        },
        |ui| {
            toggle::show_sized(ui, on, Vec2::new(36.0, 20.0));
        },
    );
}

/// Draws `add` greyed out and inert, explaining `reason` when hovered.
pub fn disabled(ui: &mut Ui, reason: &str, add: impl FnOnce(&mut Ui)) {
    let scope = ui.scope(|ui| {
        ui.disable();
        add(ui);
    });
    let rect = scope.response.rect;
    ui.painter()
        .rect_filled(rect, 0.0, colors::with_alpha(colors::SURFACE_LOW, 120));
    ui.interact(
        rect,
        ui.id()
            .with(("disabled", rect.min.x as i32, rect.min.y as i32)),
        egui::Sense::hover(),
    )
    .on_hover_text(reason);
}
