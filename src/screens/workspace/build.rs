//! Build mode. Bottom: the category tabs (nature, water and terrain, objects, lights, materials, sky; the sky
//! panel opens wide here). Right: the panel of the selected category over the ground section.

use super::{ground_section, ModeSlots, Panel, Slot};
use crate::commands::garden;
use crate::components::button::{Button, ButtonStyle};
use crate::components::choice_row::{self, Choice};
use crate::components::{icons, text};
use crate::config::theme::{colors, typography};
use crate::features::build::components::objects_panel;
use crate::features::environment::components::panel as environment_panel;
use crate::features::landscape::components::panel as landscape_panel;
use crate::features::landscape::models::Tool;
use crate::features::lighting::components::panel as lighting_panel;
use crate::models::navigation::{BuildCategory, Screen};
use crate::models::AppState;
use crate::services::localization::tr;
use eframe::egui::{pos2, Rect, ScrollArea, Ui, UiBuilder};

/// Width of the right panel.
const SIDE_WIDTH: f32 = 340.0;
/// Height of the drawer when it only holds the category tabs.
const TABS_HEIGHT: f32 = 84.0;
/// Height of the drawer when the sky panel is open.
const SKY_HEIGHT: f32 = 330.0;

/// Icon and label key of each category, in [`BuildCategory::ALL`] order.
const CATEGORIES: [(char, &str); 6] = [
    (icons::NATURE, "cat.nature"),
    (icons::TERRAIN, "cat.water_terrain"),
    (icons::VIEW_IN_AR, "cat.objects"),
    (icons::LIGHT, "cat.lights"),
    (icons::TEXTURE, "cat.materials"),
    (icons::CLOUD, "cat.sky"),
];

/// Slots of build mode.
pub fn slots() -> ModeSlots {
    ModeSlots {
        side: Slot {
            title: "panel.properties",
            glyph: icons::TUNE,
            extent: |_| SIDE_WIDTH,
            body: side as Panel,
        },
        bottom: Slot {
            title: "panel.categories",
            glyph: icons::GRID_VIEW,
            extent: |app| {
                if app.category == BuildCategory::Sky {
                    SKY_HEIGHT
                } else {
                    TABS_HEIGHT
                }
            },
            body: bottom as Panel,
        },
        overlay: |_, _, _| {},
        rail: None,
        side_left: false,
        plays: false,
    }
}

fn bottom(ui: &mut Ui, app: &mut AppState) {
    let area = ui.max_rect();
    let mut inner =
        ui.new_child(UiBuilder::new().max_rect(area.shrink2(eframe::egui::vec2(10.0, 8.0))));
    inner.spacing_mut().item_spacing.y = 10.0;
    let choices: Vec<_> = CATEGORIES
        .iter()
        .map(|(glyph, key)| Choice {
            icon: Some(*glyph),
            label: tr(key),
        })
        .collect();
    let mut index = BuildCategory::ALL
        .iter()
        .position(|category| *category == app.category)
        .unwrap_or(0);
    let before = index;
    choice_row::show(
        &mut inner,
        &choices,
        &mut index,
        &choice_row::tab_style(typography::label_md(), 16.0),
        4.0,
    );
    if index != before {
        app.category = BuildCategory::ALL[index];
        match app.category {
            BuildCategory::Nature => app.landscape.tool = Tool::Scatter,
            BuildCategory::WaterTerrain if app.landscape.tool == Tool::Scatter => {
                app.landscape.tool = Tool::Raise
            }
            _ => {}
        }
    }
    if app.category == BuildCategory::Sky {
        ScrollArea::vertical()
            .id_salt("build-sky")
            .auto_shrink([false, false])
            .show(&mut inner, |ui| {
                if environment_panel::show(ui, &mut app.environment) {
                    ui.ctx().request_repaint();
                }
            });
    }
}

fn side(ui: &mut Ui, app: &mut AppState) {
    let area = ui.max_rect().shrink(10.0);
    let mut top = ui.new_child(UiBuilder::new().max_rect(area));
    top.spacing_mut().item_spacing.y = 8.0;
    ground_section::show(&mut top, app);
    let rest = Rect::from_min_max(pos2(area.left(), top.min_rect().bottom()), area.max);
    let mut body = ui.new_child(UiBuilder::new().max_rect(rest));
    body.set_clip_rect(rest.intersect(ui.clip_rect()));
    body.spacing_mut().item_spacing.y = 8.0;
    match app.category {
        BuildCategory::Objects => objects_panel::panel(&mut body, &mut app.objects),
        BuildCategory::Lights => lighting_panel(&mut body, &mut app.lighting),
        _ => {
            ScrollArea::vertical()
                .id_salt("build-side")
                .auto_shrink([false, false])
                .show(&mut body, |ui| match app.category {
                    BuildCategory::Nature | BuildCategory::WaterTerrain => {
                        if landscape_panel(ui, &mut app.landscape) {
                            garden::plant_garden(app);
                        }
                    }
                    BuildCategory::Materials => materials(ui, app),
                    _ => {
                        text::label(
                            ui,
                            tr("cat.sky_hint"),
                            typography::body_sm(),
                            colors::OUTLINE,
                        );
                    }
                });
        }
    }
}

fn materials(ui: &mut Ui, app: &mut AppState) {
    text::label(
        ui,
        tr("cat.materials_hint"),
        typography::body_sm(),
        colors::OUTLINE,
    );
    if Button::new(tr("cat.open_materials"), ButtonStyle::Pale)
        .icon(icons::TEXTURE, 16.0)
        .show(ui)
        .clicked()
    {
        app.navigate(Screen::MaterialEditor);
    }
}
