//! Photo mode: effects (filters) in the right panel and the strip of saved cameras in the bottom drawer.

use super::{ModeSlots, Panel};
use crate::components::button::{Button, ButtonStyle};
use crate::components::slider::Slider;
use crate::components::{icons, paint, text, toggle};
use crate::config::theme::{colors, typography};
use crate::models::app_state::default_effects;
use crate::models::AppState;
use crate::services::localization::{tr, trf};
use eframe::egui::{
    pos2, vec2, Align2, CornerRadius, CursorIcon, Margin, Rect, ScrollArea, Sense, Stroke,
    StrokeKind, Ui,
};

const CARD: (f32, f32) = (136.0, 96.0);
const SIDE_WIDTH: f32 = 280.0;
const BOTTOM_HEIGHT: f32 = 156.0;

/// Slots of photo mode.
pub fn slots() -> ModeSlots {
    ModeSlots {
        side: ("panel.effects", icons::TUNE, SIDE_WIDTH, effects as Panel),
        bottom: (
            "panel.cameras",
            icons::CAMERA,
            BOTTOM_HEIGHT,
            strip as Panel,
        ),
        overlay: |_, _, _| {},
        plays: false,
    }
}

fn effects(ui: &mut Ui, app: &mut AppState) {
    ScrollArea::vertical()
        .auto_shrink([false, false])
        .show(ui, |ui| {
            eframe::egui::Frame::new()
                .inner_margin(Margin::same(12))
                .show(ui, |ui| {
                    ui.set_width(SIDE_WIDTH - 24.0);
                    ui.spacing_mut().item_spacing.y = 8.0;
                    for effect in &mut app.effects {
                        ui.horizontal(|ui| {
                            ui.spacing_mut().item_spacing.x = 8.0;
                            toggle::show(ui, &mut effect.enabled);
                            let ink = if effect.enabled {
                                colors::ON_SURFACE
                            } else {
                                colors::OUTLINE
                            };
                            text::label(ui, tr(effect.key), typography::label_md(), ink);
                        });
                        ui.add_enabled_ui(effect.enabled, |ui| {
                            Slider::precision().show(ui, &mut effect.amount);
                        });
                        ui.add_space(6.0);
                    }
                    if Button::new(tr("effect.reset"), ButtonStyle::Container)
                        .icon(icons::RESTORE, 16.0)
                        .show(ui)
                        .clicked()
                    {
                        app.effects = default_effects();
                    }
                });
        });
}

fn strip(ui: &mut Ui, app: &mut AppState) {
    let mut recall = None;
    let mut remove = None;
    let mut save = false;
    ScrollArea::horizontal()
        .auto_shrink([false, false])
        .show(ui, |ui| {
            ui.horizontal_centered(|ui| {
                ui.spacing_mut().item_spacing.x = 10.0;
                ui.add_space(12.0);
                if save_card(ui, app.cameras.has_room()) {
                    save = true;
                }
                for (index, camera) in app.cameras.saved.iter().enumerate() {
                    let selected = app.cameras.selected == Some(index);
                    match camera_card(ui, &camera.name, camera.pose.yaw, selected) {
                        CardAction::Recall => recall = Some(index),
                        CardAction::Remove => remove = Some(index),
                        CardAction::None => {}
                    }
                }
                ui.add_space(12.0);
            });
        });
    if save {
        app.save_current_view();
    }
    if let Some(index) = recall {
        app.recall_camera(index);
    }
    if let Some(index) = remove {
        app.cameras.remove(index);
    }
}

enum CardAction {
    Recall,
    Remove,
    None,
}

fn save_card(ui: &mut Ui, enabled: bool) -> bool {
    let (rect, response) = ui.allocate_exact_size(vec2(CARD.0, CARD.1), Sense::click());
    let hovered = enabled && response.hovered();
    let fill = if hovered {
        colors::SURFACE_HIGH
    } else {
        colors::SURFACE_CONTAINER
    };
    let ink = if enabled {
        colors::PRIMARY
    } else {
        colors::OUTLINE_VARIANT
    };
    ui.painter().rect_filled(rect, CornerRadius::same(3), fill);
    text::paint_icon(
        ui.painter(),
        pos2(rect.center().x, rect.center().y - 10.0),
        icons::ADD_CIRCLE,
        26.0,
        ink,
    );
    text::paint_text(
        ui.painter(),
        pos2(rect.center().x, rect.center().y + 18.0),
        Align2::CENTER_CENTER,
        tr("camera.save_current"),
        typography::label_sm(),
        ink,
    );
    let response = if enabled {
        response.on_hover_cursor(CursorIcon::PointingHand)
    } else {
        response.on_hover_text(tr("camera.full"))
    };
    enabled && response.clicked()
}

fn camera_card(ui: &mut Ui, name: &str, yaw: f32, selected: bool) -> CardAction {
    let (rect, response) = ui.allocate_exact_size(vec2(CARD.0, CARD.1), Sense::click());
    let response = response.on_hover_cursor(CursorIcon::PointingHand);
    let thumb = Rect::from_min_size(rect.min, vec2(rect.width(), CARD.1 - 30.0));
    let painter = ui.painter();
    painter.rect_filled(rect, CornerRadius::same(3), colors::SURFACE_CONTAINER);
    paint::vertical_gradient(
        painter,
        thumb,
        colors::with_alpha(colors::SECONDARY_CONTAINER, 90),
        colors::SURFACE_LOWEST,
    );
    text::paint_icon(
        painter,
        thumb.center(),
        icons::VIDEOCAM,
        24.0,
        colors::OUTLINE,
    );
    text::paint_text(
        painter,
        pos2(rect.left() + 8.0, thumb.bottom() + 9.0),
        Align2::LEFT_CENTER,
        name,
        typography::label_md(),
        colors::ON_SURFACE,
    );
    text::paint_text(
        painter,
        pos2(rect.left() + 8.0, thumb.bottom() + 22.0),
        Align2::LEFT_CENTER,
        &trf("camera.heading", &[&format!("{yaw:.0}")]),
        typography::mono(10.0),
        colors::OUTLINE,
    );
    if selected {
        painter.rect_stroke(
            rect,
            CornerRadius::same(3),
            Stroke::new(2.0, colors::PRIMARY_CONTAINER),
            StrokeKind::Inside,
        );
    }
    let close = Rect::from_center_size(
        pos2(rect.right() - 12.0, rect.top() + 12.0),
        vec2(20.0, 20.0),
    );
    if response.hovered() {
        let remove = ui
            .interact(close, ui.id().with(("remove-camera", name)), Sense::click())
            .on_hover_text(tr("camera.remove"));
        ui.painter().circle_filled(
            close.center(),
            10.0,
            colors::with_alpha(colors::SURFACE_LOWEST, 220),
        );
        text::paint_icon(
            ui.painter(),
            close.center(),
            icons::CLOSE,
            13.0,
            colors::ON_SURFACE,
        );
        if remove.clicked() {
            return CardAction::Remove;
        }
    }
    if response.clicked() {
        CardAction::Recall
    } else {
        CardAction::None
    }
}
