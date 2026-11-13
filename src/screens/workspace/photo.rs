//! Photo mode: the camera studio (lens, exposure and the filter stack) on the right and the strip of saved
//! cameras in the bottom drawer. Saving takes the real camera of the 3D view; recalling glides the view there.
//! The saved cameras live in the project document (`cameras`).

use super::{ModeSlots, Panel, Slot};
use crate::commands::camera_link;
use crate::components::{icons, text};
use crate::config::theme::{colors, typography};
use crate::features::camera::components::studio;
use crate::models::AppState;
use crate::services::localization::{tr, trf};
use eframe::egui::{
    pos2, vec2, Align2, CornerRadius, CursorIcon, Rect, ScrollArea, Sense, Stroke, StrokeKind, Ui,
};

const CARD: (f32, f32) = (150.0, 96.0);
const BOTTOM_HEIGHT: f32 = 156.0;

/// Slots of photo mode.
pub fn slots() -> ModeSlots {
    ModeSlots {
        side: Slot {
            title: "panel.effects",
            glyph: icons::TUNE,
            extent: |_| studio::PANEL_WIDTH,
            body: effects as Panel,
        },
        bottom: Slot {
            title: "panel.cameras",
            glyph: icons::CAMERA,
            extent: |_| BOTTOM_HEIGHT,
            body: strip as Panel,
        },
        overlay: |_, _, _| {},
        plays: false,
    }
}

fn effects(ui: &mut Ui, app: &mut AppState) {
    studio::show(ui, ui.max_rect(), &mut app.camera_studio);
}

enum CardAction {
    Recall,
    Remove,
    None,
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
                save |= save_card(ui);
                for (index, camera) in app.cameras.iter().enumerate() {
                    let detail = trf("strip.lens", &[&format!("{:.0}", camera.focal_mm)]);
                    match camera_card(ui, &camera.name, &detail) {
                        CardAction::Recall => recall = Some(index),
                        CardAction::Remove => remove = Some(index),
                        CardAction::None => {}
                    }
                }
                ui.add_space(12.0);
            });
        });
    if save {
        camera_link::save_current(app);
    }
    if let Some(index) = recall {
        camera_link::recall(app, index);
    }
    if let Some(index) = remove {
        app.cameras.remove(index);
    }
}

fn save_card(ui: &mut Ui) -> bool {
    let (rect, response) = ui.allocate_exact_size(vec2(CARD.0, CARD.1), Sense::click());
    let fill = if response.hovered() {
        colors::SURFACE_HIGH
    } else {
        colors::SURFACE_CONTAINER
    };
    ui.painter().rect_filled(rect, CornerRadius::same(3), fill);
    text::paint_icon(
        ui.painter(),
        pos2(rect.center().x, rect.center().y - 10.0),
        icons::ADD_CIRCLE,
        26.0,
        colors::PRIMARY,
    );
    text::paint_text(
        ui.painter(),
        pos2(rect.center().x, rect.center().y + 18.0),
        Align2::CENTER_CENTER,
        tr("strip.save_current"),
        typography::label_sm(),
        colors::PRIMARY,
    );
    response
        .on_hover_cursor(CursorIcon::PointingHand)
        .on_hover_text(tr("strip.save_hint"))
        .clicked()
}

fn camera_card(ui: &mut Ui, name: &str, detail: &str) -> CardAction {
    let (rect, response) = ui.allocate_exact_size(vec2(CARD.0, CARD.1), Sense::click());
    let response = response.on_hover_cursor(CursorIcon::PointingHand);
    let painter = ui.painter();
    let fill = if response.hovered() {
        colors::SURFACE_HIGH
    } else {
        colors::SURFACE_CONTAINER
    };
    painter.rect_filled(rect, CornerRadius::same(3), fill);
    painter.rect_stroke(
        rect,
        CornerRadius::same(3),
        Stroke::new(1.0, colors::SURFACE_HIGHEST),
        StrokeKind::Inside,
    );
    text::paint_icon(
        painter,
        pos2(rect.center().x, rect.top() + 34.0),
        icons::VIDEOCAM,
        26.0,
        colors::OUTLINE,
    );
    text::paint_text(
        painter,
        pos2(rect.left() + 10.0, rect.bottom() - 28.0),
        Align2::LEFT_CENTER,
        name,
        typography::label_md(),
        colors::ON_SURFACE,
    );
    text::paint_text(
        painter,
        pos2(rect.left() + 10.0, rect.bottom() - 12.0),
        Align2::LEFT_CENTER,
        detail,
        typography::mono(10.0),
        colors::OUTLINE,
    );
    let close = Rect::from_center_size(
        pos2(rect.right() - 12.0, rect.top() + 12.0),
        vec2(20.0, 20.0),
    );
    if response.hovered() {
        let remove = ui
            .interact(close, ui.id().with(("remove-camera", name)), Sense::click())
            .on_hover_text(tr("strip.remove"));
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
