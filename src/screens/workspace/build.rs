//! Build mode: object properties on the right; category tabs (nature, water and terrain, objects, lights,
//! materials, sky) with the tools of the selected one in the bottom drawer.

use super::{ModeSlots, Panel};
use crate::components::button::{Button, ButtonStyle};
use crate::components::choice_row::{self, Choice};
use crate::components::{icons, text};
use crate::config::theme::{colors, typography};
use crate::features::build::components::{environment_tools, object_properties};
use crate::features::viewport::components::reticle;
use crate::models::navigation::{BuildCategory, Screen};
use crate::models::AppState;
use crate::services::localization::{tr, trf};
use eframe::egui::{pos2, ScrollArea, Ui, UiBuilder};

const BOTTOM_HEIGHT: f32 = 196.0;

/// Icon and label key of each category, in [`BuildCategory::ALL`] order.
const CATEGORIES: [(char, &str); 6] = [
    (icons::NATURE, "cat.nature"),
    (icons::TERRAIN, "cat.water_terrain"),
    (icons::VIEW_IN_AR, "cat.objects"),
    (icons::LIGHT, "cat.lights"),
    (icons::TEXTURE, "cat.materials"),
    (icons::CLOUD, "cat.sky"),
];

/// Placement tools shared by the categories that put things in the scene: icon and label key.
const TOOLS: [(char, &str); 6] = [
    (icons::TOUCH, "build.tool_place"),
    (icons::SELECT, "build.tool_select"),
    (icons::DELETE, "build.tool_delete"),
    (icons::MOVE, "build.tool_move"),
    (icons::ROTATE, "build.tool_rotate"),
    (icons::SCALE, "build.tool_scale"),
];

/// Slots of build mode.
pub fn slots() -> ModeSlots {
    ModeSlots {
        side: (
            "panel.properties",
            icons::TUNE,
            object_properties::WIDTH,
            properties as Panel,
        ),
        bottom: (
            "panel.categories",
            icons::GRID_VIEW,
            BOTTOM_HEIGHT,
            categories as Panel,
        ),
        overlay: |ui, rect, _| {
            let center = pos2(
                rect.left() + rect.width() * 0.42,
                rect.top() + rect.height() * 0.46,
            );
            reticle::show(ui, center, tr("viewport.cursor"), tr("build.snap"));
        },
        plays: false,
    }
}

fn properties(ui: &mut Ui, app: &mut AppState) {
    let area = ui.max_rect();
    object_properties::show(
        ui,
        area.min,
        area.height(),
        &mut app.build,
        &mut app.textures,
    );
}

fn categories(ui: &mut Ui, app: &mut AppState) {
    let area = ui.max_rect();
    let mut inner = ui.new_child(UiBuilder::new().max_rect(area.shrink(10.0)));
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
    choice_row::show(
        &mut inner,
        &choices,
        &mut index,
        &choice_row::tab_style(typography::label_md(), 16.0),
        4.0,
    );
    app.category = BuildCategory::ALL[index];
    ScrollArea::vertical()
        .id_salt("build-category-body")
        .auto_shrink([false, false])
        .show(&mut inner, |ui| {
            ui.spacing_mut().item_spacing.y = 10.0;
            match app.category {
                BuildCategory::WaterTerrain => environment_tools::landscape(ui, &mut app.build),
                BuildCategory::Sky => environment_tools::weather(ui, &mut app.build),
                BuildCategory::Materials => materials(ui, app),
                _ => placement(ui, app, index),
            }
        });
}

/// Tools and the pointer to the library for the categories that place content.
fn placement(ui: &mut Ui, app: &mut AppState, index: usize) {
    let choices: Vec<_> = TOOLS
        .iter()
        .map(|(glyph, key)| Choice {
            icon: Some(*glyph),
            label: tr(key),
        })
        .collect();
    choice_row::show(
        ui,
        &choices,
        &mut app.build.tool,
        &choice_row::tool_style(typography::label_md()),
        4.0,
    );
    let name = tr(CATEGORIES[index].1);
    text::label(
        ui,
        trf("cat.slot_hint", &[name]),
        typography::body_sm(),
        colors::OUTLINE,
    );
    if Button::new(tr("cat.open_library"), ButtonStyle::Container)
        .icon(icons::LIBRARY, 16.0)
        .show(ui)
        .clicked()
    {
        app.navigate(Screen::Library);
    }
}

fn materials(ui: &mut Ui, app: &mut AppState) {
    text::label(
        ui,
        tr("cat.materials_hint"),
        typography::body_sm(),
        colors::OUTLINE,
    );
    if Button::new(tr("cat.open_materials"), ButtonStyle::Container)
        .icon(icons::TEXTURE, 16.0)
        .show(ui)
        .clicked()
    {
        app.navigate(Screen::MaterialEditor);
    }
}
