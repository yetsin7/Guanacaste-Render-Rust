//! The bottom-left dock: mode tabs, object categories, placement tools and the cursor readout.

use crate::components::{
    choice_row::{self, Choice},
    frames, icons,
    layout::split_row,
    tag::Tag,
    text,
};
use crate::config::theme::{colors, typography};
use crate::features::build::models::BuildState;
use crate::services::localization::tr;
use crate::services::sample_data::build as sample;
use eframe::egui::{self, Align2, Pos2, ScrollArea, Ui};

/// Maximum dock width in points.
pub const MAX_WIDTH: f32 = 760.0;

const TABS: [(char, &str); 4] = [
    (icons::VIEW_IN_AR, "build.tab_objects"),
    (icons::TEXTURE, "build.tab_materials"),
    (icons::TERRAIN, "build.tab_landscape"),
    (icons::CLOUD, "build.tab_weather"),
];
const CATEGORIES: [(char, &str); 8] = [
    (icons::UPLOAD, "build.cat_imported"),
    (icons::NATURE, "build.cat_nature"),
    (icons::PEOPLE, "build.cat_people"),
    (icons::CAR, "build.cat_vehicles"),
    (icons::SEAT, "build.cat_indoor"),
    (icons::OUTDOOR, "build.cat_outdoor"),
    (icons::LIGHT, "build.cat_lights"),
    (icons::EFFECTS, "build.cat_effects"),
];
const TOOLS: [(char, &str); 6] = [
    (icons::TOUCH, "build.tool_place"),
    (icons::SELECT, "build.tool_select"),
    (icons::DELETE, "build.tool_delete"),
    (icons::MOVE, "build.tool_move"),
    (icons::ROTATE, "build.tool_rotate"),
    (icons::SCALE, "build.tool_scale"),
];

/// Draws the dock with its bottom-left corner at `corner`, at most `max_width` wide; returns true when the
/// Materials tab was chosen.
pub fn show(ui: &mut Ui, corner: Pos2, max_width: f32, state: &mut BuildState) -> bool {
    let width = max_width.min(MAX_WIDTH);
    let mut opened_materials = false;
    egui::Area::new(egui::Id::new("build-dock"))
        .order(egui::Order::Middle)
        .pivot(Align2::LEFT_BOTTOM)
        .fixed_pos(corner)
        .show(ui.ctx(), |ui| {
            ui.spacing_mut().item_spacing.y = 8.0;
            frames::overlay(4).show(ui, |ui| {
                let choices: Vec<_> = TABS
                    .iter()
                    .map(|(glyph, key)| Choice {
                        icon: Some(*glyph),
                        label: tr(key),
                    })
                    .collect();
                let mut style = choice_row::tab_style(typography::body_sm(), 16.0);
                style.selected_font = Some(typography::headline_sm());
                let previous = state.tab;
                choice_row::show(ui, &choices, &mut state.tab, &style, 4.0);
                opened_materials = state.tab == 1 && previous != 1;
                if opened_materials {
                    state.tab = previous;
                }
            });
            frames::overlay(10).show(ui, |ui| {
                ui.set_width(width - 20.0);
                ui.spacing_mut().item_spacing.y = 10.0;
                categories(ui, state);
                tools(ui, state);
            });
        });
    opened_materials
}

fn categories(ui: &mut Ui, state: &mut BuildState) {
    ScrollArea::horizontal()
        .id_salt("build-categories")
        .auto_shrink([false, true])
        .show(ui, |ui| {
            let choices: Vec<_> = CATEGORIES
                .iter()
                .map(|(glyph, key)| Choice {
                    icon: Some(*glyph),
                    label: tr(key),
                })
                .collect();
            let mut style = choice_row::pill_style(typography::label_sm());
            style.selected_font = Some(typography::mono_medium(11.0));
            choice_row::show(ui, &choices, &mut state.category, &style, 4.0);
        });
}

fn tools(ui: &mut Ui, state: &mut BuildState) {
    split_row(
        ui,
        32.0,
        |ui| {
            let choices: Vec<_> = TOOLS
                .iter()
                .map(|(glyph, key)| Choice {
                    icon: Some(*glyph),
                    label: tr(key),
                })
                .collect();
            let mut style = choice_row::tool_style(typography::label_md());
            style.selected_font = Some(typography::label_md());
            choice_row::show(ui, &choices, &mut state.tool, &style, 4.0);
        },
        cursor_readout,
    );
}

fn cursor_readout(ui: &mut Ui) {
    let axes = [
        ("X", colors::PRIMARY),
        ("Y", colors::TERTIARY),
        ("Z", colors::SECONDARY),
    ];
    let mut tag =
        Tag::new(text::caps(tr("build.cursor")), colors::OUTLINE).font(typography::label_md());
    for (index, (axis, color)) in axes.into_iter().enumerate() {
        if index > 0 {
            tag = tag.part(" ,", colors::OUTLINE);
        }
        tag = tag.part(format!("  {axis}: {:.2} m", sample::CURSOR[index]), color);
    }
    tag.trail_icon(icons::RESTORE, 12.0, colors::OUTLINE)
        .gap(10.0)
        .padding(12.0, 6.0)
        .fill(colors::SURFACE_LOWEST)
        .show(ui);
}
