//! Layout helpers for fixed-height rows.

use eframe::egui::{Align, Layout, Sense, Ui, UiBuilder, Vec2};

/// A row of the given height with one group pinned left and another pinned right, both centered vertically.
pub fn split_row(
    ui: &mut Ui,
    height: f32,
    left: impl FnOnce(&mut Ui),
    right: impl FnOnce(&mut Ui),
) {
    let (rect, _) = ui.allocate_exact_size(Vec2::new(ui.available_width(), height), Sense::hover());
    let mut left_ui = ui.new_child(
        UiBuilder::new()
            .max_rect(rect)
            .layout(Layout::left_to_right(Align::Center)),
    );
    left(&mut left_ui);
    let mut right_ui = ui.new_child(
        UiBuilder::new()
            .max_rect(rect)
            .layout(Layout::right_to_left(Align::Center)),
    );
    right(&mut right_ui);
}

/// A row with a group pinned left, a group centered in a box of `center_width`, and a group pinned right.
pub fn split_row_centered(
    ui: &mut Ui,
    height: f32,
    center_width: f32,
    left: impl FnOnce(&mut Ui),
    center: impl FnOnce(&mut Ui),
    right: impl FnOnce(&mut Ui),
) {
    let (rect, _) = ui.allocate_exact_size(Vec2::new(ui.available_width(), height), Sense::hover());
    let mut left_ui = ui.new_child(
        UiBuilder::new()
            .max_rect(rect)
            .layout(Layout::left_to_right(Align::Center)),
    );
    left(&mut left_ui);
    let middle =
        eframe::egui::Rect::from_center_size(rect.center(), Vec2::new(center_width, height));
    let mut center_ui = ui.new_child(
        UiBuilder::new()
            .max_rect(middle)
            .layout(Layout::left_to_right(Align::Center)),
    );
    center(&mut center_ui);
    let mut right_ui = ui.new_child(
        UiBuilder::new()
            .max_rect(rect)
            .layout(Layout::right_to_left(Align::Center)),
    );
    right(&mut right_ui);
}
