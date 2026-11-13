//! A floating palette of icon tools over the 3D view: a top row of shortcuts, a matrix of tool buttons below it,
//! a large green import button on the left and two extra buttons on the right.

use super::{frames, icon_button::IconButton, text};
use crate::config::theme::{colors, typography};
use eframe::egui::{self, Align2, CornerRadius, Id, Margin, Order, Pos2, Sense, Ui, Vec2};

/// One icon button.
pub struct Item {
    /// Icon glyph.
    pub glyph: char,
    /// Tooltip (already localized).
    pub tip: &'static str,
    /// Whether the tool is switched on.
    pub active: bool,
}

/// What a palette shows.
pub struct Palette<'a> {
    /// The row of shortcuts above the matrix.
    pub top: &'a [Item],
    /// The matrix of tools, filled row by row.
    pub grid: &'a [Item],
    /// Buttons per row of the matrix.
    pub columns: usize,
    /// Label of the large green button.
    pub import: &'a str,
    /// Tooltip of the large green button.
    pub import_tip: &'a str,
    /// Buttons stacked at the right of the matrix.
    pub right: &'a [Item],
}

/// What the user pressed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Click {
    /// A button of the top row.
    Top(usize),
    /// A button of the matrix.
    Grid(usize),
    /// The green button.
    Import,
    /// A button of the right column.
    Right(usize),
}

const SIDE: f32 = 28.0;

fn item(ui: &mut Ui, it: &Item) -> bool {
    IconButton::new(it.glyph)
        .sizes(16.0, SIDE)
        .fill(Some(colors::SURFACE_CONTAINER))
        .active(it.active)
        .tooltip(it.tip)
        .show(ui)
        .clicked()
}

/// Draws the palette with its top-left corner at `corner` and returns what was pressed.
pub fn show(
    ctx: &egui::Context,
    id: &'static str,
    corner: Pos2,
    palette: &Palette,
) -> Option<Click> {
    let mut clicked = None;
    egui::Area::new(Id::new(id))
        .order(Order::Middle)
        .pivot(Align2::LEFT_TOP)
        .fixed_pos(corner)
        .show(ctx, |ui| {
            frames::overlay(6)
                .inner_margin(Margin::same(6))
                .show(ui, |ui| {
                    ui.spacing_mut().item_spacing = Vec2::splat(4.0);
                    ui.vertical(|ui| {
                        ui.horizontal(|ui| {
                            for (index, it) in palette.top.iter().enumerate() {
                                if item(ui, it) {
                                    clicked = Some(Click::Top(index));
                                }
                            }
                        });
                        ui.horizontal(|ui| {
                            let rows = palette.grid.len().div_ceil(palette.columns.max(1));
                            let height = rows as f32 * (SIDE + 4.0) - 4.0;
                            if import_button(ui, palette, height) {
                                clicked = Some(Click::Import);
                            }
                            ui.vertical(|ui| {
                                let columns = palette.columns.max(1);
                                for (row_index, row) in palette.grid.chunks(columns).enumerate() {
                                    ui.horizontal(|ui| {
                                        for (column, it) in row.iter().enumerate() {
                                            if item(ui, it) {
                                                clicked =
                                                    Some(Click::Grid(row_index * columns + column));
                                            }
                                        }
                                    });
                                }
                            });
                            ui.vertical(|ui| {
                                for (index, it) in palette.right.iter().enumerate() {
                                    if item(ui, it) {
                                        clicked = Some(Click::Right(index));
                                    }
                                }
                            });
                        });
                    });
                });
        });
    clicked
}

fn import_button(ui: &mut Ui, palette: &Palette, height: f32) -> bool {
    let (rect, response) = ui.allocate_exact_size(Vec2::new(84.0, height), Sense::click());
    let fill = if response.hovered() {
        colors::TERTIARY
    } else {
        colors::with_alpha(colors::TERTIARY, 220)
    };
    ui.painter().rect_filled(rect, CornerRadius::same(4), fill);
    text::paint_text(
        ui.painter(),
        rect.center(),
        Align2::CENTER_CENTER,
        palette.import,
        typography::label_lg(),
        colors::ON_TERTIARY,
    );
    response
        .on_hover_cursor(egui::CursorIcon::PointingHand)
        .on_hover_text(palette.import_tip)
        .clicked()
}
