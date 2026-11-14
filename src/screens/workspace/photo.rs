//! Photo mode, laid out like a photo studio: a rail of sections on the far left (environment, image, camera,
//! material, quality, AI render), the panel of the chosen section next to it, and a bottom bar with the new-photo
//! button, five numbered camera slots (the saved cameras, with a picture captured from the view) and the render
//! button. Saving takes the real camera of the 3D view; recalling glides the view there.

use super::photo_bar;
use super::photo_panorama;
use super::{ModeSlots, Panel, Slot};
use crate::components::button::{Button, ButtonStyle};
use crate::components::{icons, text};
use crate::config::theme::{colors, typography};
use crate::features::camera::components::{filter_list, focus_panel, lens_panel};
use crate::features::environment::components::panel as environment_panel;
use crate::features::rendering::components::output_panel;
use crate::models::navigation::{PhotoSection, Screen};
use crate::models::AppState;
use crate::services::localization::tr;
use eframe::egui::{vec2, Margin, ScrollArea, Ui};

/// Height of the bottom bar.
const BOTTOM_HEIGHT: f32 = 214.0;
/// Width of the section panel.
const PANEL_WIDTH: f32 = 340.0;
/// Width of the section panel when the environment controls (three columns) are shown.
const WIDE_PANEL_WIDTH: f32 = 640.0;

/// Icon and label key of each section, in [`PhotoSection::ALL`] order.
pub const SECTIONS: [(char, &str); 7] = [
    (icons::SUN, "photo.environment"),
    (icons::IMAGE, "photo.image"),
    (icons::CAMERA, "photo.camera"),
    (icons::TEXTURE, "photo.material"),
    (icons::TUNE, "photo.quality"),
    (icons::VIEW_IN_AR, "photo.panorama"),
    (icons::BOLT, "photo.ai"),
];

/// Slots of photo mode.
pub fn slots() -> ModeSlots {
    ModeSlots {
        side: Slot {
            title: "panel.photo",
            glyph: icons::CAMERA,
            extent: |app| {
                if app.photo_section == PhotoSection::Environment {
                    WIDE_PANEL_WIDTH
                } else {
                    PANEL_WIDTH
                }
            },
            body: section as Panel,
        },
        bottom: Slot {
            title: "panel.cameras",
            glyph: icons::CAMERA,
            extent: |_| BOTTOM_HEIGHT,
            body: photo_bar::show as Panel,
        },
        rail: Some(photo_bar::rail as Panel),
        side_left: true,
        overlay: |_, _, _| {},
        plays: false,
    }
}

fn section(ui: &mut Ui, app: &mut AppState) {
    let area = ui.max_rect();
    let mut inner = ui.new_child(eframe::egui::UiBuilder::new().max_rect(area.shrink(10.0)));
    inner.set_clip_rect(area.intersect(ui.clip_rect()));
    inner.spacing_mut().item_spacing.y = 8.0;
    photo_bar::actions(&mut inner, app);
    let (_, key) = SECTIONS[PhotoSection::ALL
        .iter()
        .position(|s| *s == app.photo_section)
        .unwrap_or(0)];
    text::label(
        &mut inner,
        text::caps(tr(key)),
        typography::label_md(),
        colors::SECONDARY,
    );
    ScrollArea::vertical()
        .id_salt("photo-section")
        .auto_shrink([false, false])
        .show(&mut inner, |ui| {
            eframe::egui::Frame::new()
                .inner_margin(Margin::symmetric(0, 4))
                .show(ui, |ui| {
                    ui.spacing_mut().item_spacing.y = 10.0;
                    content(ui, app);
                });
        });
}

fn content(ui: &mut Ui, app: &mut AppState) {
    match app.photo_section {
        PhotoSection::Environment => {
            if environment_panel::show(ui, &mut app.environment) {
                ui.ctx().request_repaint();
            }
        }
        PhotoSection::Image => {
            filter_list::show(ui, &mut app.camera_studio);
        }
        PhotoSection::Camera => {
            let camera = &mut app.camera_studio.camera;
            lens_panel::show_lens(ui, camera);
            lens_panel::show_distance(ui, camera);
            focus_panel::show_focus(ui, camera);
            focus_panel::show_exposure(ui, camera);
        }
        PhotoSection::Material => {
            text::paragraph(
                ui,
                tr("cat.materials_hint"),
                typography::body_sm(),
                colors::OUTLINE,
                16.0,
            );
            if Button::new(tr("cat.open_materials"), ButtonStyle::Pale)
                .icon(icons::TEXTURE, 16.0)
                .show(ui)
                .clicked()
            {
                app.navigate(Screen::MaterialEditor);
            }
        }
        PhotoSection::Quality => {
            let rect = ui.max_rect();
            ui.allocate_space(vec2(rect.width(), 0.0));
            let _ = output_panel::show(ui, rect, &mut app.studio.state);
        }
        PhotoSection::Panorama => photo_panorama::show(ui, app),
        PhotoSection::AiRender => {
            text::label(
                ui,
                text::caps(tr("photo.ai_unavailable")),
                typography::label_md(),
                colors::ERROR,
            );
            text::paragraph(
                ui,
                tr("photo.ai_note"),
                typography::body_sm(),
                colors::OUTLINE,
                16.0,
            );
        }
    }
}
