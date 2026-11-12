//! The 28 px custom title bar of the frameless window: product and layer on the left; telemetry pill, avatar and
//! window buttons on the right. Dragging moves the window and a double click toggles maximize.

use super::{icons, tag, tag::Tag, text};
use crate::config::theme::{colors, typography};
use crate::services::localization::tr;
use eframe::egui::{Align, CornerRadius, Layout, Sense, Ui, Vec2, ViewportCommand};

/// Height of the title bar in points.
pub const HEIGHT: f32 = 28.0;

/// Text shown by the title bar.
pub struct TitleBarInfo<'a> {
    /// Frame-rate readout, e.g. `"37 FPS"`.
    pub fps: &'a str,
    /// Resolution readout, e.g. `"| 4.2K"`.
    pub resolution: &'a str,
    /// Whether the frame-rate pill is shown.
    pub show_fps: bool,
}

/// Draws the title bar inside `ui`, which must be exactly [`HEIGHT`] tall.
pub fn show(ui: &mut Ui, info: &TitleBarInfo) -> Option<eframe::egui::Pos2> {
    let mut menu = None;
    let bar = ui.max_rect();
    let drag = ui.interact(bar, ui.id().with("title-drag"), Sense::click_and_drag());
    let context = ui.ctx().clone();
    if drag.drag_started() {
        context.send_viewport_cmd(ViewportCommand::StartDrag);
    }
    if drag.double_clicked() {
        toggle_maximized(&context);
    }
    ui.horizontal_centered(|ui| {
        ui.spacing_mut().item_spacing.x = 0.0;
        ui.add_space(8.0);
        text::icon(ui, icons::LOGO, 16.0, colors::PRIMARY);
        ui.add_space(4.0);
        text::label(
            ui,
            text::caps(tr("common.product")),
            typography::label_md(),
            colors::ON_SURFACE,
        );
        ui.add_space(10.0);
        divider(ui);
        ui.add_space(10.0);
        Tag::new(tr("common.layer"), colors::ON_SURFACE_VARIANT)
            .lead_icon(icons::LAYERS, 12.0, colors::OUTLINE)
            .trail_icon(icons::EXPAND_MORE, 12.0, colors::OUTLINE)
            .gap(2.0)
            .padding(4.0, 2.0)
            .fill(colors::SURFACE_CONTAINER)
            .border(colors::SURFACE_HIGHEST)
            .show(ui);
        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
            ui.spacing_mut().item_spacing.x = 0.0;
            window_buttons(ui);
            ui.add_space(2.0);
            let (rect, _) = ui.allocate_exact_size(Vec2::new(1.0, HEIGHT), Sense::hover());
            ui.painter().rect_filled(rect, 0.0, colors::SURFACE_HIGHEST);
            ui.add_space(8.0);
            if let Some(corner) = avatar(ui) {
                menu = Some(corner);
            }
            ui.add_space(8.0);
            if info.show_fps {
                tag::fps_pill(ui, info.fps, info.resolution);
            }
        });
    });
    menu
}

fn toggle_maximized(context: &eframe::egui::Context) {
    let maximized = context.input(|input| input.viewport().maximized.unwrap_or(false));
    context.send_viewport_cmd(ViewportCommand::Maximized(!maximized));
}

fn divider(ui: &mut Ui) {
    let (rect, _) = ui.allocate_exact_size(Vec2::new(1.0, 12.0), Sense::hover());
    ui.painter().rect_filled(rect, 0.0, colors::OUTLINE_VARIANT);
}

/// The avatar opens the application menu; returns the menu anchor (bottom-right of the avatar) when clicked.
fn avatar(ui: &mut Ui) -> Option<eframe::egui::Pos2> {
    let (rect, response) = ui.allocate_exact_size(Vec2::splat(20.0), Sense::click());
    let response = response
        .on_hover_cursor(eframe::egui::CursorIcon::PointingHand)
        .on_hover_text(tr("menu.account"));
    let fill = if response.hovered() {
        colors::PRIMARY_CONTAINER
    } else {
        colors::PRIMARY
    };
    ui.painter().circle_filled(rect.center(), 10.0, fill);
    text::paint_icon(
        ui.painter(),
        rect.center(),
        icons::PERSON,
        14.0,
        colors::ON_PRIMARY,
    );
    response
        .clicked()
        .then(|| rect.right_bottom() + Vec2::new(6.0, 6.0))
}

/// Minimize, maximize/restore and close, laid out right to left.
fn window_buttons(ui: &mut Ui) {
    let context = ui.ctx().clone();
    let maximized = context.input(|input| input.viewport().maximized.unwrap_or(false));
    let buttons = [
        (icons::CLOSE, "window.close", ViewportCommand::Close, true),
        (
            if maximized {
                icons::RESTORE_WINDOW
            } else {
                icons::MAXIMIZE
            },
            if maximized {
                "window.restore"
            } else {
                "window.maximize"
            },
            ViewportCommand::Maximized(!maximized),
            false,
        ),
        (
            icons::MINIMIZE,
            "window.minimize",
            ViewportCommand::Minimized(true),
            false,
        ),
    ];
    for (glyph, tooltip, command, danger) in buttons {
        let (rect, response) = ui.allocate_exact_size(Vec2::new(24.0, HEIGHT), Sense::click());
        let hovered = response.hovered();
        if hovered {
            let fill = if danger {
                colors::ERROR_CONTAINER
            } else {
                colors::SURFACE_CONTAINER
            };
            ui.painter().rect_filled(rect, CornerRadius::ZERO, fill);
        }
        let color = match (hovered, danger) {
            (true, true) => colors::ON_SURFACE,
            (true, false) => colors::ON_SURFACE,
            _ => colors::OUTLINE,
        };
        text::paint_icon(ui.painter(), rect.center(), glyph, 14.0, color);
        if response.on_hover_text(tr(tooltip)).clicked() {
            context.send_viewport_cmd(command);
        }
    }
}
